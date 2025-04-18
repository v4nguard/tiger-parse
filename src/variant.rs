use std::io::SeekFrom;

use anyhow::Context;

use crate::TigerReadable;

pub trait VariantEnum: Sized {
    fn read_variant_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
        class: u32,
    ) -> anyhow::Result<Self>;
}

#[macro_export]
macro_rules! tiger_variant_enum {
    ($(#[derive($($derive:ident),+)])? $([Unknown($enable_unknown:expr)])? enum $enum_name:ident { $($variant:ident),* }) => {
        $(
            #[derive($($derive),*)]
        )?
        pub enum $enum_name {
            $(
                $variant(Box<$variant>),
            )*

            $(
                #[doc = concat!("Generated by [Unknown(", stringify!($enable_unknown), ")]")]
                Unknown {
                    class: u32,
                    offset: u64,
                },
            )*
        }

        #[allow(non_snake_case, non_upper_case_globals)]
        impl VariantEnum for $enum_name {
            fn read_variant_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
                reader: &mut R,
                endian: tiger_parse::Endian,
                class: u32,
            ) -> anyhow::Result<Self> {
                use tiger_parse::TigerReadable;
                paste::paste! {
                    $(
                        const [<$variant _ID>] : u32 = $variant::ID.expect("Missing class ID");
                    )*
                    match class {
                        $(
                            [<$variant _ID>] => Ok(Self::$variant(Box::new(TigerReadable::read_ds_endian(reader, endian)?))),
                        )*

                        $(
                            _ if $enable_unknown => {
                                Ok(Self::Unknown {
                                    class,
                                    offset: reader.stream_position()?,
                                })
                                // let mut buffer = [0u8; 4];
                                // reader.read_exact(&mut buffer)?;
                                // anyhow::bail!("Unknown variant class 0x{:X} for variant enum {}", class, std::any::type_name::<Self>());
                            }
                        )*
                        #[allow(dead_code)]
                        u => anyhow::bail!(
                            "Unknown variant class 0x{u:X} for variant enum {}",
                            std::any::type_name::<Self>()
                        ),
                    }
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct OptionalVariantPointer<T: VariantEnum + Sized>(Option<T>);

impl<T: VariantEnum + Sized> TigerReadable for OptionalVariantPointer<T> {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let offset_base = reader.stream_position()?;
        let offset: i64 = TigerReadable::read_ds_endian(reader, endian)?;
        if offset == 0 || offset == i64::MAX {
            return Ok(Self(None));
        }

        let offset_save = reader.stream_position()?;

        reader.seek(SeekFrom::Start(offset_base))?;
        reader.seek(SeekFrom::Current(offset - 4))?;
        let resource_type: u32 = TigerReadable::read_ds_endian(reader, endian)?;
        reader.seek(SeekFrom::Start(offset_base))?;
        reader.seek(SeekFrom::Current(offset + 0x10))?;
        let data = T::read_variant_endian(reader, endian, resource_type)?;

        reader.seek(SeekFrom::Start(offset_save))?;

        Ok(Self(Some(data)))
    }

    const ID: Option<u32> = None;
    const ZEROCOPY: bool = false;
    const SIZE: usize = 8;
}

impl<T: VariantEnum + Sized> std::ops::Deref for OptionalVariantPointer<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct VariantPointer<T: VariantEnum + Sized>(T);

impl<T: VariantEnum + Sized> TigerReadable for VariantPointer<T> {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let inner: OptionalVariantPointer<T> = TigerReadable::read_ds_endian(reader, endian)?;
        Ok(Self(inner.0.context(
            "Variant pointer is null (might be an OptionalVariantPointer)",
        )?))
    }

    const ID: Option<u32> = None;
    const ZEROCOPY: bool = false;
    const SIZE: usize = 8;
}

impl<T: VariantEnum + Sized> std::ops::Deref for VariantPointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
