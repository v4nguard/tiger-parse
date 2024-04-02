use core::slice;
use std::mem;

use anyhow::ensure;

use crate::{Offset, Size, TigerReadable};

impl<T: TigerReadable> TigerReadable for Vec<T> {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let size = Size::read_ds_endian(reader, endian)? as usize;
        let ptr = reader.stream_position()? + Offset::read_ds_endian(reader, endian)? as u64;
        let save_pos = reader.stream_position()?;

        if size == 0 {
            return Ok(Vec::new());
        }

        reader.seek(std::io::SeekFrom::Start(ptr))?;
        let size_header = Size::read_ds_endian(reader, endian)? as usize;
        assert_eq!(
            size,
            size_header,
            "Size mismatch in {}",
            std::any::type_name::<Self>()
        );
        #[cfg(feature = "check_types")]
        if T::ID.is_some() && (T::ID != Some(u32::MAX) || cfg!(feature = "check_types_strict")) {
            let element_type = u32::read_ds_endian(reader, endian)?;
            ensure!(
                element_type == T::ID.unwrap(),
                "Element type mismatch! Expected 0x{:08X}, got 0x{:08X} (array @ 0x{ptr:X})",
                T::ID.unwrap(),
                element_type
            );
        }

        #[cfg(feature = "check_types_debug")]
        if T::ID == Some(u32::MAX) {
            reader.seek(std::io::SeekFrom::Start(
                ptr + std::mem::size_of::<Size>() as u64,
            ))?;
            let element_type = u32::read_ds_endian(reader, endian)?;
            tracing::warn!(
                "Rust tag has no ID, please set one. Data tag type ID is 0x{element_type:08X} for Rust type {} (0x{:08X})",
                std::any::type_name::<T>(), T::ID.unwrap_or(u32::MAX)
            );
        }

        reader.seek(std::io::SeekFrom::Start(ptr + 16))?;
        let data = if T::ZEROCOPY && endian == crate::Endian::Little {
            let mut data = Vec::with_capacity(size);

            let size_bytes = size * mem::size_of::<T>();
            // Safety: the necessary space is available in the vector
            unsafe {
                let data_ptr = data.as_mut_ptr();
                reader.read_exact(slice::from_raw_parts_mut(data_ptr as *mut u8, size_bytes))?;
                data.set_len(size);
            }

            data
        } else {
            let mut data = Vec::with_capacity(size);
            for _ in 0..size {
                data.push(T::read_ds_endian(reader, endian)?);
            }

            data
        };

        reader.seek(std::io::SeekFrom::Start(save_pos))?;

        Ok(data)
    }

    const ZEROCOPY: bool = false;

    // TODO(cohae): multiversion struct ids
    const ID: Option<u32> = None;
    const SIZE: usize = std::mem::size_of::<(Size, Offset)>();
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::TigerReadable;

    pub type TestType = (u32, f32, i64);

    #[test]
    fn test_vec_zerocopy() {
        assert_eq!(TestType::ZEROCOPY, true);

        const DATA: [u8; 112] = [
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xB8, 0x9F, 0x80, 0x80, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x7B, 0x00, 0x00, 0x00, 0x00, 0x40, 0x9A, 0x44,
            0xD4, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xEF, 0xBE, 0xAD, 0xDE, 0x00, 0xFF,
            0x7F, 0x47, 0x64, 0x05, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCE, 0xFA, 0xED, 0xFE,
            0x00, 0x20, 0xA7, 0x44, 0xEF, 0xBE, 0x0D, 0xF0, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0x00, 0x00, 0x90, 0x4F, 0xC3, 0x47, 0xEF, 0xBE, 0x37, 0x13, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut cursor = Cursor::new(&DATA);
        let vec: Vec<TestType> =
            TigerReadable::read_ds_endian(&mut cursor, crate::Endian::Little).unwrap();

        assert_eq!(vec.len(), 4);

        assert_eq!(
            &vec,
            &[
                (123, 1234.0, -300,),
                (0xDEADBEEF, 65535.0, 132452,),
                (0xFEEDFACE, 1337.0, 0xF00DBEEF,),
                (0xFFFF, 99999.125, 0x1337BEEF,),
            ]
        )
    }
}
