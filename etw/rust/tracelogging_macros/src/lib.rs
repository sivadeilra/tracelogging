// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! Implements the macros that are exported by the tracelogging crate.

#![forbid(unsafe_code)]
#![allow(unused_imports)]

use proc_macro2::Span;

use crate::event_generator::EventGenerator;
use crate::event_info::EventInfo;
use crate::provider_generator::ProviderGenerator;
use crate::provider_info::ProviderInfo;

#[proc_macro]
pub fn define_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let prov = syn::parse_macro_input!(input as ProviderInfo);
    ProviderGenerator::generate(prov).into()
}

#[proc_macro]
pub fn write_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let event_info = syn::parse_macro_input!(input as EventInfo);
    EventGenerator::new(Span::call_site())
        .generate(event_info)
        .into()
}

// The tracelogging crate depends on the tracelogging_macros crate so the
// tracelogging_macros crate can't depend on the tracelogging crate. Instead, pull in
// the source code for needed modules.

#[allow(dead_code)]
mod guid;

mod enums;
mod errors;
mod event_generator;
mod event_info;
mod field_info;
mod field_option;
mod field_options;
mod ident_builder;
mod parser;
mod provider_generator;
mod provider_info;
mod strings;
mod tree;

#[cfg(test)]
mod tests;
