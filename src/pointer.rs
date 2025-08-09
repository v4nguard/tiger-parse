use std::{
    fmt::{Debug, Formatter},
    io::SeekFrom,
    ops::Deref,
};

use crate::{Offset, TigerReadable};

pub struct Pointer<T: TigerReadable>(pub T, Offset);

impl<T: TigerReadable> TigerReadable for Pointer<T> {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> crate::Result<Self> {
        let ptr = reader.stream_position()? as i64 + Offset::read_ds_endian(reader, endian)? as i64;
        let save_pos = reader.stream_position()?;

        reader.seek(std::io::SeekFrom::Start(ptr as u64))?;

        let data = T::read_ds_endian(reader, endian)?;

        reader.seek(std::io::SeekFrom::Start(save_pos))?;

        Ok(Pointer(data, ptr as Offset))
    }

    const ZEROCOPY: bool = false;

    const ID: Option<u32> = None;
    const SIZE: usize = std::mem::size_of::<Offset>();
}

impl<T: TigerReadable> Pointer<T> {
    pub fn offset(&self) -> Offset {
        self.1
    }
}

impl<T: TigerReadable> Deref for Pointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: TigerReadable + Debug> Debug for Pointer<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pointer")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

impl<T: TigerReadable + Clone> Clone for Pointer<T> {
    fn clone(&self) -> Self {
        Pointer(self.0.clone(), self.1)
    }
}

pub struct PointerOptional<T: TigerReadable>(pub Option<T>, Offset);

impl<T: TigerReadable> TigerReadable for PointerOptional<T> {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> crate::Result<Self> {
        let ptr_pos = reader.stream_position()? as i64;
        let ptr_data = Offset::read_ds_endian(reader, endian)?;
        if ptr_data == 0 {
            return Ok(PointerOptional(None, ptr_pos as Offset));
        }

        let ptr = ptr_pos + ptr_data as i64;
        let save_pos = reader.stream_position()?;

        reader.seek(std::io::SeekFrom::Start(ptr as u64))?;

        let data = T::read_ds_endian(reader, endian)?;

        reader.seek(std::io::SeekFrom::Start(save_pos))?;

        Ok(PointerOptional(Some(data), ptr_pos as Offset))
    }

    const ZEROCOPY: bool = false;

    const ID: Option<u32> = None;
    const SIZE: usize = std::mem::size_of::<Offset>();
}

impl<T: TigerReadable> PointerOptional<T> {
    pub fn offset(&self) -> Option<Offset> {
        self.0.as_ref().map(|_| self.1)
    }
}

impl<T: TigerReadable> Deref for PointerOptional<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: TigerReadable + Debug> Debug for PointerOptional<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PointerOptional")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct ResourcePointer {
    pub offset: u64,
    pub resource_type: u32,
    pub is_valid: bool,
}

impl TigerReadable for ResourcePointer {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> crate::Result<Self> {
        let offset_base = reader.stream_position()?;
        let offset: Offset = TigerReadable::read_ds_endian(reader, endian)?;
        if offset == 0 || offset == Offset::MAX {
            return Ok(ResourcePointer {
                offset: 0,
                resource_type: u32::MAX,
                is_valid: false,
            });
        }

        let offset_save = reader.stream_position()?;

        reader.seek(SeekFrom::Start(offset_base))?;
        reader.seek(SeekFrom::Current(offset as i64 - 4))?;
        let resource_type: u32 = TigerReadable::read_ds_endian(reader, endian)?;

        reader.seek(SeekFrom::Start(offset_save))?;

        Ok(ResourcePointer {
            offset: offset_base.saturating_add_signed(offset as i64),
            resource_type,
            is_valid: true,
        })
    }

    const ID: Option<u32> = None;
    const ZEROCOPY: bool = false;
    const SIZE: usize = 8;
}

impl Debug for ResourcePointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "ResourcePointer(type=0x{:08x})",
            self.resource_type
        ))
    }
}

#[cfg(feature = "tiger_pkg")]
#[derive(Clone, Copy)]
pub struct ResourcePointerWithClass {
    pub offset: u64,
    pub is_valid: bool,

    pub resource_type: u32,
    /// Usually just the current tag
    pub parent_tag: tiger_pkg::TagHash,
    pub class_type: u32,
}

#[cfg(feature = "tiger_pkg")]
impl TigerReadable for ResourcePointerWithClass {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> crate::Result<Self> {
        let offset_base = reader.stream_position()?;
        let offset: Offset = TigerReadable::read_ds_endian(reader, endian)?;
        if offset == 0 || offset == Offset::MAX {
            return Ok(ResourcePointerWithClass {
                offset: 0,
                is_valid: false,
                resource_type: u32::MAX,
                parent_tag: tiger_pkg::TagHash::NONE,
                class_type: u32::MAX,
            });
        }

        let offset_save = reader.stream_position()?;

        reader.seek(SeekFrom::Start(offset_base))?;
        reader.seek(SeekFrom::Current(offset as i64 - 4))?;
        let resource_type: u32 = TigerReadable::read_ds_endian(reader, endian)?;
        let parent_tag: tiger_pkg::TagHash = TigerReadable::read_ds_endian(reader, endian)?;
        let class_type: u32 = TigerReadable::read_ds_endian(reader, endian)?;

        let true_offset = reader.stream_position()?;
        reader.seek(SeekFrom::Start(offset_save))?;

        Ok(ResourcePointerWithClass {
            offset: true_offset,
            is_valid: true,
            resource_type,
            parent_tag,
            class_type,
        })
    }

    const ZEROCOPY: bool = false;
    const ID: Option<u32> = None;
    const SIZE: usize = 8;
}

#[cfg(feature = "tiger_pkg")]
impl Debug for ResourcePointerWithClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "ResourcePointer(type=0x{:08X}, parent={}, class={:08X})",
            self.resource_type,
            self.parent_tag,
            self.class_type.to_be()
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek};

    use crate::{Pointer, TigerReadable};

    #[test]
    fn test_pointer() {
        #[cfg(not(feature = "32bit"))]
        let data: [u8; 0x28] = [
            0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xEF, 0xBE, 0xDA, 0xED, 0xFE, 0x00, 0x00, 0x00,
        ];

        #[cfg(feature = "32bit")]
        let data: [u8; 0x28] = [
            0x20, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xEF, 0xBE, 0xDA, 0xED, 0xFE, 0x00, 0x00, 0x00,
        ];

        let mut cursor = Cursor::new(&data);
        let ptr: Pointer<u64> =
            TigerReadable::read_ds_endian(&mut cursor, crate::Endian::Little).unwrap();

        println!("{:X}", *ptr);
        assert_eq!(*ptr, 0xfeed_da_beef)
    }

    #[test]
    fn test_backwards_pointer() {
        let data: [u8; 0x28] = [
            0xef, 0xbe, 0xda, 0xed, 0xfe, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xe0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        let mut cursor = Cursor::new(&data);
        cursor.seek(std::io::SeekFrom::Start(0x20)).unwrap();
        let ptr: Pointer<u64> =
            TigerReadable::read_ds_endian(&mut cursor, crate::Endian::Little).unwrap();

        println!("{:X}", *ptr);
        assert_eq!(*ptr, 0xfeed_da_beef)
    }
}
