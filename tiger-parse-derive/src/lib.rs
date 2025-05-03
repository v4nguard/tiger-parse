use darling::{ast::NestedMeta, FromField, FromMeta};
use proc_macro2::{self, Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};

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
    let struct_id_or_zero = struct_id.unwrap_or(0);
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

    let mut last_offset = 0u64;
    let mut fieldstream = TokenStream::new();
    let mut fieldstream_assign = TokenStream::new();
    let mut uses_offsets = false;
    let mut is_tuple = false;
    for (i, f) in struc.fields.iter_mut().enumerate() {
        let d = OptsField::from_field(f).expect("Invalid field options");

        let (fident, display_ident) = if let Some(fident) = f.ident.clone() {
            (fident.clone(), fident.to_string())
        } else {
            is_tuple = true;
            (
                Ident::new(format!("f{i}").as_str(), Span::call_site()),
                format!("{i}"),
            )
        };

        let ftype = f.ty.clone();
        if let Some(field_offset) = d.field_offset {
            if field_offset >= last_offset {
                last_offset = field_offset;
            } else {
                return quote! {
                    compile_error!("Field offsets must be in ascending order");
                }
                .into();
            }

            uses_offsets = true;
            zerocopy_base_safety = false;
            fieldstream.extend(quote! {
                reader.seek(::std::io::SeekFrom::Start(start_pos+#field_offset))?;
            });

            // Reset size field stream
            fieldstream_size = quote! {
                (#field_offset as usize)
            };
        }

        if d.debug {
            zerocopy_base_safety = false;
            fieldstream.extend(quote! {
                let offset = reader.stream_position()?;
            });
        }

        fieldstream.extend(quote! {
            let #fident = <_>::read_ds_endian(reader, endian).with_field(&tiger_parse::ShortName::of::<Self>().to_string(), #display_ident)?;
        });

        if d.debug {
            fieldstream.extend(quote! {
                    eprintln!("[{}.{} @ 0x{:X}]: {:#X?}", tiger_parse::ShortName::of::<Self>(), stringify!(#fident), offset, #fident);
                });
        }

        fieldstream_assign.extend(quote! {
            #fident,
        });

        fieldstream_size.extend(quote! {
            + <#ftype as ::tiger_parse::TigerReadable>::SIZE
        });

        // Carry over the zerocopy flag from fields. All fields must be ZEROCOPY for the struct to be ZEROCOPY
        fieldstream_zerocopy.extend(quote! {
            && <#ftype as ::tiger_parse::TigerReadable>::ZEROCOPY
        });
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

    let return_statement = if is_tuple {
        quote! {
            Self(#fieldstream_assign)
        }
    } else {
        quote! {
            Self {
                #fieldstream_assign
            }
        }
    };

    let mut reflected_struct_stream = TokenStream::new();
    if cfg!(feature = "reflect") {
        let mut struct_reflect_field_stream = TokenStream::new();
        let mut struct_reflect_field_offset_stream = TokenStream::new();
        struct_reflect_field_offset_stream.extend(quote! {
            0
        });
        for (i, f) in struc.fields.iter_mut().enumerate() {
            let d = OptsField::from_field(f).expect("Invalid field options");
            let fident = if let Some(fident) = f.ident.clone() {
                fident.to_string()
            } else {
                format!("{i}")
            };

            let explicit_offset = d.field_offset.is_some();
            if let Some(field_offset) = d.field_offset {
                struct_reflect_field_offset_stream = quote! {
                    (#field_offset as usize)
                };
            }

            let ftype = f.ty.clone();
            let type_reflect = type_to_reflect(&f.ty);

            struct_reflect_field_stream.extend(quote! {
                ::tiger_parse::reflect::ReflectedField {
                    name: std::borrow::Cow::Borrowed(#fident),
                    size: <#ftype as ::tiger_parse::TigerReadable>::SIZE,
                    offset: #struct_reflect_field_offset_stream,
                    explicit_offset: #explicit_offset,
                    ty: #type_reflect,
                },
            });

            struct_reflect_field_offset_stream.extend(quote! {
                + <#ftype as ::tiger_parse::TigerReadable>::SIZE
            });
        }

        let reflected_struct_ident = format_ident!("_{}_REFLECT", ident);
        reflected_struct_stream.extend(quote! {
            #[allow(non_upper_case_globals)]
            #[::tiger_parse::distributed_slice(crate::STRUCTS)]
            static #reflected_struct_ident: ::tiger_parse::reflect::ReflectedStruct = ::tiger_parse::reflect::ReflectedStruct {
                id: #struct_id_or_zero,
                name: std::borrow::Cow::Borrowed(stringify!(#ident)),
                is_tuple: #is_tuple,
                fields: std::borrow::Cow::Borrowed(&[
                    #struct_reflect_field_stream
                ]),
                size: <#ident as ::tiger_parse::TigerReadable>::SIZE,
            };
        });
    }

    // Strip the tag attribute from all fields
    for f in struc.fields.iter_mut() {
        f.attrs.retain(|v| !v.meta.path().is_ident("tag"));
    }

    let item_stream = struc.to_token_stream();
    let output = quote! {
        #[repr(C)]
        #item_stream

        impl ::tiger_parse::TigerReadable for #ident {
            fn read_ds_endian<R: ::std::io::Read + ::std::io::Seek>(reader: &mut R, endian: ::tiger_parse::Endian) -> ::tiger_parse::Result<Self> {
                use tiger_parse::ResultExt;
                let start_pos = reader.stream_position()?;

                #fieldstream

                Ok(#return_statement)
            }

            const ZEROCOPY: bool = #zerocopy_base_safety #fieldstream_zerocopy;

            #impl_struct_id
            #impl_struct_type
            #impl_struct_size
        }

        #reflected_struct_stream

        // If a custom size is specific, it must be at least the total sum of the field type sizes
        const _: () = {
            assert!(<#ident as ::tiger_parse::TigerReadable>::SIZE >= (#fieldstream_size), "Declared struct size must be greater than or equal to the total sum of the field type sizes");
        };
    };

    output.into()
}

fn type_to_reflect(ty: &syn::Type) -> TokenStream {
    let t = match ty {
        syn::Type::Array(type_array) => {
            let len = type_array.len.clone();
            let element_type = type_to_reflect(&type_array.elem);
            quote! {
                FixedArray(#len, ::tiger_parse::reflect::CowBox::Borrowed(&#element_type))
            }
        }
        syn::Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap();
            match last_segment.ident.to_string().as_str() {
                "u8" => quote!(UInt8),
                "u16" => quote!(UInt16),
                "u32" => quote!(UInt32),
                "u64" => quote!(UInt64),
                "i8" => quote!(Int8),
                "i16" => quote!(Int16),
                "i32" => quote!(Int32),
                "i64" => quote!(Int64),
                "f32" => quote!(Float32),
                "f64" => quote!(Float64),
                "Vec2" => quote!(Vec2),
                "Vec3" => quote!(Vec3),
                "Vec4" => quote!(Vec4),
                "TagHash" => quote!(TagHash),
                "Vec" => {
                    let syn::PathArguments::AngleBracketed(path_args) =
                        last_segment.arguments.clone()
                    else {
                        unreachable!("Expected angle bracketed arguments for Vec type");
                    };

                    let syn::GenericArgument::Type(inner_ty) = &path_args.args[0] else {
                        unreachable!("Expected type argument for Vec type");
                    };

                    let inner_ty_reflected = type_to_reflect(&inner_ty);

                    quote!(Array(::tiger_parse::reflect::CowBox::Borrowed(&#inner_ty_reflected)))
                }
                s => quote!(Other(std::borrow::Cow::Borrowed(#s))),
            }
        }
        syn::Type::Tuple(type_tuple) => {
            let mut fields = TokenStream::new();
            for field in &type_tuple.elems {
                let field_reflected = type_to_reflect(field);
                fields.extend(quote!(#field_reflected, ));
            }
            quote!(Tuple(std::borrow::Cow::Borrowed(&[#fields])))
        }
        _ => {
            quote!(Other(std::borrow::Cow::Borrowed(stringify!(#ty))))
        }
    };

    quote!(::tiger_parse::reflect::ReflectedType::#t)
}
