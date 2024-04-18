use darling::{ast::NestedMeta, FromField, FromMeta};
use proc_macro2::{self, TokenStream};
use quote::{quote, ToTokens};

#[derive(Debug, Clone, Copy, Default, FromMeta)]
#[darling(default)]
enum FieldType {
    #[default]
    Normal,

    // cohae: These types currently serve no purpose
    #[darling(rename = "taghash")]
    TagHash,
    #[darling(rename = "taghash64")]
    TagHash64,

    /// Casts the raw tag data to the specified type
    /// Works for Size+Clone types
    #[darling(rename = "taghash_raw")]
    TagHashRaw,
}

#[derive(FromMeta, Default, Debug)]
struct Opts {
    #[darling(rename = "id")]
    struct_id: Option<u32>,

    #[darling(rename = "etype")]
    struct_type: Option<u8>,
    #[darling(rename = "esubtype")]
    struct_subtype: Option<u8>,

    #[darling(rename = "size")]
    struct_size: Option<usize>,
}

#[derive(FromField, Default, Debug)]
#[darling(default, attributes(tag), forward_attrs(allow, doc, cfg))]
struct OptsField {
    #[darling(rename = "offset")]
    field_offset: Option<u64>,
    #[darling(rename = "ftype")]
    field_type: FieldType,

    debug: bool,
}

#[proc_macro_attribute]
pub fn tiger_tag(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item2 = item.clone();
    let mut struc = syn::parse_macro_input!(item2 as syn::ItemStruct);

    let args = NestedMeta::parse_meta_list(attr.into()).unwrap();
    let opts = match <Opts as darling::FromMeta>::from_list(&args) {
        Ok(x) => x,
        Err(e) => return e.write_errors().into(),
    };

    let ident = struc.ident.clone();

    let struct_id = opts.struct_id;
    let impl_struct_id = if let Some(struct_id) = struct_id {
        quote! {
            const ID: Option<u32> = Some(#struct_id);
        }
    } else {
        quote! {}
    };

    if opts.struct_subtype.is_some() && opts.struct_type.is_none() {
        return quote! {
            compile_error!("If subtype is defined, type must be defined as well");
        }
        .into();
    }

    let impl_struct_type = if let (Some(st), sbt) = (opts.struct_type, opts.struct_subtype) {
        let sbt = if let Some(sbt) = sbt {
            quote! { Some(#sbt) }
        } else {
            quote! { None }
        };

        quote! {
            const ETYPE: Option<(u8, Option<u8>)> = Some((#st, #sbt));
        }
    } else {
        quote! {}
    };

    let mut fieldstream_zerocopy = TokenStream::new();
    let mut fieldstream_size = TokenStream::new();

    fieldstream_size.extend(quote! {
        0
    });

    let mut zerocopy_base_safety = true;

    let mut fieldstream = TokenStream::new();
    let mut fieldstream_assign = TokenStream::new();
    let mut uses_offsets = false;
    for f in struc.fields.iter_mut() {
        let d = OptsField::from_field(f).expect("Invalid field options");
        // Remove the tag attribute from the field
        f.attrs.retain(|v| !v.meta.path().is_ident("tag"));

        let fident = f.ident.clone();
        let ftype = f.ty.clone();
        if let Some(fident) = fident {
            if let Some(field_offset) = d.field_offset {
                uses_offsets = true;
                zerocopy_base_safety = false;
                fieldstream.extend(quote! {
                    reader.seek(::std::io::SeekFrom::Start(start_pos+#field_offset))?;
                });
            }

            if d.debug {
                zerocopy_base_safety = false;
                fieldstream.extend(quote! {
                    let offset = reader.stream_position()?;
                });
            }

            fieldstream.extend(quote! {
                let #fident = <_>::read_ds_endian(reader, endian)?;
            });

            if d.debug {
                fieldstream.extend(quote! {
                    eprintln!("[{}.{} @ 0x{:X}]: {:#X?}", std::any::type_name::<Self>(), stringify!(#fident), offset, #fident);
                });
            }

            fieldstream_assign.extend(quote! {
                #fident,
            });

            fieldstream_size.extend(quote! {
                + <#ftype>::SIZE
            });

            // Carry over the zerocopy flag from fields. All fields must be ZEROCOPY for the struct to be ZEROCOPY
            fieldstream_zerocopy.extend(quote! {
                && <#ftype>::ZEROCOPY
            });
        }
    }

    let impl_struct_size = if let Some(defined_size) = opts.struct_size {
        quote! {
            const SIZE: usize = #defined_size;
        }
    } else {
        if uses_offsets {
            return quote! {
                compile_error!("Structs with offsets must define a size");
            }
            .into();
        }

        quote! {
            const SIZE: usize = #fieldstream_size;
        }
    };

    let item_stream = struc.to_token_stream();
    let output = quote! {
        #[repr(C)]
        #item_stream

        impl ::tiger_parse::TigerReadable for #ident {
            fn read_ds_endian<R: ::std::io::Read + ::std::io::Seek>(reader: &mut R, endian: ::tiger_parse::Endian) -> ::tiger_parse::Result<Self> {
                let start_pos = reader.stream_position()?;

                #fieldstream

                Ok(Self {
                    #fieldstream_assign
                })
            }

            const ZEROCOPY: bool = #zerocopy_base_safety #fieldstream_zerocopy;

            #impl_struct_id
            #impl_struct_type
            #impl_struct_size
        }
    };

    output.into()
}
