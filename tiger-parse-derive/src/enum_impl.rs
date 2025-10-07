use proc_macro2::Ident;
use quote::quote;

use crate::ast::Repr;

pub fn generate(enumm: syn::ItemEnum) -> proc_macro::TokenStream {
    let ident = enumm.ident.clone();

    let enum_idents: Vec<Ident> = enumm.variants.iter().map(|v| v.ident.clone()).collect();

    let repr = Repr::from_attributes(&enumm.attrs).unwrap();
    let repr_type = repr.ident;

    let impl_struct_size = quote! {
        const SIZE: usize = <#repr_type as ::tiger_parse::TigerReadable>::SIZE;
    };

    let variant_match = quote! {
        let value = <#repr_type as ::tiger_parse::TigerReadable>::read_ds_endian(reader, endian)?;
        match value {
            #(x if x == (Self::#enum_idents as #repr_type) => Ok(#ident::#enum_idents),)*
            _ => Err(::tiger_parse::error::Error::EnumVariantOutOfRange(value as usize)),
        }
    };

    quote! {
        #enumm

        impl ::tiger_parse::TigerReadable for #ident {
            fn read_ds_endian<R: ::std::io::Read + ::std::io::Seek>(reader: &mut R, endian: ::tiger_parse::Endian) -> ::tiger_parse::Result<Self> {
                #variant_match
            }

            const ID: Option<u32> = None;
            #impl_struct_size
        }
    }.into()
}
