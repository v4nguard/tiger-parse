use std::io::Cursor;

use tiger_pkg::{TagHash, TagHash64, Version};

use crate::{error::Error, TigerReadable};

pub trait PackageManagerExt {
    fn read_tag_struct<T: TigerReadable>(&self, tag: impl Into<TagHash>) -> crate::Result<T>;

    fn read_tag64_struct<T: TigerReadable>(&self, hash: impl Into<TagHash64>) -> crate::Result<T>;

    fn read_named_tag_struct<T: TigerReadable>(
        &self,
        tag_name: impl AsRef<str>,
    ) -> crate::Result<T>;
}

impl PackageManagerExt for tiger_pkg::PackageManager {
    fn read_tag_struct<T: TigerReadable>(&self, tag: impl Into<TagHash>) -> crate::Result<T> {
        let tag = tag.into();

        #[cfg(feature = "check_types")]
        if T::ID.is_some() && (T::ID != Some(u32::MAX) || cfg!(feature = "check_types_strict")) {
            if let Some(entry) = self.get_entry(tag) {
                let tag_type = entry.reference;
                if tag_type != T::ID.unwrap() {
                    return Err(Error::TypeMismatch(format!(
                        "Tag type mismatch! Expected 0x{:08X}, got 0x{:08X} (tag {tag}) (type {})",
                        T::ID.unwrap(),
                        tag_type,
                        std::any::type_name::<T>()
                    )));
                }
            }
        }

        #[cfg(feature = "check_types")]
        if let Some((etype, esubtype)) = T::ETYPE {
            if let Some(entry) = self.get_entry(tag) {
                if etype != entry.file_type {
                    return Err(Error::TypeMismatch(format!(
                        "Tag type mismatch! Expected {}:{}, got {}:{} (tag {tag}) (type {})",
                        etype,
                        if let Some(subtype) = esubtype {
                            subtype.to_string()
                        } else {
                            "ANY".to_string()
                        },
                        entry.file_type,
                        entry.file_subtype,
                        std::any::type_name::<T>()
                    )));
                }
            }
        }

        let data = self
            .read_tag(tag)
            .map_err(|e| Error::TagReadFailed(e.to_string()))?;
        let mut cursor = Cursor::new(&data);
        T::read_ds_endian(&mut cursor, self.version.endian().into())
    }

    fn read_tag64_struct<T: TigerReadable>(&self, hash: impl Into<TagHash64>) -> crate::Result<T> {
        let hash = hash.into();
        let tag = self
            .lookup
            .tag64_entries
            .get(&hash.0)
            .ok_or(Error::Hash64LookupFailed(hash))?
            .hash32;

        self.read_tag_struct(tag)
    }

    fn read_named_tag_struct<T: TigerReadable>(
        &self,
        tag_name: impl AsRef<str>,
    ) -> crate::Result<T> {
        let tag = self
            .get_named_tag(
                tag_name.as_ref(),
                T::ID.ok_or_else(|| {
                    Error::TypeMismatch(format!(
                        "Type '{}' does not have a tag ID set",
                        std::any::type_name::<T>()
                    ))
                })?,
            )
            .ok_or_else(|| {
                Error::TypeMismatch(format!(
                    "Tag '{}' with ID 0x{:X} not found",
                    tag_name.as_ref(),
                    T::ID.unwrap()
                ))
            })?;

        self.read_tag_struct(tag)
    }
}

impl From<tiger_pkg::Endian> for crate::Endian {
    fn from(endian: tiger_pkg::Endian) -> Self {
        match endian {
            tiger_pkg::Endian::Big => crate::Endian::Big,
            tiger_pkg::Endian::Little => crate::Endian::Little,
        }
    }
}

impl TigerReadable for TagHash {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> crate::Result<Self> {
        Ok(TagHash(u32::read_ds_endian(reader, endian)?))
    }

    
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for TagHash64 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> crate::Result<Self> {
        Ok(TagHash64(u64::read_ds_endian(reader, endian)?))
    }

    
    const SIZE: usize = std::mem::size_of::<Self>();
}
