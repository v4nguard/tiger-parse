use std::{
    borrow::{Borrow, Cow},
    ops::Deref,
};

#[derive(Debug, Clone)]
pub struct ReflectedStruct {
    pub id: u32,
    pub name: Cow<'static, str>,
    pub fields: Cow<'static, [ReflectedField]>,
    pub is_tuple: bool,
    pub size: usize,
}

impl std::fmt::Display for ReflectedStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "#[tiger_tag(id = 0x{:08X}, size = 0x{:X})]\n",
            self.id, self.size
        ))?;
        f.write_fmt(format_args!("struct {} {{", self.name))?;
        for field in self.fields.iter() {
            f.write_fmt(format_args!(
                "\n    {field}, // size=0x{:X}, offset=0x{:X}",
                field.size, field.offset
            ))?;
        }
        f.write_str("\n}")
    }
}

#[derive(Debug, Clone)]
pub struct ReflectedField {
    pub name: Cow<'static, str>,
    pub size: usize,
    pub offset: usize,
    pub explicit_offset: bool,
    pub ty: ReflectedType,
}

impl std::fmt::Display for ReflectedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.explicit_offset {
            f.write_fmt(format_args!("#[tag(offset = 0x{:X})] ", self.offset))?;
        }
        f.write_fmt(format_args!("{}: {}", self.name, self.ty))
    }
}

#[derive(Debug, Clone)]
pub enum ReflectedType {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,

    Float32,
    Float64,
    Vec2,
    Vec3,
    Vec4,

    TagHash,
    // Struct(u32),
    Padding(usize),

    Tuple(Cow<'static, [ReflectedType]>),
    Array(CowBox<'static, ReflectedType>),
    FixedArray(usize, CowBox<'static, ReflectedType>),
    Other(Cow<'static, str>),
}

impl std::fmt::Display for ReflectedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReflectedType::UInt8 => f.write_str("u8"),
            ReflectedType::UInt16 => f.write_str("u16"),
            ReflectedType::UInt32 => f.write_str("u32"),
            ReflectedType::UInt64 => f.write_str("u64"),
            ReflectedType::Int8 => f.write_str("i8"),
            ReflectedType::Int16 => f.write_str("i16"),
            ReflectedType::Int32 => f.write_str("i32"),
            ReflectedType::Int64 => f.write_str("i64"),
            ReflectedType::Float32 => f.write_str("f32"),
            ReflectedType::Float64 => f.write_str("f64"),
            ReflectedType::Vec2 => f.write_str("Vec2"),
            ReflectedType::Vec3 => f.write_str("Vec3"),
            ReflectedType::Vec4 => f.write_str("Vec4"),
            ReflectedType::TagHash => f.write_str("TagHash"),
            ReflectedType::Padding(size) => f.write_fmt(format_args!("Padding<{size}>")),
            ReflectedType::Tuple(fields) => f.write_fmt(format_args!(
                "({})",
                fields
                    .iter()
                    .map(|field| field.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            ReflectedType::Array(inner_type) => f.write_fmt(format_args!("Vec<{inner_type}>")),
            ReflectedType::FixedArray(size, inner_type) => {
                f.write_fmt(format_args!("[{inner_type}; {size}]"))
            }
            ReflectedType::Other(str) => f.write_str(str),
        }
    }
}

/// Borrowed value or owned Box
pub enum CowBox<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    Borrowed(&'a B),
    Owned(Box<<B as ToOwned>::Owned>),
}

impl<B: ?Sized + ToOwned> Deref for CowBox<'_, B>
where
    B::Owned: Borrow<B>,
{
    type Target = B;

    fn deref(&self) -> &B {
        match *self {
            Self::Borrowed(borrowed) => borrowed,
            Self::Owned(ref owned) => owned.as_ref().borrow(),
        }
    }
}

impl<B: ?Sized + ToOwned + std::fmt::Debug> std::fmt::Debug for CowBox<'_, B>
where
    B::Owned: Borrow<B>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Borrowed(borrowed) => borrowed.fmt(f),
            Self::Owned(ref owned) => owned.as_ref().borrow().fmt(f),
        }
    }
}

impl<B: ?Sized + ToOwned + std::fmt::Display> std::fmt::Display for CowBox<'_, B>
where
    B::Owned: Borrow<B>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Borrowed(borrowed) => borrowed.fmt(f),
            Self::Owned(ref owned) => owned.as_ref().borrow().fmt(f),
        }
    }
}

impl<B: ?Sized + ToOwned> Clone for CowBox<'_, B> {
    fn clone(&self) -> Self {
        match *self {
            Self::Borrowed(b) => Self::Borrowed(b),

            Self::Owned(ref o) => Self::Owned(Box::new(o.as_ref().borrow().to_owned())),
        }
    }
}
