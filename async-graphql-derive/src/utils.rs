use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::quote;
use syn::{Attribute, Error, Expr, Ident, Lit, Meta, MetaList, NestedMeta, Path, Result};

pub fn get_crate_name(internal: bool) -> TokenStream {
    if internal {
        quote! { crate }
    } else {
        let name = crate_name("async-graphql").unwrap_or_else(|_| "async_graphql".to_owned());
        let id = Ident::new(&name, Span::call_site());
        quote! { #id }
    }
}

pub fn check_reserved_name(name: &str, internal: bool) -> Result<()> {
    if internal {
        return Ok(());
    }
    if name.ends_with("Connection") {
        Err(Error::new(
            Span::call_site(),
            "The name ending with 'Connection' is reserved",
        ))
    } else if name == "PageInfo" {
        Err(Error::new(
            Span::call_site(),
            "The name 'PageInfo' is reserved",
        ))
    } else {
        Ok(())
    }
}

pub fn get_rustdoc(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut full_docs = String::new();
    for attr in attrs {
        match attr.parse_meta()? {
            Meta::NameValue(nv) if nv.path.is_ident("doc") => {
                if let Lit::Str(doc) = nv.lit {
                    let doc = doc.value();
                    let doc_str = doc.trim();
                    if !full_docs.is_empty() {
                        full_docs += "\n";
                    }
                    full_docs += doc_str;
                }
            }
            _ => {}
        }
    }
    Ok(if full_docs.is_empty() {
        None
    } else {
        Some(full_docs)
    })
}

pub fn parse_default(lit: &Lit) -> Result<TokenStream> {
    match lit {
        Lit::Str(value) =>{
            let value = value.value();
            Ok(quote!({ #value.to_string() }))
        }
        Lit::Int(value) => {
            let value = value.base10_parse::<i32>()?;
            Ok(quote!({ #value as i32 }))
        }
        Lit::Float(value) => {
            let value = value.base10_parse::<f64>()?;
            Ok(quote!({ #value as f64 }))
        }
        Lit::Bool(value) => {
            let value = value.value;
            Ok(quote!({ #value }))
        }
        _ => Err(Error::new_spanned(
            lit,
            "The default value type only be string, integer, float and boolean, other types should use default_with",
        )),
    }
}

pub fn parse_default_with(lit: &Lit) -> Result<TokenStream> {
    if let Lit::Str(str) = lit {
        let str = str.value();
        let tokens: TokenStream = str.parse()?;
        Ok(quote! { (#tokens) })
    } else {
        Err(Error::new_spanned(
            &lit,
            "Attribute 'default' should be a string.",
        ))
    }
}

pub fn feature_block(
    crate_name: &TokenStream,
    features: &[String],
    field_name: &str,
    block: TokenStream,
) -> TokenStream {
    if !features.is_empty() {
        let error_message = format!(
            "`{}` is only available if the features `{}` are enabled",
            field_name,
            features.join(",")
        );
        quote!({
            #[cfg(not(all(#(feature = #features),*)))]
            {
                return Err(#crate_name::FieldError::from(#error_message)).map_err(std::convert::Into::into);
            }
            #[cfg(all(#(feature = #features),*))]
            {
                #block
            }
        })
    } else {
        block
    }
}

pub fn remove_attr(attrs: &mut Vec<Attribute>, name: &str) {
    attrs.retain(|attr| !attr.path.is_ident(name));
}

pub fn parse_directives(meta_list: &MetaList) -> Result<Vec<(Path, TokenStream)>> {
    let mut directives = Vec::new();

    for meta in &meta_list.nested {
        if let NestedMeta::Meta(Meta::List(ls)) = meta {
            let ty = ls.path.clone();
            let mut params = Vec::new();

            for meta in &ls.nested {
                if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
                    let name = &nv.path;
                    if let Lit::Str(value) = &nv.lit {
                        let value_str = value.value();
                        let expr = syn::parse_str::<Expr>(&value_str)?;
                        params.push(quote! { #name: (#expr).into() });
                    } else {
                        return Err(Error::new_spanned(&nv.lit, "Value must be string literal"));
                    }
                } else {
                    return Err(Error::new_spanned(&meta, "Invalid directive."));
                }
            }
            directives.push((ty, quote! { #(#params),* }));
        } else {
            return Err(Error::new_spanned(meta, "Invalid directive."));
        }
    }

    Ok(directives)
}
