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
            for v in data.iter_mut() {
                *v = T::read_ds_endian(reader, endian)?;
            }

            data
        };

        Ok(data)
    }

    const ZEROCOPY: bool = T::ZEROCOPY;
    const SIZE: usize = N * T::SIZE;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_array() -> anyhow::Result<()> {
        const DATA: &[u8] = &[
            2, 0, 0, 0, 239, 190, 173, 222, 0, 1, 2, 3, 4, 5, 6, 7, 1_0, 1_2, 1_3,
        ];
        let mut cursor = Cursor::new(&DATA);

        assert_eq!(<[u32; 2]>::read_ds(&mut cursor)?, [2, 0xDEADBEEF]);
        assert_eq!(<[u8; 8]>::read_ds(&mut cursor)?, [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(<[u8; 3]>::read_ds(&mut cursor)?, [1_0, 1_2, 1_3]);

        Ok(())
    }
}
