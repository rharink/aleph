use crate::error::CodecError;

pub(crate) struct BitWriter {
    bytes: Vec<u8>,
    acc: u32,
    nbits: u32,
}

impl BitWriter {
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(cap),
            acc: 0,
            nbits: 0,
        }
    }

    pub(crate) fn write_bits(&mut self, value: u32, count: u32) {
        if count == 0 {
            return;
        }
        let masked = value & mask(count);
        self.acc = (self.acc << count) | masked;
        self.nbits += count;
        while self.nbits >= 8 {
            self.nbits -= 8;
            let byte =
                u8::try_from((self.acc >> self.nbits) & 0xFF).expect("byte masked to 8 bits");
            self.emit(byte);
        }
        self.acc &= mask(self.nbits);
    }

    // JPEG pads the final partial byte with 1-bits, then byte-stuffs as usual.
    pub(crate) fn finish(mut self) -> Vec<u8> {
        if self.nbits > 0 {
            let pad = 8 - self.nbits;
            self.write_bits(mask(pad), pad);
        }
        self.bytes
    }

    fn emit(&mut self, byte: u8) {
        self.bytes.push(byte);
        if byte == 0xFF {
            self.bytes.push(0x00);
        }
    }
}

pub(crate) struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    acc: u32,
    nbits: u32,
    marker_hit: bool,
}

impl<'a> BitReader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            acc: 0,
            nbits: 0,
            marker_hit: false,
        }
    }

    pub(crate) fn read_bits(&mut self, count: u32) -> Result<u32, CodecError> {
        if count == 0 {
            return Ok(0);
        }
        while self.nbits < count {
            let byte = self.next_byte().ok_or(CodecError::UnexpectedEof)?;
            self.acc = (self.acc << 8) | u32::from(byte);
            self.nbits += 8;
        }
        self.nbits -= count;
        let value = (self.acc >> self.nbits) & mask(count);
        self.acc &= mask(self.nbits);
        Ok(value)
    }

    pub(crate) fn read_bit(&mut self) -> Result<u32, CodecError> {
        self.read_bits(1)
    }

    // Byte offset of the next unread byte. After a complete decode this points at
    // the first byte past the entropy data (where the terminating marker begins).
    pub(crate) fn position(&self) -> usize {
        self.pos
    }

    // Unstuffs `FF 00` into a literal `0xFF` data byte. A `0xFF` followed by any
    // non-zero byte is a marker and ends the entropy-coded segment.
    fn next_byte(&mut self) -> Option<u8> {
        if self.marker_hit {
            return None;
        }
        let byte = *self.data.get(self.pos)?;
        if byte != 0xFF {
            self.pos += 1;
            return Some(byte);
        }
        match self.data.get(self.pos + 1) {
            Some(0x00) => {
                self.pos += 2;
                Some(0xFF)
            }
            Some(_) => {
                self.marker_hit = true;
                None
            }
            None => {
                self.pos += 1;
                None
            }
        }
    }
}

fn mask(count: u32) -> u32 {
    if count >= 32 {
        u32::MAX
    } else {
        (1u32 << count) - 1
    }
}
