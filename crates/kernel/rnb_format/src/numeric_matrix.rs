use std::io::{Read, Write};

pub const NUM_MATRIX_MAGIC: [u8; 4] = *b"NMX\0";
pub const NUM_MATRIX_VERSION_MAJOR: u16 = 0;
pub const NUM_MATRIX_VERSION_MINOR: u16 = 1;

/// Numeric element type for matrices.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericType {
    F32 = 1,
}

impl NumericType {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            1 => Some(NumericType::F32),
            _ => None,
        }
    }

    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Simple dense row-major numeric matrix.
///
/// Intended as a generic payload for things like expression matrices.
#[derive(Debug, Clone, PartialEq)]
pub struct NumericMatrix {
    pub rows: u32,
    pub cols: u32,
    pub elem_type: NumericType,
    pub values: Vec<f32>, // row-major, len == rows * cols
}

impl NumericMatrix {
    pub fn new(rows: u32, cols: u32, values: Vec<f32>) -> std::io::Result<Self> {
        let expected = (rows as usize)
            .checked_mul(cols as usize)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "numeric matrix dimensions overflow",
                )
            })?;
        if values.len() != expected {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "numeric matrix values length mismatch",
            ));
        }
        Ok(Self {
            rows,
            cols,
            elem_type: NumericType::F32,
            values,
        })
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&NUM_MATRIX_MAGIC)?;
        w.write_all(&NUM_MATRIX_VERSION_MAJOR.to_le_bytes())?;
        w.write_all(&NUM_MATRIX_VERSION_MINOR.to_le_bytes())?;

        w.write_all(&self.rows.to_le_bytes())?;
        w.write_all(&self.cols.to_le_bytes())?;
        w.write_all(&self.elem_type.as_u32().to_le_bytes())?;

        // reserved
        w.write_all(&0u32.to_le_bytes())?;

        // Currently only F32 is supported.
        if self.elem_type != NumericType::F32 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported numeric matrix elem_type",
            ));
        }

        for v in &self.values {
            w.write_all(&v.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != NUM_MATRIX_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid numeric matrix magic",
            ));
        }

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let vmaj = u16::from_le_bytes(buf2);
        r.read_exact(&mut buf2)?;
        let vmin = u16::from_le_bytes(buf2);

        if vmaj != NUM_MATRIX_VERSION_MAJOR || vmin != NUM_MATRIX_VERSION_MINOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported numeric matrix version",
            ));
        }

        let mut buf4 = [0u8; 4];
        r.read_exact(&mut buf4)?;
        let rows = u32::from_le_bytes(buf4);

        r.read_exact(&mut buf4)?;
        let cols = u32::from_le_bytes(buf4);

        r.read_exact(&mut buf4)?;
        let elem_raw = u32::from_le_bytes(buf4);
        let elem_type = NumericType::from_u32(elem_raw).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unknown numeric matrix elem_type",
            )
        })?;

        r.read_exact(&mut buf4)?;
        let reserved = u32::from_le_bytes(buf4);
        if reserved != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "numeric matrix reserved field must be 0",
            ));
        }

        let count = (rows as usize)
            .checked_mul(cols as usize)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "numeric matrix dimensions overflow",
                )
            })?;

        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            r.read_exact(&mut buf4)?;
            let v = f32::from_le_bytes(buf4);
            values.push(v);
        }

        Ok(Self {
            rows,
            cols,
            elem_type,
            values,
        })
    }
}

