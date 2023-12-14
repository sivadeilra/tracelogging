// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro2::*;
use std::str;
use syn::parse::ParseBuffer;
use syn::parse::ParseStream;
use syn::Token;

/// Reads OptionIdent(ArgsGroup) or {...} then moves to the next comma or the end-of-stream.
/// Emits "expected option" errors for non-option syntax.
/// Emits "expected ..." error for other tokens encountered before comma or end-of-stream.
pub fn next_arg(input: ParseStream, want_struct: bool) -> syn::Result<ArgResult> {
    // const EXPECTED_OPTION: &str = "expected identifier for option name, e.g. Option(args...)";
    const EXPECTED_OPTION_OR_STRUCT: &str =
        "expected '{' for struct or identifier for option name, e.g. Option(args...)";
    // const EXPECTED_OPTION_ARGS: &str = "expected '(' after option name, e.g. Option(args...)";

    if input.is_empty() {
        if !want_struct {
            // Assume options are optional.
        } else {
            return Err(input.error(EXPECTED_OPTION_OR_STRUCT));
        }
        return Ok(ArgResult::None);
    }

    // Expect: OptionName or {}

    let la = input.lookahead1();
    let output = if la.peek(Token![struct]) {
        input.parse::<Token![struct]>()?;
        let inner;
        syn::parenthesized!(inner in input);
        ArgResult::OptionStruct(inner)
    } else if la.peek(syn::token::Brace) {
        let inner;
        syn::braced!(inner in input);
        ArgResult::Struct(inner)
    } else if la.peek(syn::Ident) {
        let option_ident: Ident = input.parse()?;
        let inner;
        syn::parenthesized!(inner in input);
        ArgResult::Option(option_ident, inner)
    } else {
        return Err(la.error());
    };

    // Consume a single comma after the option, if any.
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }

    Ok(output)
}

pub enum ArgResult<'a> {
    None,

    // This is foo( ... )
    Option(Ident, ParseBuffer<'a>),

    // This is struct( ... )
    OptionStruct(ParseBuffer<'a>),

    // This is { ... }
    Struct(ParseBuffer<'a>),
}

pub fn eat_comma_opt(input: ParseStream) {
    if input.peek(Token![,]) {
        let _ = input.parse::<Token![,]>();
    }
}

pub fn check_parse<T: syn::parse::Parse>(t: TokenStream) -> T {
    syn::parse2(t.clone()).unwrap_or_else(|e| {
        panic!("error parsing {}\n\nerror: {}", t, e);
    })
}
