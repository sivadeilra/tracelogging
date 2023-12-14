// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::field_option::FieldOption;
use proc_macro2::*;
use syn::Expr;

pub struct FieldInfo {
    pub span: Span,
    pub type_name_span: Span,
    pub option: &'static FieldOption,
    pub name: String,
    pub value_tokens: Option<Expr>,  // Context is type_name_span.
    pub intype_tokens: Option<Expr>, // Context is type_name_span. If empty use option.intype.
    pub outtype_or_field_count_expr: Option<Expr>, // If empty, use outtype_or_field_count_int
    pub outtype_or_field_count_int: u8, // Use only if outtype_or_field_count_expr is empty
    pub tag: Option<Expr>,
}
