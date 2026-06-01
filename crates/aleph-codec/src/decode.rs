use crate::bitio::BitReader;
use crate::error::CodecError;
use crate::huffman::HuffmanDecoder;
use crate::marker;
use crate::sample;

pub struct Decoded {
    pub width: u16,
    pub height: u16,
    pub components: u8,
    pub precision: u8,
    pub samples: Vec<u16>,
    /// Number of input bytes consumed, up to and including the EOI marker. Bytes
    /// after this are trailing data the caller may need to preserve (DNG tiles
    /// and strips can carry padding past EOI).
    pub consumed: usize,
}

/// Decode a lossless-JPEG (SOF3, process 14) bitstream produced by `encode`.
///
/// # Errors
/// Returns `CodecError` on malformed, truncated, or unsupported bitstreams.
/// Never panics on adversarial input.
pub fn decode(bytes: &[u8]) -> Result<Decoded, CodecError> {
    let mut cur = Cursor::new(bytes);

    let soi = cur.read_u16()?;
    if soi != marker::SOI {
        return Err(CodecError::InvalidMarker {
            expected: marker::SOI,
            found: soi,
        });
    }

    let mut frame: Option<FrameHeader> = None;
    let mut dc_tables: [Option<HuffmanDecoder>; 4] = [None, None, None, None];

    let scan = loop {
        match cur.read_u16()? {
            marker::SOF3 => frame = Some(read_frame_header(&mut cur)?),
            marker::DHT => read_huffman_tables(&mut cur, &mut dc_tables)?,
            marker::SOS => break read_scan_header(&mut cur, frame.as_ref())?,
            other => return Err(CodecError::UnsupportedMarker(other)),
        }
    };

    let frame = frame.ok_or(CodecError::Malformed("missing SOF3 frame header"))?;

    let mut comp_tables = Vec::with_capacity(usize::from(frame.components));
    for &td in &scan.table_per_component {
        let table = dc_tables
            .get(td)
            .and_then(Option::as_ref)
            .ok_or(CodecError::Malformed(
                "scan references undefined Huffman table",
            ))?;
        comp_tables.push(table);
    }

    let (samples, consumed) = read_entropy(&bytes[cur.pos..], &frame, &comp_tables)?;

    // The entropy segment must be terminated by EOI. Without this check a stream
    // with a stripped EOI, or with extra bytes spliced in after enough entropy to
    // satisfy the sample count, would still decode as "valid". Trailing bytes
    // after EOI are tolerated (real DNG tiles pad past it).
    cur.pos += consumed;
    let terminator = cur.read_u16()?;
    if terminator != marker::EOI {
        return Err(CodecError::Malformed("entropy not terminated by EOI"));
    }

    Ok(Decoded {
        width: frame.width,
        height: frame.height,
        components: frame.components,
        precision: frame.precision,
        samples,
        consumed: cur.pos,
    })
}

struct FrameHeader {
    width: u16,
    height: u16,
    components: u8,
    precision: u8,
}

fn read_frame_header(cur: &mut Cursor<'_>) -> Result<FrameHeader, CodecError> {
    let _length = cur.read_u16()?;
    let precision = cur.read_u8()?;
    let height = cur.read_u16()?;
    let width = cur.read_u16()?;
    let components = cur.read_u8()?;

    if !(2..=16).contains(&precision) {
        return Err(CodecError::InvalidPrecision(precision));
    }
    if !(1..=4).contains(&components) {
        return Err(CodecError::InvalidComponents(components));
    }
    if width == 0 || height == 0 {
        return Err(CodecError::InvalidDimensions { width, height });
    }

    for _ in 0..components {
        let _ci = cur.read_u8()?;
        let hv = cur.read_u8()?;
        let _tq = cur.read_u8()?;
        if hv != 0x11 {
            return Err(CodecError::Unsupported("sampling factors other than 1x1"));
        }
    }

    Ok(FrameHeader {
        width,
        height,
        components,
        precision,
    })
}

