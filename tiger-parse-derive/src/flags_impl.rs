use quote::quote;

pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: proc_macro2::TokenStream = input.into();
    let item: syn::ItemStruct = syn::parse2(ast).expect("Failed to parse item as struct");

    let ident = &item.ident;
    quote! {
        impl ::tiger_parse::TigerReadable for #ident {
            fn read_ds_endian<R: std::io::prelude::Read + std::io::prelude::Seek>(
                reader: &mut R,
                endian: ::tiger_parse::Endian,
            ) -> ::tiger_parse::Result<Self> {
                let bits: <Self as bitflags::Flags>::Bits = ::tiger_parse::TigerReadable::read_ds_endian(reader, endian)?;
                Ok(<Self as bitflags::Flags>::from_bits_truncate(bits))
            }

            const ID: Option<u32> = None;
            const SIZE: usize = <Self as bitflags::Flags>::Bits::SIZE;
        }
    }.into()
}
