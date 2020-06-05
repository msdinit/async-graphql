use crate::args;
use crate::utils::{check_reserved_name, get_crate_name, get_rustdoc};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{Data, DeriveInput, Error, FnArg, ImplItem, ItemImpl, Pat, Result, Type, TypeReference};

pub fn generate(directive_args: &args::Directive, input: &mut DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(directive_args.internal);
    let ident = &input.ident;
    let generics = &input.generics;
    let where_clause = &generics.where_clause;
    let gql_typename = directive_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());
    check_reserved_name(&gql_typename, directive_args.internal)?;

    let desc = directive_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&input.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let s = match &mut input.data {
        Data::Struct(e) => e,
        _ => return Err(Error::new_spanned(input, "It should be a struct")),
    };

    let expanded = quote! {
        #input
    };
    println!("{}", expanded);
    Ok(expanded.into())
}
