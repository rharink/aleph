//! Byte order and the TIFF field-value model.

/// Byte order of a TIFF stream.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Endian {
    /// `II` — least significant byte first.
    Little,
    /// `MM` — most significant byte first.
    Big,
}

/// A decoded TIFF field value. The variant fixes the on-disk field type.
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    /// TIFF type 1.
    Byte(Vec<u8>),
    /// TIFF type 2 (raw bytes including any trailing NUL).
    Ascii(Vec<u8>),
    /// TIFF type 3.
    Short(Vec<u16>),
    /// TIFF type 4.
    Long(Vec<u32>),
    /// TIFF type 5 (numerator, denominator).
    Rational(Vec<(u32, u32)>),
    /// TIFF type 6.
    SByte(Vec<i8>),
    /// TIFF type 7.
    Undefined(Vec<u8>),
    /// TIFF type 8.
    SShort(Vec<i16>),
    /// TIFF type 9.
    SLong(Vec<i32>),
    /// TIFF type 10 (numerator, denominator).
    SRational(Vec<(i32, i32)>),
    /// TIFF type 11.
    Float(Vec<f32>),
    /// TIFF type 12.
    Double(Vec<f64>),
}

impl Value {
    /// The on-disk TIFF field type code (1..=12).
    #[must_use]
    pub fn type_code(&self) -> u16 {
        match self {
            Value::Byte(_) => 1,
            Value::Ascii(_) => 2,
            Value::Short(_) => 3,
            Value::Long(_) => 4,
            Value::Rational(_) => 5,
            Value::SByte(_) => 6,
            Value::Undefined(_) => 7,
            Value::SShort(_) => 8,
            Value::SLong(_) => 9,
            Value::SRational(_) => 10,
            Value::Float(_) => 11,
            Value::Double(_) => 12,
        }
    }

    /// Number of elements, as stored in the IFD entry count field.
    #[must_use]
    pub fn count(&self) -> u32 {
        u32::try_from(self.elem_count()).unwrap_or(u32::MAX)
    }

    /// Interpret `Byte`/`Short`/`Long` element-wise as `u32`. `None` otherwise.
    #[must_use]
    pub fn as_u32_vec(&self) -> Option<Vec<u32>> {
        match self {
            Value::Byte(v) => Some(v.iter().map(|&x| u32::from(x)).collect()),
            Value::Short(v) => Some(v.iter().map(|&x| u32::from(x)).collect()),
            Value::Long(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Encode this value's payload (no padding) in `endian` order.
    pub(crate) fn encode(&self, endian: Endian) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.elem_count() * self.elem_size());
        match self {
            Value::Byte(v) | Value::Undefined(v) | Value::Ascii(v) => out.extend_from_slice(v),
            Value::SByte(v) => out.extend(v.iter().map(|&x| i8::to_le_bytes(x)[0])),
            Value::Short(v) => {
                for &x in v {
                    put_u16(&mut out, endian, x);
                }
            }
            Value::SShort(v) => {
                for &x in v {
                    put_u16(&mut out, endian, u16::from_ne_bytes(x.to_ne_bytes()));
                }
            }
            Value::Long(v) => {
                for &x in v {
                    put_u32(&mut out, endian, x);
                }
            }
            Value::SLong(v) => {
                for &x in v {
                    put_u32(&mut out, endian, u32::from_ne_bytes(x.to_ne_bytes()));
                }
            }
            Value::Float(v) => {
                for &x in v {
                    put_u32(&mut out, endian, x.to_bits());
                }
            }
            Value::Double(v) => {
                for &x in v {
                    put_u64(&mut out, endian, x.to_bits());
                }
            }
            Value::Rational(v) => {
                for &(n, d) in v {
                    put_u32(&mut out, endian, n);
                    put_u32(&mut out, endian, d);
                }
            }
            Value::SRational(v) => {
                for &(n, d) in v {
                    put_u32(&mut out, endian, u32::from_ne_bytes(n.to_ne_bytes()));
                    put_u32(&mut out, endian, u32::from_ne_bytes(d.to_ne_bytes()));
                }
            }
        }
        out
    }

    fn elem_count(&self) -> usize {
        match self {
            Value::Byte(v) | Value::Undefined(v) | Value::Ascii(v) => v.len(),
            Value::SByte(v) => v.len(),
            Value::Short(v) => v.len(),
            Value::SShort(v) => v.len(),
            Value::Long(v) => v.len(),
            Value::SLong(v) => v.len(),
            Value::Float(v) => v.len(),
            Value::Double(v) => v.len(),
            Value::Rational(v) => v.len(),
            Value::SRational(v) => v.len(),
        }
    }

    fn elem_size(&self) -> usize {
        match self {
            Value::Byte(_) | Value::Ascii(_) | Value::SByte(_) | Value::Undefined(_) => 1,
            Value::Short(_) | Value::SShort(_) => 2,
            Value::Long(_) | Value::SLong(_) | Value::Float(_) => 4,
            Value::Rational(_) | Value::SRational(_) | Value::Double(_) => 8,
        }
    }
}

pub(crate) fn get_u16(endian: Endian, b: &[u8]) -> u16 {
    let arr = [b[0], b[1]];
    match endian {
        Endian::Little => u16::from_le_bytes(arr),
        Endian::Big => u16::from_be_bytes(arr),
    }
}

pub(crate) fn get_u32(endian: Endian, b: &[u8]) -> u32 {
    let arr = [b[0], b[1], b[2], b[3]];
    match endian {
        Endian::Little => u32::from_le_bytes(arr),
        Endian::Big => u32::from_be_bytes(arr),
    }
}

pub(crate) fn get_u64(endian: Endian, b: &[u8]) -> u64 {
    let arr = [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]];
    match endian {
        Endian::Little => u64::from_le_bytes(arr),
        Endian::Big => u64::from_be_bytes(arr),
    }
}

