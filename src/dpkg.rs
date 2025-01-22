use std::io::Cursor;

use anyhow::{ensure, Context};
use destiny_pkg::{TagHash, TagHash64};

use crate::TigerReadable;

pub trait PackageManagerExt {
    fn read_tag_struct<T: TigerReadable>(&self, tag: impl Into<TagHash>) -> anyhow::Result<T>;

    fn read_tag64_struct<T: TigerReadable>(&self, hash: impl Into<TagHash64>) -> anyhow::Result<T>;

    fn read_named_tag_struct<T: TigerReadable>(
        &self,
        tag_name: impl AsRef<str>,
    ) -> anyhow::Result<T>;
}

impl PackageManagerExt for destiny_pkg::PackageManager {
    fn read_tag_struct<T: TigerReadable>(&self, tag: impl Into<TagHash>) -> anyhow::Result<T> {
        let tag = tag.into();

        #[cfg(feature = "check_types")]
        if T::ID.is_some() && (T::ID != Some(u32::MAX) || cfg!(feature = "check_types_strict")) {
            if let Some(entry) = self.get_entry(tag) {
                let tag_type = entry.reference;
                ensure!(
                    tag_type == T::ID.unwrap(),
                    "Tag type mismatch! Expected 0x{:08X}, got 0x{:08X} (tag {tag}) (type {})",
                    T::ID.unwrap(),
                    tag_type,
                    std::any::type_name::<T>()
                );
            }
        }

        #[cfg(feature = "check_types")]
        if let Some((etype, esubtype)) = T::ETYPE {
            if let Some(entry) = self.get_entry(tag) {
                ensure!(
                    etype == entry.file_type
                        && (esubtype.is_none() || Some(entry.file_subtype) == esubtype),
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
                );
            }
        }

        let data = self.read_tag(tag)?;
        let mut cursor = Cursor::new(&data);
        T::read_ds_endian(&mut cursor, self.version.endian().into())
    }

    fn read_tag64_struct<T: TigerReadable>(&self, hash: impl Into<TagHash64>) -> anyhow::Result<T> {
        let hash = hash.into();
        let tag = self
            .lookup
            .tag64_entries
            .get(&hash.0)
            .context("Hash not found")?
            .hash32;

        self.read_tag_struct(tag)
    }

    fn read_named_tag_struct<T: TigerReadable>(
        &self,
        tag_name: impl AsRef<str>,
    ) -> anyhow::Result<T> {
        let tag = self
            .get_named_tag(
                tag_name.as_ref(),
                T::ID.with_context(|| {
                    format!(
                        "Type '{}' does not have a tag ID set",
                        std::any::type_name::<T>()
                    )
                })?,
            )
            .with_context(|| {
                format!(
                    "Tag '{}' with ID 0x{:X} not found",
                    tag_name.as_ref(),
                    T::ID.unwrap()
                )
            })?;

        self.read_tag_struct(tag)
    }
}

impl From<destiny_pkg::Endian> for crate::Endian {
    fn from(endian: destiny_pkg::Endian) -> Self {
        match endian {
            destiny_pkg::Endian::Big => crate::Endian::Big,
            destiny_pkg::Endian::Little => crate::Endian::Little,
        }
    }
}

impl TigerReadable for TagHash {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        Ok(TagHash(u32::read_ds_endian(reader, endian)?))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for TagHash64 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        Ok(TagHash64(u64::read_ds_endian(reader, endian)?))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}
