use chroma_dbg::ChromaConfig;

#[cfg(not(feature = "reflect"))]
compile_error!("The 'reflect' feature is required for the reflect example.");

#[cfg(feature = "reflect")]
#[tiger_parse::distributed_slice]
static STRUCTS: [tiger_parse::reflect::ReflectedStruct];

#[cfg(feature = "reflect")]
mod structs {
    use tiger_parse::tiger_tag;

    #[derive(Debug, Clone)]
    #[tiger_tag]
    pub struct TagHash(u32);

    #[derive(Debug, Clone)]
    #[tiger_tag(id = 0x808087D1)]
    pub struct SDynamicMeshPart {
        pub technique: TagHash,
        pub variant_shader_index: u16,
        pub primitive_type: u8,
        pub unk7: u8,
        // 0x8
        pub index_start: u32,
        pub index_count: u32,

        // 0x10
        pub unk10: u32,
        pub external_identifier: u16,
        pub unk16: u16,
        // 0x18
        pub flags: u32,

        // 0x20
        pub unk1c: u16,
        // Running increment, +1 per part
        pub unk1e: u16,

        // 0x20
        pub unk20: u8,
        pub lod_category: u8,
        pub unk22: u16,
        pub unk24: u32,
    }

    #[derive(Debug, Clone)]
    #[tiger_tag(id = 0x808087CB, size = 0x88)]
    pub struct SDynamicMesh {
        pub vertex0_buffer: TagHash,
        pub vertex1_buffer: TagHash,
        pub buffer2: TagHash,
        pub buffer3: TagHash,
        pub index_buffer: TagHash,
        pub color_buffer: TagHash,
        pub skinning_buffer: TagHash,
        pub unk1c: u32,
        pub parts: Vec<SDynamicMeshPart>,
        /// Range of parts to render per render stage
        /// Can be obtained as follows:
        ///     - Start = part_range_per_render_stage[stage]
        ///     - End = part_range_per_render_stage[stage + 1]
        pub part_range_per_render_stage: [u16; 24 + 1],
        pub input_layout_per_render_stage: [u8; 24],
        _pad7a: [u16; 3],
    }

    #[derive(Debug)]
    #[tiger_tag(id = 0x80808620, size = 0x60)]
    pub struct SStaticMeshData {
        pub file_size: u64,
        pub mesh_groups: Vec<SStaticMeshGroup>,
        pub parts: Vec<SStaticMeshPart>,
        pub buffers: Vec<(TagHash, TagHash, TagHash, TagHash)>,
        pub unk38: u32,

        #[tag(offset = 0x40)]
        pub mesh_offset: glam::Vec3,
        pub mesh_scale: f32,
        pub texture_coordinate_scale: f32,
        pub texture_coordinate_offset: glam::Vec2,
        pub max_color_index: u32,
    }

    #[derive(Debug, Clone)]
    #[tiger_tag(id = 0x80808627)]
    pub struct SStaticMeshPart {
        pub index_start: u32,
        pub index_count: u32,
        pub buffer_index: u8,
        pub unk9: u8,
        pub lod_category: u8,
        pub primitive_type: u8,
    }

    #[derive(Debug, Clone)]
    #[tiger_tag(id = 0x80808628)]
    pub struct SStaticMeshGroup {
        pub part_index: u16,
        pub render_stage: u8,
        pub input_layout_index: u8,
        pub unk5: u8,
        /// Usually 1.
        /// If 2, at least for render_stage=ShadowGenerate, the geometry in this group has some kind of vertex animation
        /// This can be used to differentiate stationary static geometry from moving/animated statics
        pub unk6: u8,
    }

    #[derive(Debug, Clone)]
    #[tiger_tag(id = 0x80808080, size = 0x10)]
    pub struct Test {
        pub unk0: u32,
        pub unk1: u32,
        pub unk2: u32,
    }
}

fn main() {
    let cc = ChromaConfig {
        inline_struct: chroma_dbg::InlineThreshold::MaxLength(256),
        integer_format: chroma_dbg::IntegerFormat::HexWhenOver(8),
        ..Default::default()
    };
    for s in STRUCTS.iter() {
        println!("{}", cc.format(s));
        println!("{s}");
    }
}