pub(crate) fn put_u16(out: &mut Vec<u8>, endian: Endian, v: u16) {
    match endian {
        Endian::Little => out.extend_from_slice(&v.to_le_bytes()),
        Endian::Big => out.extend_from_slice(&v.to_be_bytes()),
    }
}

pub(crate) fn put_u32(out: &mut Vec<u8>, endian: Endian, v: u32) {
    match endian {
        Endian::Little => out.extend_from_slice(&v.to_le_bytes()),
        Endian::Big => out.extend_from_slice(&v.to_be_bytes()),
    }
}

pub(crate) fn put_u64(out: &mut Vec<u8>, endian: Endian, v: u64) {
    match endian {
        Endian::Little => out.extend_from_slice(&v.to_le_bytes()),
        Endian::Big => out.extend_from_slice(&v.to_be_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use super::Value;

    #[test]
    fn type_codes_cover_all_variants() {
        assert_eq!(Value::Byte(vec![]).type_code(), 1);
        assert_eq!(Value::Ascii(vec![]).type_code(), 2);
        assert_eq!(Value::Short(vec![]).type_code(), 3);
        assert_eq!(Value::Long(vec![]).type_code(), 4);
        assert_eq!(Value::Rational(vec![]).type_code(), 5);
        assert_eq!(Value::SByte(vec![]).type_code(), 6);
        assert_eq!(Value::Undefined(vec![]).type_code(), 7);
        assert_eq!(Value::SShort(vec![]).type_code(), 8);
        assert_eq!(Value::SLong(vec![]).type_code(), 9);
        assert_eq!(Value::SRational(vec![]).type_code(), 10);
        assert_eq!(Value::Float(vec![]).type_code(), 11);
        assert_eq!(Value::Double(vec![]).type_code(), 12);
    }

    #[test]
    fn count_reports_element_count() {
        assert_eq!(Value::Short(vec![1, 2, 3]).count(), 3);
        assert_eq!(Value::Rational(vec![(1, 2), (3, 4)]).count(), 2);
        assert_eq!(Value::Ascii(b"abc\0".to_vec()).count(), 4);
    }

    #[test]
    fn as_u32_vec_widens_unsigned_integer_types() {
        assert_eq!(Value::Byte(vec![1, 255]).as_u32_vec(), Some(vec![1, 255]));
        assert_eq!(
            Value::Short(vec![1, 65535]).as_u32_vec(),
            Some(vec![1, 65535])
        );
        assert_eq!(
            Value::Long(vec![1, 70000]).as_u32_vec(),
            Some(vec![1, 70000])
        );
        assert_eq!(Value::SLong(vec![1]).as_u32_vec(), None);
        assert_eq!(Value::Rational(vec![(1, 2)]).as_u32_vec(), None);
    }
}
