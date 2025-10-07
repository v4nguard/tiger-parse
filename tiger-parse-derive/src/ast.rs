// https://github.com/Juici/int-enum-rs/blob/master/src/ast.rs

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, DataEnum, Generics, Meta, Token};

pub struct EnumInput {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub generics: Generics,
    pub data: DataEnum,
}

const VALID_REPRS: &[&str] = &[
    "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128", "isize", "usize",
];

pub struct Repr {
    pub ident: Ident,
}

impl ToTokens for Repr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
    }
}

impl Repr {
    pub fn from_attributes(attrs: &[Attribute]) -> Option<Repr> {
        for attr in attrs {
            if !attr.path().is_ident("repr") {
                continue;
            }

            let Ok(nested) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                return None;
            };

            for meta in nested {
                let Meta::Path(path) = meta else { continue };
                let Some(ident) = path.get_ident() else {
                    continue;
                };

                let sym = ident.to_string();
                let sym = sym.strip_prefix("r#").unwrap_or(sym.as_ref());

                if VALID_REPRS.contains(&sym) {
                    return Some(Repr {
                        ident: ident.clone(),
                    });
                }
            }
        }

        // If no repr is specified then default to `isize`.
        // https://doc.rust-lang.org/reference/items/enumerations.html#r-items.enum.discriminant.repr-rust
        Some(Repr {
            ident: Ident::new("isize", Span::call_site()),
        })
    }
}
