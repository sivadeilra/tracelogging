// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::provider_info::ProviderInfo;
use proc_macro2::*;
use quote::quote;

pub struct ProviderGenerator {}

impl ProviderGenerator {
    pub fn generate(provider: ProviderInfo) -> TokenStream {
        // Reserve space for size.
        let mut meta = Vec::<u8>::new();
        meta.push(0);
        meta.push(0);

        // Provider name + NUL
        meta.extend_from_slice(provider.name.as_bytes());
        meta.push(0);

        if let Some(ref group_id) = provider.group_id {
            // Provider group id
            meta.push(19); // size is 19: sizeof(size) + sizeof(type) + sizeof(guid) = 2 + 1 + 16
            meta.push(0);
            meta.push(1); // EtwProviderTraitTypeGroup
            meta.extend_from_slice(&group_id.to_bytes_le());
        }

        meta[0] = meta.len() as u8;
        meta[1] = (meta.len() >> 8) as u8;

        let provider_symbol = &provider.symbol;
        let id_fields = provider.id.to_fields();
        let id_fields_0: u32 = id_fields.0;
        let id_fields_1: u16 = id_fields.1;
        let id_fields_2: u16 = id_fields.2;
        let id_fields_3: [u8; 8] = id_fields.3;

        quote! {
            static #provider_symbol: ::tracelogging::Provider =
                ::tracelogging::_internal::provider_new(
                    &[ #(#meta),* ],
                    &tracelogging::Guid::from_fields(
                        #id_fields_0,
                        #id_fields_1,
                        #id_fields_2,
                        [#(#id_fields_3),*],
                    )
                );
        }
    }
}
