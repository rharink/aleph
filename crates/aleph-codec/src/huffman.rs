use crate::error::CodecError;

// Number of magnitude categories (SSSS = 0..=16) plus one reserved code point
// that guarantees the all-ones codeword stays unused (ITU-T81 Annex K).
const SYMBOL_COUNT: usize = 17;
const RESERVED: usize = SYMBOL_COUNT;
const FREQ_LEN: usize = SYMBOL_COUNT + 1;
const MAX_CLEN: usize = 32;

/// Canonical DC Huffman table over the SSSS categories, ready for both
/// serialization into a `DHT` segment and direct encoding.
pub(crate) struct HuffmanTable {
    pub(crate) bits: [u8; 16],
    pub(crate) values: Vec<u8>,
    codes: [u16; SYMBOL_COUNT],
    sizes: [u8; SYMBOL_COUNT],
}

impl HuffmanTable {
    pub(crate) fn from_histogram(histogram: &[u32; SYMBOL_COUNT]) -> Self {
        let codesize = code_sizes(histogram);
        let bits = bit_lengths(&codesize);
        let values = sorted_symbols(&codesize, &bits);

        let mut codes = [0u16; SYMBOL_COUNT];
        let mut sizes = [0u8; SYMBOL_COUNT];
        let mut next_code: u16 = 0;
        let mut k = 0;
        for (len_minus_one, &count) in bits.iter().enumerate() {
            for _ in 0..count {
                let symbol = usize::from(values[k]);
                sizes[symbol] = u8::try_from(len_minus_one + 1).expect("length <= 16");
                codes[symbol] = next_code;
                next_code += 1;
                k += 1;
            }
            next_code <<= 1;
        }

        Self {
            bits,
            values,
            codes,
            sizes,
        }
    }

    pub(crate) fn code(&self, symbol: u8) -> (u16, u8) {
        let s = usize::from(symbol);
        (self.codes[s], self.sizes[s])
    }
}

/// Decode-side view rebuilt from a serialized `(bits, values)` pair.
pub(crate) struct HuffmanDecoder {
    min_code: [i32; 17],
    max_code: [i32; 17],
    val_ptr: [usize; 17],
    values: Vec<u8>,
}

impl HuffmanDecoder {
    pub(crate) fn new(bits: &[u8; 16], values: Vec<u8>) -> Result<Self, CodecError> {
        let total: usize = bits.iter().map(|&c| usize::from(c)).sum();
        if total != values.len() {
            return Err(CodecError::Malformed("Huffman table count mismatch"));
        }

        let mut min_code = [0i32; 17];
        let mut max_code = [-1i32; 17];
        let mut val_ptr = [0usize; 17];
        let mut code: i32 = 0;
        let mut k: usize = 0;
        for len in 1..=16usize {
            let count = i32::from(bits[len - 1]);
            if count > 0 {
                val_ptr[len] = k;
                min_code[len] = code;
                code += count;
                if code > (1i32 << len) {
                    return Err(CodecError::Malformed("over-subscribed Huffman table"));
                }
                max_code[len] = code - 1;
                k += usize::try_from(count).expect("count is non-negative");
            }
            code <<= 1;
        }

        Ok(Self {
            min_code,
            max_code,
            val_ptr,
            values,
        })
    }

    pub(crate) fn decode_symbol(
        &self,
        mut read_bit: impl FnMut() -> Result<u32, CodecError>,
    ) -> Result<u8, CodecError> {
        let mut code: i32 = 0;
        for len in 1..=16usize {
            code = (code << 1) | i32::try_from(read_bit()?).expect("bit is 0 or 1");
            if self.max_code[len] >= 0 && code <= self.max_code[len] {
                let offset =
                    usize::try_from(code - self.min_code[len]).expect("offset within length");
                return Ok(self.values[self.val_ptr[len] + offset]);
            }
        }
        Err(CodecError::Malformed("undecodable Huffman code"))
    }
}

// ITU-T81 Figure K.1: per-symbol code lengths via the frequency-merge tree, with
// a dummy reserved symbol (frequency 1) so the all-ones codeword is never
// assigned to a real symbol.
fn code_sizes(histogram: &[u32; SYMBOL_COUNT]) -> [usize; FREQ_LEN] {
    let mut freq = [0u64; FREQ_LEN];
    for (i, &f) in histogram.iter().enumerate() {
        freq[i] = u64::from(f);
    }
    freq[RESERVED] = 1;

    let mut codesize = [0usize; FREQ_LEN];
    let mut others = [-1i32; FREQ_LEN];

    while let Some(v1) = least_frequent(&freq, None) {
        let Some(v2) = least_frequent(&freq, Some(v1)) else {
            break;
        };

        freq[v1] += freq[v2];
        freq[v2] = 0;

        codesize[v1] += 1;
        let mut chain = v1;
        while others[chain] >= 0 {
            chain = usize::try_from(others[chain]).expect("chain index non-negative");
            codesize[chain] += 1;
        }
        others[chain] = i32::try_from(v2).expect("symbol index fits i32");

        codesize[v2] += 1;
        let mut chain = v2;
        while others[chain] >= 0 {
            chain = usize::try_from(others[chain]).expect("chain index non-negative");
            codesize[chain] += 1;
        }
    }
    codesize
}

