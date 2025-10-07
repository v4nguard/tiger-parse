use quote::quote;

mod ast;
mod enum_impl;
mod flags_impl;
mod struct_impl;

#[proc_macro_attribute]
pub fn tiger_type(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item2 = item.clone();
    let input = syn::parse_macro_input!(item2 as syn::DeriveInput);
    match input.data {
        syn::Data::Struct(data_struct) => {
            let struc = syn::ItemStruct {
                attrs: input.attrs.clone(),
                vis: input.vis.clone(),
                struct_token: data_struct.struct_token,
                ident: input.ident.clone(),
                generics: input.generics.clone(),
                fields: data_struct.fields.clone(),
                semi_token: data_struct.semi_token,
            };
            struct_impl::generate(attr, struc)
        }
        syn::Data::Enum(data_enum) => {
            let enumm = syn::ItemEnum {
                attrs: input.attrs.clone(),
                vis: input.vis.clone(),
                enum_token: data_enum.enum_token,
                ident: input.ident.clone(),
                generics: input.generics.clone(),
                brace_token: data_enum.brace_token,
                variants: data_enum.variants.clone(),
            };

            enum_impl::generate(enumm)
        }
        syn::Data::Union(_) => quote! {
            compile_error!("Unions are not supported");
        }
        .into(),
    }
}

#[proc_macro_derive(TigerFlags)]
pub fn tiger_flags(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    flags_impl::generate(input)
}
