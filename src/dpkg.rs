use std::io::Cursor;

use anyhow::{ensure, Context};
use destiny_pkg::{TagHash, TagHash64};

use crate::TigerReadable;

pub trait PackageManagerExt {
    fn read_tag_struct<T: TigerReadable>(&self, tag: impl Into<TagHash>) -> anyhow::Result<T>;

    fn read_tag64_struct<T: TigerReadable>(&self, hash: impl Into<TagHash64>) -> anyhow::Result<T>;
}

impl PackageManagerExt for destiny_pkg::PackageManager {
    fn read_tag_struct<T: TigerReadable>(&self, tag: impl Into<TagHash>) -> anyhow::Result<T> {
        let tag = tag.into();

        #[cfg(feature = "check_types")]
        if T::ID.is_some() && T::ID != Some(u32::MAX) {
            if let Some(entry) = self.get_entry(tag) {
                let tag_type = entry.reference;
                ensure!(
                    tag_type == T::ID.unwrap(),
                    "Tag type mismatch! Expected 0x{:08X}, got 0x{:08X} (tag {tag})",
                    T::ID.unwrap(),
                    tag_type
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
            .hash64_table
            .get(&hash.0)
            .context("Hash not found")?
            .hash32;

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