// Figure K.3: count codes per length, then redistribute lengths exceeding 16
// bits, and finally drop the reserved code point from the longest length.
fn bit_lengths(codesize: &[usize; FREQ_LEN]) -> [u8; 16] {
    let mut counts = [0u32; MAX_CLEN + 1];
    for &len in codesize {
        if len > 0 {
            counts[len] += 1;
        }
    }

    let mut i = MAX_CLEN;
    while i > 16 {
        while counts[i] > 0 {
            let mut j = i - 2;
            while counts[j] == 0 {
                j -= 1;
            }
            counts[i] -= 2;
            counts[i - 1] += 1;
            counts[j + 1] += 2;
            counts[j] -= 1;
        }
        i -= 1;
    }

    while counts[i] == 0 {
        i -= 1;
    }
    counts[i] -= 1;

    let mut bits = [0u8; 16];
    for (len, slot) in bits.iter_mut().enumerate() {
        *slot = u8::try_from(counts[len + 1]).expect("category count fits u8");
    }
    bits
}

// Figure K.4: emit real symbols ordered by code length, ascending by symbol
// value within a length. The reserved symbol is excluded.
fn sorted_symbols(codesize: &[usize; FREQ_LEN], bits: &[u8; 16]) -> Vec<u8> {
    let total: usize = bits.iter().map(|&c| usize::from(c)).sum();
    let mut values = Vec::with_capacity(total);
    for len in 1..=MAX_CLEN {
        for (symbol, &size) in codesize.iter().take(SYMBOL_COUNT).enumerate() {
            if size == len {
                values.push(u8::try_from(symbol).expect("symbol < 17"));
            }
        }
    }
    values
}

// Smallest non-zero frequency; ties resolve to the largest index.
fn least_frequent(freq: &[u64; FREQ_LEN], exclude: Option<usize>) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_freq = u64::MAX;
    for (i, &f) in freq.iter().enumerate() {
        if f == 0 || Some(i) == exclude {
            continue;
        }
        if f <= best_freq {
            best_freq = f;
            best = Some(i);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::HuffmanDecoder;
    use super::HuffmanTable;
    use crate::error::CodecError;

    fn decode_via(table: &HuffmanTable, symbol: u8) -> u8 {
        let decoder = HuffmanDecoder::new(&table.bits, table.values.clone()).expect("valid table");
        let (code, size) = table.code(symbol);
        let mut remaining = size;
        let bits = code;
        let next = move || -> Result<u32, CodecError> {
            remaining -= 1;
            Ok(u32::from((bits >> remaining) & 1))
        };
        decoder.decode_symbol(next).expect("decodes")
    }

    #[test]
    fn canonical_codes_round_trip_through_decoder() {
        let mut histogram = [0u32; 17];
        for (i, slot) in histogram.iter_mut().enumerate() {
            *slot = u32::try_from((i + 1) * 7).expect("small");
        }
        let table = HuffmanTable::from_histogram(&histogram);
        for symbol in 0..17u8 {
            assert_eq!(decode_via(&table, symbol), symbol);
        }
    }

    #[test]
    fn all_codes_within_16_bits() {
        let mut histogram = [1u32; 17];
        histogram[0] = 1_000_000;
        let table = HuffmanTable::from_histogram(&histogram);
        let total: usize = table.bits.iter().map(|&c| usize::from(c)).sum();
        assert_eq!(total, table.values.len());
        for symbol in 0..17u8 {
            let (_code, size) = table.code(symbol);
            assert!((1..=16).contains(&size));
        }
    }

    #[test]
    fn reserved_all_ones_codeword_unused() {
        let mut histogram = [0u32; 17];
        histogram[0] = 5;
        histogram[1] = 5;
        let table = HuffmanTable::from_histogram(&histogram);
        for symbol in 0..17u8 {
            let (code, size) = table.code(symbol);
            if size == 0 {
                continue;
            }
            let all_ones = (1u16 << size) - 1;
            assert_ne!(code, all_ones, "symbol {symbol} got the reserved codeword");
        }
    }

    #[test]
    fn single_symbol_histogram_gets_length_one() {
        let mut histogram = [0u32; 17];
        histogram[0] = 42;
        let table = HuffmanTable::from_histogram(&histogram);
        let (_code, size) = table.code(0);
        assert_eq!(size, 1);
    }
}
