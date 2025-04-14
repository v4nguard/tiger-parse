pub use anyhow::Result;
pub use pointer::{Pointer, PointerOptional, ResourcePointer};

#[cfg(feature = "tiger_pkg")]
pub use pointer::ResourcePointerWithClass;

pub use padding::Padding;
pub use string::NullString;
pub use tiger_parse_derive::tiger_tag;
pub use variant::{OptionalVariantPointer, VariantEnum, VariantPointer};

pub type FnvHash = u32;

mod array;
mod glam;
mod padding;
mod tuples;
mod variant;
mod vector;

pub mod error;
pub mod pointer;
pub mod string;

#[cfg(feature = "tiger_pkg")]
pub mod dpkg;

#[cfg(feature = "tiger_pkg")]
pub use dpkg::PackageManagerExt;

use std::io::{Read, Seek};

#[cfg(feature = "32bit")]
type Offset = u32;

#[cfg(feature = "32bit")]
type Size = u32;

#[cfg(not(feature = "32bit"))]
type Offset = u64;

#[cfg(not(feature = "32bit"))]
type Size = u64;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Endian {
    Little,
    Big,
}

pub trait TigerReadable: Sized {
    // TODO(cohae): Destiny reader
    fn read_ds<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Self::read_ds_endian(reader, Endian::Little)
    }

    fn read_ds_endian<R: Read + Seek>(reader: &mut R, endian: Endian) -> Result<Self>;

    const ZEROCOPY: bool = false;

    /// 0x8080XXXX structure ID
    const ID: Option<u32> = None;

    const ETYPE: Option<(u8, Option<u8>)> = None;

    /// Total size of this struct, in bytes
    const SIZE: usize;
}

macro_rules! impl_read_primitives {
    ($($typ:ty : $size:expr),+) => {
        $(
            impl TigerReadable for $typ {
                fn read_ds_endian<R: ::std::io::Read + ::std::io::Seek>(reader: &mut R, endian: Endian) -> Result<Self> {
                    let mut bytes = [0u8; $size];
                    reader.read_exact(&mut bytes)?;
                    Ok(match endian {
                        Endian::Little => <$typ>::from_le_bytes(bytes),
                        Endian::Big => <$typ>::from_be_bytes(bytes),
                    })
                }

                const ZEROCOPY: bool = true;

                const SIZE: usize = $size;
            }
        )*
    };
}

impl_read_primitives! {
    u8:1,
    u16:2,
    u32:4,
    u64:8,
    u128:16,

    i8:1,
    i16:2,
    i32:4,
    i64:8,
    i128:16,

    f32:4,
    f64:8
}

impl TigerReadable for () {
    fn read_ds_endian<R: Read + Seek>(_reader: &mut R, _endian: crate::Endian) -> Result<Self> {
        Ok(())
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = 0;
}

impl TigerReadable for bool {
    fn read_ds_endian<R: Read + Seek>(reader: &mut R, endian: crate::Endian) -> Result<Self> {
        Ok(u8::read_ds_endian(reader, endian)? != 0)
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = 1;
}
