use crate::TigerReadable;

impl<T: TigerReadable, const N: usize> TigerReadable for [T; N] {
    fn read_ds_endian<R: ::std::io::Read + ::std::io::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> anyhow::Result<Self> {
        let data = if T::ZEROCOPY && endian == crate::Endian::Little {
            unsafe {
                let mut data: Self = std::mem::zeroed();
                reader.read_exact(std::slice::from_raw_parts_mut(
                    data.as_mut_ptr() as *mut u8,
                    N * std::mem::size_of::<T>(),
                ))?;
                data
            }
        } else {
            let mut data: Self = unsafe { std::mem::zeroed() };
            for i in 0..N {
                data[i] = T::read_ds_endian(reader, endian)?;
            }

            data
        };

        Ok(data)
    }

    const ZEROCOPY: bool = T::ZEROCOPY;
    const SIZE: usize = N * T::SIZE;
}
