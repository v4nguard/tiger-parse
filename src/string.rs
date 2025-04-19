use crate::{error::Error, TigerReadable};

#[derive(Debug, Clone)]
pub struct NullString(pub String);

impl TigerReadable for NullString {
    fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        _endian: crate::Endian,
    ) -> crate::Result<Self> {
        let mut buf = String::new();

        let mut b = [0u8; 1];
        for _ in 0..10240 {
            reader.read_exact(&mut b)?;
            if b[0] == 0 {
                return Ok(NullString(buf));
            }
            buf.push(b[0] as char);
        }

        Err(Error::StringTooLong)
    }

    const ZEROCOPY: bool = false;

    const ID: Option<u32> = None;
    const SIZE: usize = 0;
}

impl std::fmt::Display for NullString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
