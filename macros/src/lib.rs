use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Spanned)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let output = match &input.data {
        syn::Data::Struct(data_struct) => derive_struct(name, &data_struct.fields),
        syn::Data::Enum(data_enum) => derive_enum(name, &data_enum.variants),
        syn::Data::Union(_) => {
            syn::Error::new_spanned(name, "`Spanned` cannot be derived on unions.")
                .to_compile_error()
        }
    };

    output.into()
}

fn derive_struct(name: &syn::Ident, fields: &syn::Fields) -> proc_macro2::TokenStream {
    let has_span = fields
        .iter()
        .any(|field| field.ident.as_ref().is_some_and(|ident| ident == "span"));

    if !has_span {
        return syn::Error::new_spanned(name, "Spanned requires a field named `span`")
            .to_compile_error();
    }

    quote! {
        impl Spanned for #name {
            fn span(&self) -> Span {
                self.span
            }
        }
    }
}

fn derive_enum(
    name: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let mut arms = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;

        match &variant.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                arms.push(quote! {
                    Self::#variant_name(value) => value.span(),
                });
            }

            _ => {
                return syn::Error::new_spanned(
                    variant,
                    "Spanned enum variants must have exactly one tuple field",
                )
                .to_compile_error();
            }
        }
    }

    quote! {
        impl Spanned for #name {
            fn span(&self) -> Span {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}
