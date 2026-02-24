use std::io::{Read, Write};

use crate::numeric_matrix::NumericType;

pub const SPARSE_MATRIX_MAGIC: [u8; 4] = *b"SMX\0";
pub const SPARSE_MATRIX_VERSION_MAJOR: u16 = 0;
pub const SPARSE_MATRIX_VERSION_MINOR: u16 = 1;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparseStorage {
    Csr = 1,
    Csc = 2,
}

impl SparseStorage {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            1 => Some(SparseStorage::Csr),
            2 => Some(SparseStorage::Csc),
            _ => None,
        }
    }

    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Simple CSR/CSC sparse numeric matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct SparseMatrix {
    pub rows: u32,
    pub cols: u32,
    pub elem_type: NumericType,
    pub storage: SparseStorage,
    pub row_axis_object_type_id: Option<u32>,
    pub col_axis_object_type_id: Option<u32>,
    pub semantic_tag_sid: Option<u32>,
    pub indptr: Vec<u32>,
    pub indices: Vec<u32>,
    pub data: Vec<f32>,
}

impl SparseMatrix {
    pub fn new(
        rows: u32,
        cols: u32,
        elem_type: NumericType,
        storage: SparseStorage,
        row_axis_object_type_id: Option<u32>,
        col_axis_object_type_id: Option<u32>,
        semantic_tag_sid: Option<u32>,
        indptr: Vec<u32>,
        indices: Vec<u32>,
        data: Vec<f32>,
    ) -> std::io::Result<Self> {
        let expected_nnz = indices.len();
        if data.len() != expected_nnz {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "sparse matrix indices/data length mismatch",
            ));
        }

        let expected_ptr_len = match storage {
            SparseStorage::Csr => rows as usize + 1,
            SparseStorage::Csc => cols as usize + 1,
        };
        if indptr.len() != expected_ptr_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "sparse matrix indptr length mismatch",
            ));
        }

        if let Some(&last) = indptr.last() {
            if last as usize != expected_nnz {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "sparse matrix indptr last != nnz",
                ));
            }
        }

        // Monotonicity and bounds checks.
        let mut prev = 0u32;
        for &p in &indptr {
            if p < prev {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "sparse matrix indptr must be non-decreasing",
                ));
            }
            if p as usize > expected_nnz {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "sparse matrix indptr entry out of range",
                ));
            }
            prev = p;
        }

        let dim = match storage {
            SparseStorage::Csr => cols,
            SparseStorage::Csc => rows,
        } as u32;
        for &idx in &indices {
            if idx >= dim {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "sparse matrix index out of bounds",
                ));
            }
        }

        Ok(Self {
            rows,
            cols,
            elem_type,
            storage,
            row_axis_object_type_id,
            col_axis_object_type_id,
            semantic_tag_sid,
            indptr,
            indices,
            data,
        })
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&SPARSE_MATRIX_MAGIC)?;
        w.write_all(&SPARSE_MATRIX_VERSION_MAJOR.to_le_bytes())?;
        w.write_all(&SPARSE_MATRIX_VERSION_MINOR.to_le_bytes())?;

        w.write_all(&self.rows.to_le_bytes())?;
        w.write_all(&self.cols.to_le_bytes())?;
        w.write_all(&self.elem_type.as_u32().to_le_bytes())?;
        w.write_all(&self.storage.as_u32().to_le_bytes())?;

        fn write_opt_u32<W: Write>(w: &mut W, v: Option<u32>) -> std::io::Result<()> {
            let raw = v.unwrap_or(u32::MAX);
            w.write_all(&raw.to_le_bytes())
        }

        write_opt_u32(&mut w, self.row_axis_object_type_id)?;
        write_opt_u32(&mut w, self.col_axis_object_type_id)?;
        write_opt_u32(&mut w, self.semantic_tag_sid)?;

        let nnz: u32 = self
            .indices
            .len()
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "nnz too large"))?;
        let indptr_len: u32 = self
            .indptr
            .len()
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "indptr too large"))?;

        w.write_all(&nnz.to_le_bytes())?;
        w.write_all(&indptr_len.to_le_bytes())?;
        w.write_all(&0u32.to_le_bytes())?; // reserved

        for v in &self.indptr {
            w.write_all(&v.to_le_bytes())?;
        }
        for v in &self.indices {
            w.write_all(&v.to_le_bytes())?;
        }
        // Currently only F32 is supported; the enum exists to allow future extension.
        if self.elem_type != NumericType::F32 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported sparse matrix elem_type",
            ));
        }
        for v in &self.data {
            w.write_all(&v.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != SPARSE_MATRIX_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid sparse matrix magic",
            ));
        }

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let vmaj = u16::from_le_bytes(buf2);
        r.read_exact(&mut buf2)?;
        let vmin = u16::from_le_bytes(buf2);
        if vmaj != SPARSE_MATRIX_VERSION_MAJOR || vmin != SPARSE_MATRIX_VERSION_MINOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported sparse matrix version",
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
                "unknown sparse matrix elem_type",
            )
        })?;

        r.read_exact(&mut buf4)?;
        let storage_raw = u32::from_le_bytes(buf4);
        let storage = SparseStorage::from_u32(storage_raw).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unknown sparse matrix storage",
            )
        })?;

        fn read_opt_u32<R: Read>(r: &mut R, buf: &mut [u8; 4]) -> std::io::Result<Option<u32>> {
            r.read_exact(buf)?;
            let raw = u32::from_le_bytes(*buf);
            if raw == u32::MAX {
                Ok(None)
            } else {
                Ok(Some(raw))
            }
        }

        let row_axis_object_type_id = read_opt_u32(&mut r, &mut buf4)?;
        let col_axis_object_type_id = read_opt_u32(&mut r, &mut buf4)?;
        let semantic_tag_sid = read_opt_u32(&mut r, &mut buf4)?;

        r.read_exact(&mut buf4)?;
        let nnz = u32::from_le_bytes(buf4) as usize;
        r.read_exact(&mut buf4)?;
        let indptr_len = u32::from_le_bytes(buf4) as usize;

        r.read_exact(&mut buf4)?;
        let reserved = u32::from_le_bytes(buf4);
        if reserved != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "sparse matrix reserved field must be 0",
            ));
        }

        let mut indptr = vec![0u32; indptr_len];
        for v in &mut indptr {
            r.read_exact(&mut buf4)?;
            *v = u32::from_le_bytes(buf4);
        }

        let mut indices = vec![0u32; nnz];
        for v in &mut indices {
            r.read_exact(&mut buf4)?;
            *v = u32::from_le_bytes(buf4);
        }

        let mut data = Vec::with_capacity(nnz);
        for _ in 0..nnz {
            r.read_exact(&mut buf4)?;
            let v = f32::from_le_bytes(buf4);
            data.push(v);
        }

        SparseMatrix::new(
            rows,
            cols,
            elem_type,
            storage,
            row_axis_object_type_id,
            col_axis_object_type_id,
            semantic_tag_sid,
            indptr,
            indices,
            data,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_sparse_matrix_csr() {
        // 3x3 matrix with nonzeros at (0,1)=1.0 and (2,2)=2.0
        let rows = 3;
        let cols = 3;
        let indptr = vec![0, 1, 1, 2]; // row 0: 0..1, row 1: 1..1, row 2: 1..2
        let indices = vec![1, 2];
        let data = vec![1.0f32, 2.0f32];

        let m = SparseMatrix::new(
            rows,
            cols,
            NumericType::F32,
            SparseStorage::Csr,
            Some(1),
            Some(2),
            Some(3),
            indptr.clone(),
            indices.clone(),
            data.clone(),
        )
        .unwrap();

        let mut buf = Vec::new();
        m.write_to(&mut buf).unwrap();
        let decoded = SparseMatrix::read_from(&buf[..]).unwrap();

        assert_eq!(decoded.rows, rows);
        assert_eq!(decoded.cols, cols);
        assert_eq!(decoded.storage, SparseStorage::Csr);
        assert_eq!(decoded.elem_type, NumericType::F32);
        assert_eq!(decoded.indptr, indptr);
        assert_eq!(decoded.indices, indices);
        assert_eq!(decoded.data, data);
        assert_eq!(decoded.row_axis_object_type_id, Some(1));
        assert_eq!(decoded.col_axis_object_type_id, Some(2));
        assert_eq!(decoded.semantic_tag_sid, Some(3));
    }
}

