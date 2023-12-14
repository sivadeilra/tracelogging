// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro2::*;
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr, Type};

/// If array_count == 0: `identity::<&type_path>(value_tokens)`
///
/// If array_count != 0: `identity::<&[type_path; array_count]>(value_tokens)`
pub fn identity_call(type_path: Type, array_count: usize, value_tokens: Expr) -> Expr {
    if array_count != 0 {
        parse_quote! {
            core::convert::identity::<&[#type_path ; #array_count]>(#value_tokens)
        }
    } else {
        parse_quote! {
            core::convert::identity::<& #type_path >(#value_tokens)
        }
    }
}

/// Either `None` or `Some(borrow(tokens))`
pub fn borrowed_option_from_tokens<T: ToTokens>(opt: Option<&T>) -> Expr {
    if let Some(tokens) = opt {
        parse_quote!(Some(::core::borrow::Borrow::borrow(#tokens)))
    } else {
        parse_quote!(None)
    }
}

/// If array_count == 0: `type_path`
///
/// If array_count != 0: `[type_path; array_count]`
pub fn scalar_type_path(type_path: Type, array_count: usize) -> Type {
    if array_count == 0 {
        type_path
    } else {
        parse_quote!([#type_path; #array_count])
    }
}
