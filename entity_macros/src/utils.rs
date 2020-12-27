use std::collections::HashMap;
use syn::{
    parse_quote, spanned::Spanned, Attribute, Data, DeriveInput, Expr, Fields, FieldsNamed,
    GenericArgument, Ident, Lit, Meta, NestedMeta, PathArguments, PathSegment, Type,
};

/// Returns true if the attribute is in the form of ent(...) where
/// the interior is checked for an identifier of the given str
pub fn has_ent_attr(attrs: &[Attribute], ident_str: &str) -> bool {
    attrs
        .iter()
        .filter_map(|a| a.parse_meta().ok())
        .any(|m| match m {
            Meta::List(x) if x.path.is_ident("ent") => x.nested.iter().any(|m| match m {
                NestedMeta::Meta(x) => match x {
                    Meta::Path(x) => x.is_ident(ident_str),
                    _ => false,
                },
                _ => false,
            }),
            _ => false,
        })
}

/// Returns a map of inner attributes for <ROOT>(...) where
/// the the map's keys are the identifiers as strings and the values are
/// true/false for whether the attribute was <NAME> or no_<NAME>
pub fn attrs_into_attr_map(attrs: &[Attribute], root: &str) -> Option<HashMap<String, bool>> {
    attrs
        .iter()
        .filter_map(|a| a.parse_meta().ok())
        .find_map(|m| match m {
            Meta::List(x) if x.path.is_ident(root) => {
                Some(nested_meta_iter_into_attr_map(x.nested.iter()))
            }
            Meta::Path(x) if x.is_ident(root) => Some(HashMap::new()),
            Meta::NameValue(x) if x.path.is_ident(root) => Some(HashMap::new()),
            _ => None,
        })
}

pub fn nested_meta_iter_into_attr_map<'a, I: Iterator<Item = &'a NestedMeta>>(
    it: I,
) -> HashMap<String, bool> {
    it.filter_map(|m| match m {
        NestedMeta::Meta(x) => match x {
            Meta::Path(x) => x.segments.last().map(|s| s.ident.to_string()).map(|s| {
                let is_no = s.starts_with("no_");
                (s, !is_no)
            }),
            _ => None,
        },
        _ => None,
    })
    .collect()
}

pub fn nested_meta_iter_into_named_attr_map<'a, I: Iterator<Item = &'a NestedMeta>>(
    it: I,
) -> HashMap<String, Option<String>> {
    it.filter_map(|m| match m {
        NestedMeta::Meta(x) => match x {
            Meta::Path(x) => x
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .map(|s| (s, None)),
            Meta::NameValue(x) => x
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .map(|s| {
                    (
                        s,
                        Some(match &x.lit {
                            Lit::Str(x) => x.value(),
                            Lit::ByteStr(x) => String::from_utf8_lossy(&x.value()).to_string(),
                            Lit::Byte(x) => x.value().to_string(),
                            Lit::Char(x) => x.value().to_string(),
                            Lit::Int(x) => x.base10_digits().to_string(),
                            Lit::Float(x) => x.base10_digits().to_string(),
                            Lit::Bool(x) => x.value.to_string(),
                            Lit::Verbatim(x) => x.to_string(),
                        }),
                    )
                }),
            _ => None,
        },
        _ => None,
    })
    .collect()
}

/// Extracts and returns the named fields from the input, if possible
pub fn get_named_fields(input: &DeriveInput) -> Result<&FieldsNamed, syn::Error> {
    match &input.data {
        Data::Struct(x) => match &x.fields {
            Fields::Named(x) => Ok(x),
            _ => Err(syn::Error::new(input.span(), "Expected named fields")),
        },
        _ => Err(syn::Error::new(input.span(), "Expected struct")),
    }
}

