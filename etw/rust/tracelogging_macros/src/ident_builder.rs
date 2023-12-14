// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro2::Span;
use std::fmt::Write;
use syn::Ident;

pub struct IdentBuilder {
    ident: String,
    base_len: usize,
    span: Span,
}

impl IdentBuilder {
    pub fn new(base_name: &str, span: Span) -> IdentBuilder {
        let mut builder = Self {
            span,
            ident: String::with_capacity(base_name.len() + 4),
            base_len: base_name.len(),
        };

        builder.ident.push_str(base_name);

        return builder;
    }

    pub fn current(&self) -> Ident {
        Ident::new(&self.ident, self.span)
    }

    pub fn set_suffix(&mut self, suffix: usize) -> &str {
        self.ident.truncate(self.base_len);
        write!(self.ident, "{}", suffix).unwrap();
        return &self.ident;
    }
}