fn read_huffman_tables(
    cur: &mut Cursor<'_>,
    dc_tables: &mut [Option<HuffmanDecoder>; 4],
) -> Result<(), CodecError> {
    let length = usize::from(cur.read_u16()?);
    if length < 2 {
        return Err(CodecError::Malformed("invalid DHT length"));
    }
    let end = cur.pos + length - 2;

    while cur.pos < end {
        let tc_th = cur.read_u8()?;
        let counts = cur.read_bytes(16)?;
        let mut bits = [0u8; 16];
        bits.copy_from_slice(counts);
        let total: usize = bits.iter().map(|&c| usize::from(c)).sum();
        let values = cur.read_bytes(total)?.to_vec();

        // Lossless JPEG references only DC tables (class Tc = 0); AC tables, if
        // present, are unused. Real DNG streams carry one DC table per component
        // at slots Th = 0, 1, ... so all of them must be retained.
        let class = tc_th >> 4;
        let slot = usize::from(tc_th & 0x0F);
        if class == 0 {
            if slot >= dc_tables.len() {
                return Err(CodecError::Malformed("DC Huffman table index out of range"));
            }
            dc_tables[slot] = Some(HuffmanDecoder::new(&bits, values)?);
        }
    }
    Ok(())
}

struct Scan {
    table_per_component: Vec<usize>,
}

fn read_scan_header(cur: &mut Cursor<'_>, frame: Option<&FrameHeader>) -> Result<Scan, CodecError> {
    let _length = cur.read_u16()?;
    let ns = cur.read_u8()?;
    if let Some(frame) = frame {
        if usize::from(ns) != usize::from(frame.components) {
            return Err(CodecError::Malformed(
                "scan component count differs from frame",
            ));
        }
    }

    let mut table_per_component = Vec::with_capacity(usize::from(ns));
    for _ in 0..ns {
        let _cs = cur.read_u8()?;
        let td_ta = cur.read_u8()?;
        // Td (the DC table selector) is the high nibble.
        table_per_component.push(usize::from(td_ta >> 4));
    }

    let predictor = cur.read_u8()?;
    let _se = cur.read_u8()?;
    let _ah_al = cur.read_u8()?;
    if predictor != 1 {
        return Err(CodecError::Unsupported("lossless predictor other than 1"));
    }

    Ok(Scan {
        table_per_component,
    })
}

fn read_entropy(
    data: &[u8],
    frame: &FrameHeader,
    comp_tables: &[&HuffmanDecoder],
) -> Result<(Vec<u16>, usize), CodecError> {
    let width = usize::from(frame.width);
    let height = usize::from(frame.height);
    let components = usize::from(frame.components);

    let count = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(components))
        .ok_or(CodecError::Malformed("frame dimensions overflow"))?;

    let mut samples = Vec::new();
    samples
        .try_reserve_exact(count)
        .map_err(|_| CodecError::Malformed("frame too large to allocate"))?;
    samples.resize(count, 0u16);

    let mut reader = BitReader::new(data);
    for y in 0..height {
        for x in 0..width {
            for (c, table) in comp_tables.iter().enumerate() {
                let ssss = table.decode_symbol(|| reader.read_bit())?;
                let d = read_difference(&mut reader, ssss)?;
                let i = sample::index(x, y, c, width, components);
                let px = sample::predict(&samples, x, y, c, width, components, frame.precision);
                samples[i] = px.wrapping_add(u16::from_ne_bytes(d.to_ne_bytes()));
            }
        }
    }

    Ok((samples, reader.position()))
}

