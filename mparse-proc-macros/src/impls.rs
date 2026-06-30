use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result, spanned::Spanned};

pub fn stringify_impl(input: DeriveInput) -> Result<TokenStream> {
  match input.data {
    Data::Enum(e) => {
      let ty = input.ident;
      let variants = e.variants;
      let exprs = variants.iter().map(|variant| {
        let ident = &variant.ident;
        let name = ident.to_string();
        let fields = &variant.fields;

        let pattern = match fields {
          syn::Fields::Unnamed(_) => quote! {
              #ty::#ident(..)
          },

          syn::Fields::Named(_) => quote! {
              #ty::#ident { .. }
          },

          syn::Fields::Unit => quote! { #ty::#ident },
        };

        quote! {
          #pattern => #name,
        }
      });

      Ok(quote! {
        impl #ty {
          pub fn as_str(&self) -> &'static str {
            match self {
              #(#exprs)*
            }
          }
        }
      })
    }

    _ => Err(Error::new(input.span(), "expected enum decl")),
  }
}
