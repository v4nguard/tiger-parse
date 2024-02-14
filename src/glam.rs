use crate::TigerReadable;

impl TigerReadable for glam::Vec2 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data: [f32; 2] = <_>::read_ds_endian(reader, endian)?;

        Ok(glam::Vec2::from_array(data))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for glam::Vec3 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data: [f32; 3] = <_>::read_ds_endian(reader, endian)?;

        Ok(glam::Vec3::from_array(data))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for glam::Vec4 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data: [f32; 4] = <_>::read_ds_endian(reader, endian)?;

        Ok(glam::Vec4::from_array(data))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for glam::IVec2 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data: [i32; 2] = <_>::read_ds_endian(reader, endian)?;

        Ok(glam::IVec2::from_array(data))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for glam::IVec3 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data: [i32; 3] = <_>::read_ds_endian(reader, endian)?;

        Ok(glam::IVec3::from_array(data))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for glam::IVec4 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data: [i32; 4] = <_>::read_ds_endian(reader, endian)?;

        Ok(glam::IVec4::from_array(data))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for glam::Quat {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data: [f32; 4] = <_>::read_ds_endian(reader, endian)?;

        Ok(glam::Quat::from_array(data))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}

impl TigerReadable for glam::Mat4 {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data: [f32; 16] = <_>::read_ds_endian(reader, endian)?;

        Ok(glam::Mat4::from_cols_array(&data))
    }

    const ZEROCOPY: bool = true;
    const SIZE: usize = std::mem::size_of::<Self>();
}
