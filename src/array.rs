use crate::{ResultExt, TigerReadable};

impl<T: TigerReadable, const N: usize> TigerReadable for [T; N] {
    fn read_ds_endian<R: ::std::io::Read + ::std::io::Seek>(
        reader: &mut R,
        endian: crate::Endian,
    ) -> crate::Result<Self> {
        let mut data: Self = unsafe { std::mem::zeroed() };
        for (i, v) in data.iter_mut().enumerate() {
            unsafe {
                (&raw mut *v).write(T::read_ds_endian(reader, endian).with_array_element(i)?);
            }
        }

        Ok(data)
    }

    const SIZE: usize = N * T::SIZE;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_array() -> crate::Result<()> {
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
