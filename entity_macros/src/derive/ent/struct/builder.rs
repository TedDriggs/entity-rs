use super::{utils, Ent};
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Path};

pub fn impl_ent_builder(
    root: &Path,
    input: &DeriveInput,
    ent: &Ent,
) -> darling::Result<TokenStream> {
    let ent_name = &input.ident;
    let builder_name = format_ident!("{}Builder", ent_name);
    let builder_error_name = format_ident!("{}Error", builder_name);

    let vis = &input.vis;
    let named_fields = &utils::get_named_fields(input)?.named;
    let ent_database_field_name = &ent.database;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut struct_field_names = Vec::new();
    let mut struct_field_defaults = Vec::new();
    let mut struct_fields = Vec::new();
    let mut struct_setters = Vec::new();
    let mut error_variants = Vec::new();
    let mut error_variant_field_names = Vec::new();
    let mut build_assignments = Vec::new();
    let mut has_normal_struct_field = false;

    for f in named_fields {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;

        struct_field_names.push(name);

        // If our special id field, we set an automatic default of the
        // ephemeral id
        if name == &ent.id {
            struct_fields.push(quote!(#name: #ty));
            struct_field_defaults.push(quote!(#root::EPHEMERAL_ID));
            build_assignments.push(quote!(#name: self.#name));

            struct_setters.push(quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#name = value;
                    self
                }
            });
        // If our database field, we set it to an empty ref by default
        } else if name == &ent.database {
            struct_fields.push(quote!(#name: #ty));
            struct_field_defaults.push(quote!(#root::WeakDatabaseRc::new()));
            build_assignments.push(quote!(#name: self.#name));

            struct_setters.push(quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#name = value;
                    self
                }
            });
        // If our created or last_updated field, we set it to the current time
        } else if name == &ent.created || name == &ent.last_updated {
            struct_fields.push(quote!(#name: #ty));
            struct_field_defaults.push(quote!(::std::time::SystemTime::now()
                .duration_since(::std::time::UNIX_EPOCH)
                .expect("Corrupt system time")
                .as_millis()
                as ::std::primitive::u64));
            build_assignments.push(quote!(#name: self.#name));

            struct_setters.push(quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#name = value;
                    self
                }
            });
        // Otherwise, we have no default available for fields & edges
        } else {
            has_normal_struct_field = true;
            struct_fields.push(quote!(#name: ::std::option::Option<#ty>));
            struct_field_defaults.push(quote!(::std::option::Option::None));

            let error_variant = format_ident!("Missing{}", name.to_string().to_camel_case());
            build_assignments.push(quote! {
                #name: self.#name.ok_or(#builder_error_name::#error_variant)?
            });
            error_variants.push(error_variant);
            error_variant_field_names.push(name);

            struct_setters.push(quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#name = ::std::option::Option::Some(value);
                    self
                }
            });
        }
    }

    let display_fmt_inner = if has_normal_struct_field {
        quote! {
            match self {
                #(
                    Self::#error_variants => ::std::write!(
                        f,
                        concat!("Missing ", ::std::stringify!(#error_variant_field_names)),
                    ),
                )*
            }
        }
    } else {
        quote!(::std::result::Result::Ok(()))
    };

    Ok(quote! {
        #[derive(
            ::std::marker::Copy,
            ::std::clone::Clone,
            ::std::fmt::Debug,
            ::std::cmp::PartialEq,
            ::std::cmp::Eq,
        )]
        #[automatically_derived]
        #[allow(clippy::enum_variant_names)]
        #vis enum #builder_error_name {
            #(#error_variants),*
        }

        impl ::std::fmt::Display for #builder_error_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #display_fmt_inner
            }
        }

        impl ::std::error::Error for #builder_error_name {}

        impl #impl_generics #ent_name #ty_generics #where_clause {
            /// Begin building a new ent, initialized using the global database
            /// if it is available
            pub fn build() -> #builder_name #ty_generics #where_clause {
                <#builder_name #ty_generics as ::std::default::Default>::default()
                    .#ent_database_field_name(#root::global::db())
            }
        }

        #[automatically_derived]
        #vis struct #builder_name #ty_generics #where_clause {
            #(#struct_fields),*
        }

        #[automatically_derived]
        impl #impl_generics ::std::default::Default for #builder_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(
                        #struct_field_names: #struct_field_defaults,
                    )*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics #builder_name #ty_generics #where_clause {
            #(#struct_setters)*

            /// Called when finished constructing the ent, will consume the
            /// builder and return a new ent **without** committing it to
            /// the database.
            pub fn finish(self) -> ::std::result::Result<#ent_name #ty_generics, #builder_error_name> {
                ::std::result::Result::Ok(#ent_name {
                    #(#build_assignments),*
                })
            }

            /// Called when finished constructing the ent, will consume the
            /// builder and return a new ent after committing it to the
            /// associated database. If no database is connected to the ent,
            /// this will fail.
            pub fn finish_and_commit(self) -> ::std::result::Result<
                #root::DatabaseResult<#ent_name #ty_generics>,
                #builder_error_name,
            > {
                self.finish().map(|mut ent| {
                    if let ::std::result::Result::Err(x) = #root::Ent::commit(&mut ent) {
                        ::std::result::Result::Err(x)
                    } else {
                        ::std::result::Result::Ok(ent)
                    }
                })
            }
        }
    })
}
