use std::io::Cursor;

use tiger_parse::{Endian, Padding, TigerReadable};
use tiger_parse_derive::tiger_tag;

#[derive(Debug, Clone)]
#[tiger_tag(id = 0x80806D44)]
pub struct SStaticMesh {
    pub file_size: u64,
    pub unk8: TagHash,
    pub _pad: Padding<4>,
    #[tag(debug)]
    pub materials: Vec<TagHash>,
    pub unk20: Vec<SStaticMeshOverlay>, // Overlay/transparent meshes
    pub unk30: [u32; 2],
    pub unk38: [f32; 6],
    pub unk50: glam::Vec3, // ? Similar to model_offset, but not quite right...
    pub unk5c: f32,
}

#[derive(Debug)]
#[tiger_tag(id = 0x80806D30, size = 0x60)]
pub struct SStaticMeshData {
    pub file_size: u64,
    pub mesh_groups: Vec<Unk8080719b>,
    pub parts: Vec<Unk8080719a>,
    pub buffers: Vec<(TagHash, TagHash, TagHash, TagHash)>,

    #[tag(offset = 0x40)]
    pub mesh_offset: glam::Vec3,
    pub mesh_scale: f32,
    pub texture_coordinate_scale: f32,
    pub texture_coordinate_offset: glam::Vec2,
}

#[derive(Debug, Clone)]
#[tiger_tag(id = 0xFFFFFFFF)]
pub struct Unk8080719a {
    pub index_start: u32,
    pub index_count: u32,
    pub buffer_index: u8,
    pub unk9: u8,
    pub lod_category: u8,   //ELodCategory,
    pub primitive_type: u8, //EPrimitiveType,
}

#[derive(Debug, Clone)]
#[tiger_tag(id = 0xFFFFFFFF)]
pub struct Unk8080719b {
    pub part_index: u16,
    pub unk2: u8,
    pub unk3: u8,
    pub unk5: u16,
}

#[derive(Debug, Clone)]
#[tiger_tag(id = 0xFFFFFFFF, size = 0x98)]
pub struct SStaticMeshInstances {
    #[tag(offset = 0x18)]
    pub occlusion_bounds: TagHash,

    #[tag(offset = 0x40)]
    pub transforms: Vec<Unk808071a3>,
    pub unk50: u64,
    pub unk58: [u64; 4],
    pub statics: Vec<TagHash>,
    pub instance_groups: Vec<SStaticMeshInstanceGroup>,
}

#[derive(Debug, Clone)]
#[tiger_tag(id = 0xFFFFFFFF)]
pub struct SStaticMeshInstanceGroup {
    pub instance_count: u16,
    pub instance_start: u16,
    pub static_index: u16,
    pub unk6: u16,
}

#[derive(Debug, Clone)]
#[tiger_tag(id = 0xFFFFFFFF)]
pub struct Unk808071a3 {
    pub rotation: glam::Quat,
    pub translation: glam::Vec3,
    pub scale: glam::Vec3,
    pub unk28: u32,
    pub unk2c: u32,
    pub unk30: [u32; 4],
}

#[derive(Debug, Clone)]
#[tiger_tag(id = 0xFFFFFFFF)]
pub struct SStaticMeshOverlay {
    pub render_stage: u8, // TfxRenderStage,
    pub unk1: u8,
    pub lod: u8, // ELodCategory,
    pub unk3: i8,
    pub primitive_type: u8, // EPrimitiveType,
    pub unk5: u8,
    pub unk6: u16,
    pub index_buffer: TagHash,
    pub vertex_buffer: TagHash,
    pub vertex_buffer2: TagHash,
    pub color_buffer: TagHash,
    pub index_start: u32,
    pub index_count: u32,
    pub material: TagHash,
}

#[derive(Debug, Clone)]
#[tiger_tag(etype = 32, esubtype = 4)]
pub struct TagHash {
    #[tag(debug)]
    pub value: u32,
}

#[tiger_tag]
struct Test(i32, u32);

pub fn main() {
    let data = include_bytes!("testdata.bin");
    let mut cursor = Cursor::new(data);

    let v: SStaticMesh = TigerReadable::read_ds_endian(&mut cursor, Endian::Little).unwrap();
    println!("{:#x?}", v);

    assert_eq!(TagHash::ETYPE, Some((32, Some(4))));

    const TEST: [u8; 8] = [0xfe, 0xff, 0xff, 0xff, 0x7b, 0x00, 0x00, 0x00];
    let mut cursor = std::io::Cursor::new(&TEST);

    let test = Test::read_ds(&mut cursor).unwrap();
    assert_eq!(test.0, -2);
    assert_eq!(test.1, 123);
}
