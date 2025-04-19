use crate::{error::Error, TigerReadable};

#[derive(Debug, Clone)]
pub struct Padding<const N: usize>;

impl<const N: usize> TigerReadable for Padding<N> {
    fn read_ds_endian<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: crate::Endian,
    ) -> crate::Result<Self> {
        // Read N bytes and make sure they are all zero
        let mut buf = [0; N];
        reader.read_exact(&mut buf)?;
        if buf.iter().all(|&x| x == 0) {
            Ok(Self)
        } else {
            Err(Error::PaddingNotZero(buf.to_vec()))
        }
    }

    const SIZE: usize = N;
}
