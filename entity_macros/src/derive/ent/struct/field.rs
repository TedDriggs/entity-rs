use super::EntField;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Generics, Ident};

/// Implements individual methods for each of the provided fields for
/// the ent with the given name
pub(crate) fn impl_typed_field_methods(
    name: &Ident,
    generics: &Generics,
    fields: &[EntField],
) -> TokenStream {
    let mut field_methods: Vec<TokenStream> = Vec::new();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    for field in fields {
        let field_name = &field.name;
        let field_type = &field.ty;

        let getter = quote! {
            pub fn #field_name(&self) -> &#field_type {
                &self.#field_name
            }
        };
        field_methods.push(getter);

        if field.mutable {
            let setter_name = format_ident!("set_{}", field_name);
            let setter = quote! {
                pub fn #setter_name(&mut self, x: #field_type) -> #field_type {
                    ::std::mem::replace(&mut self.#field_name, x)
                }
            };
            field_methods.push(setter);
        }
    }

    quote! {
        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#field_methods)*
        }
    }
}
