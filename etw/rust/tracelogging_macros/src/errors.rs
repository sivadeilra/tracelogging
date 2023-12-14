// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro2::*;
use quote::ToTokens;
use syn::Error;

pub struct Errors {
    error_tokens: Option<Error>,
}

impl Errors {
    pub const fn new() -> Self {
        Self { error_tokens: None }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.error_tokens.is_none()
    }

    pub fn check(self) -> syn::Result<()> {
        if let Some(e) = self.error_tokens {
            Err(e)
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn into_items(self) -> TokenStream {
        if let Some(e) = self.error_tokens {
            e.to_compile_error().into_token_stream()
        } else {
            TokenStream::new()
        }
    }

    pub fn add(&mut self, pos: Span, error_message: &str) {
        self.push(Error::new(pos, error_message));
    }

    pub fn push(&mut self, e: Error) {
        if let Some(ref mut existing) = self.error_tokens {
            existing.combine(e);
        } else {
            self.error_tokens = Some(e);
        }
    }
}
