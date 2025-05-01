use std::{
    borrow::{Borrow, Cow},
    ops::Deref,
};

#[derive(Debug, Clone)]
pub struct ReflectedStruct {
    pub id: u32,
    pub name: Cow<'static, str>,
    pub fields: Cow<'static, [ReflectedField]>,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct ReflectedField {
    pub name: Cow<'static, str>,
    pub size: usize,
    pub offset: usize,
    pub ty: ReflectedType,
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

impl<B: ?Sized + ToOwned> Clone for CowBox<'_, B> {
    fn clone(&self) -> Self {
        match *self {
            Self::Borrowed(b) => Self::Borrowed(b),

            Self::Owned(ref o) => Self::Owned(Box::new(o.as_ref().borrow().to_owned())),
        }
    }
}
