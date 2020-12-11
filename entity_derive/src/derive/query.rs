use super::EntInfo;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Visibility};

pub fn impl_ent_query(
    root: &TokenStream,
    name: &Ident,
    vis: &Visibility,
    const_type_name: &Ident,
    ent_info: &EntInfo,
) -> Result<TokenStream, syn::Error> {
    let query_name = format_ident!("{}Query", name);
    let mut struct_setters = Vec::new();

    // Default query methods available outside of fields
    struct_setters.push(quote! {
        #[doc = "Produces query that satisifies if either one of self and other pass"]
        pub fn or(self, other: #query_name) -> Self {
            Self(#root::Query::new(#root::Condition::Or(
                ::std::boxed::Box::from(self.0.into_condition()),
                ::std::boxed::Box::from(other.0.into_condition()),
            )))
        }

        #[doc = "Produces query that satisifies if only one of self and other pass"]
        pub fn xor(self, other: #query_name) -> Self {
            Self(#root::Query::new(#root::Condition::Xor(
                ::std::boxed::Box::from(self.0.into_condition()),
                ::std::boxed::Box::from(other.0.into_condition()),
            )))
        }

        #[doc = "Produces query opposite of current definition"]
        pub fn not(self) -> Self {
            Self(#root::Query::new(#root::Condition::Not(
                ::std::boxed::Box::from(self.0.into_condition()),
            )))
        }

        #[doc = "Updates query to return ent with given id"]
        pub fn with_id(self, id: #root::Id) -> Self {
            Self(self.0.chain(#root::Condition::HasId(id)))
        }

        #[doc = "Updates query to return all ents created before N milliseconds since epoch"]
        pub fn created_before(self, value: u64) -> Self {
            Self(self.0.chain(#root::Condition::Created(
                #root::TimeCondition::Before(value),
            )))
        }

        #[doc = "Updates query to return all ents created on or before N milliseconds since epoch"]
        pub fn created_on_or_before(self, value: u64) -> Self {
            Self(self.0.chain(#root::Condition::Created(
                #root::TimeCondition::OnOrBefore(value),
            )))
        }

        #[doc = "Updates query to return all ents created after N milliseconds since epoch"]
        pub fn created_after(self, value: u64) -> Self {
            Self(self.0.chain(#root::Condition::Created(
                #root::TimeCondition::After(value),
            )))
        }

        #[doc = "Updates query to return all ents created on or after N milliseconds since epoch"]
        pub fn created_on_or_after(self, value: u64) -> Self {
            Self(self.0.chain(#root::Condition::Created(
                #root::TimeCondition::OnOrAfter(value),
            )))
        }

        #[doc = "Updates query to return all ents created between N milliseconds since epoch"]
        pub fn created_between(self, start: u64, end: u64) -> Self {
            Self(self.0.chain(#root::Condition::Created(
                #root::TimeCondition::Between(start, end),
            )))
        }

        #[doc = "Updates query to return all ents created on or between N milliseconds since epoch"]
        pub fn created_on_or_between(self, start: u64, end: u64) -> Self {
            Self(self.0.chain(#root::Condition::Created(
                #root::TimeCondition::OnOrBetween(start, end),
            )))
        }

        #[doc = "Updates query to return all ents last updated before N milliseconds since epoch"]
        pub fn last_updated_before(self, value: u64) -> Self {
            Self(self.0.chain(#root::Condition::LastUpdated(
                #root::TimeCondition::Before(value),
            )))
        }

        #[doc = "Updates query to return all ents last updated on or before N milliseconds since epoch"]
        pub fn last_updated_on_or_before(self, value: u64) -> Self {
            Self(self.0.chain(#root::Condition::LastUpdated(
                #root::TimeCondition::OnOrBefore(value),
            )))
        }

        #[doc = "Updates query to return all ents last updated after N milliseconds since epoch"]
        pub fn last_updated_after(self, value: u64) -> Self {
            Self(self.0.chain(#root::Condition::LastUpdated(
                #root::TimeCondition::After(value),
            )))
        }

        #[doc = "Updates query to return all ents last updated on or after N milliseconds since epoch"]
        pub fn last_updated_on_or_after(self, value: u64) -> Self {
            Self(self.0.chain(#root::Condition::LastUpdated(
                #root::TimeCondition::OnOrAfter(value),
            )))
        }

        #[doc = "Updates query to return all ents last updated between N milliseconds since epoch"]
        pub fn last_updated_between(self, start: u64, end: u64) -> Self {
            Self(self.0.chain(#root::Condition::LastUpdated(
                #root::TimeCondition::Between(start, end),
            )))
        }

        #[doc = "Updates query to return all ents last updated on or between N milliseconds since epoch"]
        pub fn last_updated_on_or_between(self, start: u64, end: u64) -> Self {
            Self(self.0.chain(#root::Condition::LastUpdated(
                #root::TimeCondition::OnOrBetween(start, end),
            )))
        }
    });

    for f in &ent_info.fields {
        let name = &f.name;
        let ty = &f.ty;

        let name_eq = format_ident!("{}_eq", name);
        let name_lt = format_ident!("{}_lt", name);
        let name_gt = format_ident!("{}_gt", name);
        let doc_eq = format!(
            "Updates query to return all ents where {} is equal to given value",
            name
        );
        let doc_lt = format!(
            "Updates query to return all ents where {} is less than given value",
            name
        );
        let doc_gt = format!(
            "Updates query to return all ents where {} is greater than given value",
            name
        );

        struct_setters.push(quote! {
            #[doc = #doc_eq]
            pub fn #name_eq(self, value: #ty) -> Self {
                Self(self.0.chain(#root::Condition::Field(
                    ::std::string::String::from(stringify!(#name)),
                    #root::FieldCondition::Value(
                        #root::ValueCondition::EqualTo(
                            #root::Value::from(value),
                        ),
                    ),
                )))
            }

            #[doc = #doc_lt]
            pub fn #name_lt(self, value: #ty) -> Self {
                Self(self.0.chain(#root::Condition::Field(
                    ::std::string::String::from(stringify!(#name)),
                    #root::FieldCondition::Value(
                        #root::ValueCondition::LessThan(
                            #root::Value::from(value),
                        ),
                    ),
                )))
            }

            #[doc = #doc_gt]
            pub fn #name_gt(self, value: #ty) -> Self {
                Self(self.0.chain(#root::Condition::Field(
                    ::std::string::String::from(stringify!(#name)),
                    #root::FieldCondition::Value(
                        #root::ValueCondition::GreaterThan(
                            #root::Value::from(value),
                        ),
                    ),
                )))
            }
        });
    }

    let default_doc_str = format!("Creates new query that selects all {} by default", name);

    Ok(quote! {
        #vis struct #query_name(#root::Query);

        impl ::std::convert::From<#query_name> for #root::Query {
            fn from(q: #query_name) -> Self {
                q.0
            }
        }

        impl ::std::default::Default for #query_name {
            #[doc = #default_doc_str]
            fn default() -> Self {
                Self(#root::Query::new(
                    #root::Condition::HasType(
                        ::std::string::String::from(#const_type_name),
                    )
                ))
            }
        }

        impl #query_name {
            #(#struct_setters)*
        }
    })
}