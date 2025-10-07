mod ast;
mod flags_impl;
mod struct_impl;

#[proc_macro_attribute]
pub fn tiger_type(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    struct_impl::generate(attr, item)
}

#[proc_macro_derive(TigerFlags)]
pub fn tiger_flags(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    flags_impl::generate(input)
}
