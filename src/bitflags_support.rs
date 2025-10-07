#[cfg(test)]
mod tests {
    use tiger_parse_derive::TigerFlags;
    bitflags::bitflags! {
        #[derive(Clone, TigerFlags)]
        struct TestBitFlags32: u32 {
            const NONE = 0;
            const FLAG_A = 0x1;
            const FLAG_B = 0x2;
            const FLAG_C = 0x4;
        }
    }
    use crate::TigerReadable;

    #[test]
    fn test_bitflags() {
        const DATA: &[u8] = &[0x5, 0, 0xFF, 1];
        let mut cursor = std::io::Cursor::new(&DATA);
        let flags = TestBitFlags32::read_ds(&mut cursor).unwrap();
        assert!(flags.contains(TestBitFlags32::FLAG_A));
        assert!(flags.contains(TestBitFlags32::FLAG_C));
    }
}