// Inverse of the JPEG value mapping (ITU-T81 EXTEND), including the
// SSSS == 16 special case (d == -32768) which carries no value bits.
fn read_difference(reader: &mut BitReader<'_>, ssss: u8) -> Result<i16, CodecError> {
    if ssss == 0 {
        return Ok(0);
    }
    if ssss == 16 {
        return Ok(i16::MIN);
    }
    if ssss > 16 {
        return Err(CodecError::Malformed("invalid magnitude category"));
    }

    let s = u32::from(ssss);
    let value = reader.read_bits(s)?;
    let threshold = 1u32 << (s - 1);
    let signed = i32::try_from(value).expect("value fits i32");
    let d = if value >= threshold {
        signed
    } else {
        signed - ((1i32 << s) - 1)
    };
    i16::try_from(d).map_err(|_| CodecError::Malformed("difference out of range"))
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, CodecError> {
        let byte = *self.data.get(self.pos).ok_or(CodecError::UnexpectedEof)?;
        self.pos += 1;
        Ok(byte)
    }

    fn read_u16(&mut self) -> Result<u16, CodecError> {
        let hi = self.read_u8()?;
        let lo = self.read_u8()?;
        Ok((u16::from(hi) << 8) | u16::from(lo))
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], CodecError> {
        let end = self.pos.checked_add(n).ok_or(CodecError::UnexpectedEof)?;
        let slice = self
            .data
            .get(self.pos..end)
            .ok_or(CodecError::UnexpectedEof)?;
        self.pos = end;
        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::decode;
    use crate::encode::encode;
    use crate::encode::Frame;
    use proptest::collection::vec;
    use proptest::prelude::any;
    use proptest::proptest;

    #[test]
    fn rejects_empty() {
        assert!(decode(&[]).is_err());
    }

    #[test]
    fn rejects_random_garbage() {
        let garbage: Vec<u8> = (0..256u32)
            .map(|i| u8::try_from((i * 31 + 7) % 256).expect("mod 256"))
            .collect();
        assert!(decode(&garbage).is_err());
    }

    #[test]
    fn rejects_bad_soi() {
        assert!(decode(&[0x00, 0x00, 0x12, 0x34]).is_err());
    }

    #[test]
    fn rejects_truncated_stream() {
        let frame = Frame {
            width: 8,
            height: 8,
            components: 2,
            precision: 10,
            samples: &(0..(8u16 * 8 * 2))
                .map(|v| (v * 5) % 1024)
                .collect::<Vec<u16>>(),
        };
        let bytes = encode(&frame).expect("encode succeeds");
        for cut in 0..bytes.len() {
            // Any prefix must error without panicking; a full stream decodes.
            let _ = decode(&bytes[..cut]);
        }
        assert!(decode(&bytes).is_ok());
    }

    #[test]
    fn rejects_truncated_header() {
        assert!(decode(&[0xFF, 0xD8, 0xFF, 0xC3, 0x00]).is_err());
    }

    fn sample_stream() -> Vec<u8> {
        let frame = Frame {
            width: 6,
            height: 5,
            components: 2,
            precision: 12,
            samples: &(0..(6u16 * 5 * 2))
                .map(|v| (v * 37) % 4096)
                .collect::<Vec<u16>>(),
        };
        encode(&frame).expect("encode succeeds")
    }

    #[test]
    fn rejects_stream_with_eoi_stripped() {
        let bytes = sample_stream();
        assert_eq!(&bytes[bytes.len() - 2..], &[0xFF, 0xD9]);
        // Drop the trailing EOI: the sample count is still satisfiable, so without
        // the termination check this would decode as valid.
        assert!(decode(&bytes[..bytes.len() - 2]).is_err());
    }

    #[test]
    fn rejects_bytes_spliced_before_eoi() {
        let bytes = sample_stream();
        let (body, eoi) = bytes.split_at(bytes.len() - 2);
        let mut tampered = body.to_vec();
        tampered.extend_from_slice(&[0x12, 0x34]);
        tampered.extend_from_slice(eoi);
        assert!(decode(&tampered).is_err());
    }

    #[test]
    fn tolerates_trailing_bytes_after_eoi() {
        // Real DNG tiles pad past EOI; trailing bytes must not fail the decode.
        let mut bytes = sample_stream();
        bytes.extend_from_slice(&[0x00, 0x00, 0xDE, 0xAD]);
        assert!(decode(&bytes).is_ok());
    }

    proptest! {
        // The decoder's documented contract: never panic on adversarial input,
        // only return Err. Most random bytes bounce off the SOI/marker checks.
        #[test]
        fn decode_never_panics_on_arbitrary_bytes(data in vec(any::<u8>(), 0..4096)) {
            let _ = decode(&data);
        }

        // Flipping bytes of a valid stream drives malformed data deep into the
        // entropy decoder — paths arbitrary bytes never reach.
        #[test]
        fn decode_never_panics_on_flipped_stream(flips in vec(any::<usize>(), 0..32)) {
            let mut bytes = sample_stream();
            let len = bytes.len();
            for f in flips {
                bytes[f % len] ^= 0xFF;
            }
            let _ = decode(&bytes);
        }
    }
}