/// Transforms some value with the given name (ident) to the specified type,
/// producing an expression
pub fn convert_from_value(name: &Ident, ty: &Type) -> Expr {
    if let Ok(inner_ty) = strip_option(ty) {
        parse_quote! {
            #name.try_into_option::<#inner_ty>()
        }
    } else {
        parse_quote! {
            {
                use ::std::convert::TryInto;
                #name.try_into()
            }
        }
    }
}

/// Returns true if given type appears to be any of the following:
/// * [`std::collections::HashMap`]
/// * [`std::collections::BTreeMap`]
pub fn is_map_type(input: &Type) -> bool {
    type_to_ident(input)
        .map(|ident| {
            matches!(
                ident.to_string().to_lowercase().as_str(),
                "hashmap" | "btreemap"
            )
        })
        .unwrap_or_default()
}

/// Returns ident of a type if it is a type path
///
/// * `path::to::MyType` -> Some(`MyType`)
/// * `MyType` -> Some(`MyType`)
/// * `MyType<String>` -> Some(`MyType`)
pub fn type_to_ident(input: &Type) -> Option<&Ident> {
    match input {
        Type::Path(x) => match x.path.segments.last() {
            Some(x) => Some(&x.ident),
            _ => None,
        },
        _ => None,
    }
}

/// If given a type of Option<T>, will strip the outer type and return
/// a reference to type of T, returning an error if anything else
pub fn strip_option(input: &Type) -> Result<&Type, syn::Error> {
    strip_for_type_str(input, "Option")
}

/// If given a type of Vec<T>, will strip the outer type and return
/// a reference to type of T, returning an error if anything else
pub fn strip_vec(input: &Type) -> Result<&Type, syn::Error> {
    strip_for_type_str(input, "Vec")
}

fn strip_for_type_str<'a, 'b>(input: &'a Type, ty_str: &'b str) -> Result<&'a Type, syn::Error> {
    match input {
        Type::Path(x) => match x.path.segments.last() {
            Some(x) if x.ident.to_string().to_lowercase() == ty_str.to_lowercase() => {
                match &x.arguments {
                    PathArguments::AngleBracketed(x) if x.args.len() == 1 => {
                        match x.args.last().unwrap() {
                            GenericArgument::Type(x) => Ok(x),
                            _ => Err(syn::Error::new(
                                x.span(),
                                format!("Unexpected type argument for {}", ty_str),
                            )),
                        }
                    }
                    PathArguments::AngleBracketed(_) => Err(syn::Error::new(
                        x.span(),
                        format!("Unexpected number of type parameters for {}", ty_str),
                    )),
                    PathArguments::Parenthesized(_) => Err(syn::Error::new(
                        x.span(),
                        format!("Unexpected {}(...) instead of {}<...>", ty_str, ty_str),
                    )),
                    PathArguments::None => Err(syn::Error::new(
                        x.span(),
                        format!("{} missing generic parameter", ty_str),
                    )),
                }
            }
            Some(x) => Err(syn::Error::new(
                x.span(),
                format!("Type is not {}<...>", ty_str),
            )),
            None => Err(syn::Error::new(x.span(), "Expected type to have a path")),
        },
        x => Err(syn::Error::new(x.span(), "Expected type to be a path")),
    }
}

/// Retrieves the inner type from a path segment, returning a reference to
/// the type at the position if available, or returning an error
pub fn get_inner_type_from_segment(
    seg: &PathSegment,
    pos: usize,
    max_supported: usize,
) -> Result<&Type, syn::Error> {
    match &seg.arguments {
        PathArguments::AngleBracketed(x) => {
            if x.args.len() <= max_supported && x.args.len() > pos {
                match x.args.iter().nth(pos).unwrap() {
                    GenericArgument::Type(x) => Ok(x),
                    _ => Err(syn::Error::new(seg.span(), "Unexpected type argument")),
                }
            } else {
                Err(syn::Error::new(seg.span(), "Invalid total type arguments"))
            }
        }
        _ => Err(syn::Error::new(seg.span(), "Unsupported type")),
    }
}