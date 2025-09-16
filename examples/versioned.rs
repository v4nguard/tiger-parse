use std::io::Cursor;

use tiger_parse::{tiger_tag, Endian, TigerReadable};

fn main() {
    const DATA: &[u8] = &[
        0x11, 0x11, 0x11, 0x11, // a: u32
        0x00, 0x00, 0x80, 0x3F, // b: f32 (1.0)
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, // c: u64
        0x33, 0x33, 0x33, 0x33, // d: u32 (only if version > Destiny2BeyondLight)
    ];

    let mut cur = Cursor::new(DATA);
    let s = TestStruct::read_ds_endian(&mut cur, Endian::Little).unwrap();
    println!("{:#X?}", s);
}

#[tiger_tag(id = 0x11111111)]
#[derive(Debug)]
struct TestStruct {
    a: u32,
    b: f32,
    c: u64,
    #[tag(if = "version > DestinyVersion::Destiny2BeyondLight")]
    d: Option<u32>,
    #[tag(if = "version < DestinyVersion::Destiny2BeyondLight")]
    e: u32,
    #[tag(
        if = "version < DestinyVersion::Destiny2BeyondLight",
        default = "0xDEADBEEF"
    )]
    f: u32,
}
