// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro2::*;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

use crate::errors::Errors;
use crate::guid::Guid;
use crate::parser::{next_arg, ArgResult};

pub struct ProviderInfo {
    pub symbol: Ident,
    pub name: String,
    pub id: Guid,
    pub group_id: Option<Guid>,
}

impl Parse for ProviderInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::try_from_tokens(input)
    }
}

impl ProviderInfo {
    pub fn try_from_tokens(root_parser: ParseStream) -> syn::Result<ProviderInfo> {
        let mut prov_id_set = false;
        let mut group_name_set = false;
        let mut errors = Errors::new();

        // symbol name

        // "expected identifier for provider symbol, e.g. MY_PROVIDER",
        let symbol: Ident = root_parser.parse()?;
        root_parser.parse::<Token![,]>()?;

        // provider name

        // "expected string literal for provider name, e.g. define_provider!(MY_PROVIDER, \"MyCompany.MyComponent\")",
        let prov_name_lit: LitStr = root_parser.parse()?;
        let prov_name = prov_name_lit.value();

        if prov_name.len() >= 32768 {
            errors.add(
                prov_name_lit.span(),
                "provider name.len() must be less than 32KB",
            );
        }
        if prov_name.contains('\0') {
            errors.add(prov_name_lit.span(), "provider name must not contain '\\0'");
        }

        let mut prov = ProviderInfo {
            name: prov_name,
            id: Guid::zero(),
            group_id: None,
            symbol,
        };

        if root_parser.peek(Token![,]) {
            root_parser.parse::<Token![,]>()?;
        }

        // provider options (id or group_id)

        while let ArgResult::Option(option_name_ident, option_args_parser) =
            next_arg(root_parser, false)?
        {
            let id_dest = match option_name_ident.to_string().as_str() {
                "debug" => {
                    continue;
                }
                "id" => {
                    if prov_id_set {
                        errors.add(option_name_ident.span(), "id already set");
                    }
                    prov_id_set = true;
                    &mut prov.id
                }
                "group_id" | "groupid" => {
                    if prov.group_id.is_some() {
                        errors.add(option_name_ident.span(), "group_id already set");
                    }
                    prov.group_id.insert(Guid::zero())
                }
                "group_name" | "groupname" => {
                    if group_name_set {
                        errors.add(option_name_ident.span(), "group_name already set");
                    }

                    // "expected \"groupname\""
                    let group_name_lit: LitStr = option_args_parser.parse()?;
                    let id_str = group_name_lit.value();

                    for ch in id_str.chars() {
                        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() {
                            errors.add(group_name_lit.span(), "group_name must contain only lowercase ASCII letters and ASCII digits");
                            break;
                        }
                    }
                    group_name_set = !id_str.is_empty();

                    continue;
                }
                _ => {
                    errors.add(
                        option_name_ident.span(),
                        "expected id(\"GUID\") or group_id(\"GUID\")",
                    );
                    continue;
                }
            };

            const EXPECTED_GUID: &str =
                "expected \"GUID\", e.g. \"20cf46dd-3b90-476c-94e9-4e74bbc30e31\"";
            let id_lit: LitStr = option_args_parser.parse()?;
            let id_str = id_lit.value();
            if let Some(id_val) = Guid::try_parse(&id_str) {
                *id_dest = id_val;
            } else {
                errors.add(id_lit.span(), EXPECTED_GUID);
            }
        }

        if !prov_id_set {
            prov.id = Guid::from_name(&prov.name);
        }

        errors.check()?;
        Ok(prov)
    }
}
