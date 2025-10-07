use crate::{error::Error, Offset, Size, TigerReadable};

impl<T: TigerReadable> TigerReadable for Vec<T> {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> crate::Result<Self> {
        let size = Size::read_ds_endian(reader, endian)? as usize;
        let ptr = reader.stream_position()? + Offset::read_ds_endian(reader, endian)? as u64;
        let save_pos = reader.stream_position()?;

        if size == 0 {
            return Ok(Vec::new());
        }

        reader.seek(std::io::SeekFrom::Start(ptr))?;
        let size_header = Size::read_ds_endian(reader, endian)? as usize;
        if size != size_header {
            return Err(Error::InvalidStructure(format!(
                "Vector size mismatch in {typename} at 0x{save_pos:X} (pointer 0x{ptr:X}). {size} elements in pointer vs {size_header} elements in header.",
                typename = std::any::type_name::<Self>(),
            )));
        }
        #[cfg(feature = "check_types")]
        if T::ID.is_some() && (T::ID != Some(u32::MAX) || cfg!(feature = "check_types_strict")) {
            let element_type = u32::read_ds_endian(reader, endian)?;
            if element_type != T::ID.unwrap() {
                return Err(Error::TypeMismatch(format!(
                    "Element type mismatch! Expected 0x{:08X}, got 0x{:08X} (array @ 0x{ptr:X})",
                    T::ID.unwrap(),
                    element_type
                )));
            }
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
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(T::read_ds_endian(reader, endian)?);
        }

        reader.seek(std::io::SeekFrom::Start(save_pos))?;

        Ok(data)
    }

    // TODO(cohae): multiversion struct ids
    const ID: Option<u32> = None;
    const SIZE: usize = std::mem::size_of::<(Size, Offset)>();
}
