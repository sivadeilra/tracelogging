// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro2::*;
use quote::ToTokens;

use crate::enums::OutType;
use crate::errors::Errors;
use crate::field_info::FieldInfo;
use crate::field_option::FieldStrategy;
use crate::field_options::FIELD_OPTIONS;
use crate::parser::{eat_comma_opt, next_arg, ArgResult};
use crate::strings::*;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, parse_quote_spanned, Expr, LitStr, Token};

const METADATA_BYTES_MAX: u16 = u16::MAX; // TraceLogging limit
const STRUCT_FIELDS_MAX: u8 = 127; // TraceLogging limit
const DATA_DESC_MAX: u8 = 128; // EventWrite limit
const FIELDS_MAX: usize = 128; // TDH limit

pub struct EventInfo {
    pub provider_symbol: Ident,
    pub name: String,
    pub id_tokens: Option<Expr>,
    pub version_tokens: Option<Expr>,
    pub channel_tokens: Option<Expr>,
    pub opcode_tokens: Option<Expr>,
    pub task_tokens: Option<Expr>,
    pub level: Option<Expr>,
    pub keywords: Vec<Expr>,
    pub tag: Option<Expr>,
    pub activity_id: Option<Expr>,
    pub related_id: Option<Expr>,
    pub fields: Vec<FieldInfo>,

    // Set to 0 if we've already emitted an error message.
    data_desc_used: u8,

    // Set to 0 if we've already emitted an error message.
    // Accurate except that we assume all structs have at least one field and all tags
    // require 4 bytes.
    estimated_metadata_bytes_used: u16,
}

impl Parse for EventInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let level: Option<Expr> = None;
        let keywords: Vec<Expr> = Vec::new();
        let tag: Option<Expr> = None;
        let activity_id: Option<Expr> = None;
        let related_id: Option<Expr> = None;
        let fields: Vec<FieldInfo> = Vec::new();
        let data_desc_used: u8 = 2; // provider_meta, event_meta
        let mut estimated_metadata_bytes_used: u16 = 2 + 4; // metadata_size + estimated event tag size

        let mut errors = Errors::new();

        /*
        #[cfg(debug_assertions)]
        for i in 1..FIELD_OPTIONS.len() {
            debug_assert!(
                FIELD_OPTIONS[i - 1]
                    .option_name
                    .lt(FIELD_OPTIONS[i].option_name),
                "{} <=> {}",
                FIELD_OPTIONS[i - 1].option_name,
                FIELD_OPTIONS[i].option_name
            );
        }
        */

        // provider

        let provider_symbol: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // event name
        // "expected string literal for event name, e.g. write_event!(PROVIDER, \"EventName\", ...)",
        let event_name: LitStr = input.parse()?;
        let name = event_name.value();
        add_estimated_metadata(
            &mut estimated_metadata_bytes_used,
            &mut errors,
            event_name.span(),
            name.len() + 1,
        );

        eat_comma_opt(input);

        if name.contains('\0') {
            errors.add(event_name.span(), "event name must not contain '\\0'");
        }

        let mut event = EventInfo {
            provider_symbol,
            name,
            id_tokens: None,
            version_tokens: None,
            channel_tokens: None,
            opcode_tokens: None,
            task_tokens: None,
            level,
            keywords,
            tag,
            activity_id,
            related_id,
            fields,
            data_desc_used,
            estimated_metadata_bytes_used,
        };

        // options

        event.parse_event_options(&mut errors, input, false)?;

        // Set defaults for optional values

        // id default: 0
        if event.id_tokens.is_none() {
            event.id_tokens = Some(parse_quote!(0u16));
        }

        // version default: 0
        if event.version_tokens.is_none() {
            event.version_tokens = Some(parse_quote!(0u8));
        }

        // channel default: Channel::TraceLogging
        if event.channel_tokens.is_none() {
            event.channel_tokens = Some(parse_quote!(::tracelogging::Channel::TraceLogging));
        }

        // level default: Level::Verbose
        if event.level.is_none() {
            event.level = Some(parse_quote!(::tracelogging::Level::Verbose));
        }

        // opcode default: Opcode::Info
        if event.opcode_tokens.is_none() {
            event.opcode_tokens = Some(parse_quote!(::tracelogging::Opcode::Info));
        }

        // task default: 0
        if event.task_tokens.is_none() {
            event.task_tokens = Some(parse_quote!(0u16));
        }

        // keyword default: 1u64
        if event.keywords.is_empty() {
            event.keywords.push(parse_quote!(1));
        }

        // tag default: 0
        if event.tag.is_none() {
            event.tag = Some(parse_quote!(0));
        }

        // Done.

        errors.check()?;

        Ok(event)
    }
}

impl EventInfo {
    /// Parses options. Returns the number of logical fields added to the event.
    fn parse_event_options(
        &mut self,
        errors: &mut Errors,
        parent_parser: ParseStream,
        in_struct: bool,
    ) -> syn::Result<u8> {
        let mut logical_fields_added: u8 = 0;

        while !parent_parser.is_empty() {
            let option_ident: Ident = parent_parser.parse()?;
            let option_name = option_ident.to_string();

            let option_parser = {
                let la = parent_parser.lookahead1();
                if la.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in parent_parser);
                    content
                } else if la.peek(syn::token::Brace) {
                    let content;
                    syn::braced!(content in parent_parser);
                    content
                } else {
                    errors.push(la.error());
                    break;
                }
            };

            let field_span = option_ident.span();

            if let Ok(field_option_index) =
                FIELD_OPTIONS.binary_search_by(|o| o.option_name.cmp(&option_name))
            {
                let mut field = FieldInfo {
                    span: field_span,
                    type_name_span: option_ident.span(),
                    option: &FIELD_OPTIONS[field_option_index],
                    name: String::new(),
                    value_tokens: None,
                    intype_tokens: None,
                    outtype_or_field_count_expr: None,
                    outtype_or_field_count_int: FIELD_OPTIONS[field_option_index].outtype as u8,
                    tag: None,
                };

                let field_has_metadata = field.option.strategy.has_metadata();

                if !field_has_metadata {
                    // No metadata, so don't try to parse a field name.
                } else {
                    // "expected field name (must be a string literal, e.g. \"field name\")",
                    let field_name_lit: LitStr = option_parser.parse()?;
                    let field_name = field_name_lit.value();

                    eat_comma_opt(&option_parser);

                    field.name = field_name;
                    if field.name.contains('\0') {
                        errors.add(field_name_lit.span(), "field name must not contain '\\0'");
                    }
                }

                let field_accepts_tag;
                let field_accepts_format;
                let field_wants_struct;

                match field.option.strategy {
                    FieldStrategy::Scalar
                    | FieldStrategy::SystemTime
                    | FieldStrategy::Time32
                    | FieldStrategy::Time64
                    | FieldStrategy::Sid
                    | FieldStrategy::CStr
                    | FieldStrategy::Counted
                    | FieldStrategy::Slice => {
                        field_accepts_tag = true;
                        field_accepts_format = true;
                        field_wants_struct = false;
                    }
                    FieldStrategy::Struct => {
                        field_accepts_tag = true;
                        field_accepts_format = false;
                        field_wants_struct = true;
                    }
                    FieldStrategy::RawField
                    | FieldStrategy::RawFieldSlice
                    | FieldStrategy::RawMeta
                    | FieldStrategy::RawMetaSlice => {
                        field_accepts_tag = true;
                        field_accepts_format = true;
                        field_wants_struct = false;

                        // &expected_enum_message("InType", "Bool32", 13),
                        field.intype_tokens = Some(filter_enum_tokens(
                            option_parser.parse()?,
                            Ident::new("InType", Span::call_site()),
                            INTYPE_ENUMS,
                        )?);
                    }
                    FieldStrategy::RawStruct | FieldStrategy::RawStructSlice => {
                        field_accepts_tag = true;
                        field_accepts_format = false;
                        field_wants_struct = false;

                        if in_struct {
                            errors.add(option_ident.span(), "RawStruct not allowed within Struct");
                        }

                        let tokens: Expr = option_parser.parse()?;
                        // .next_tokens(Required, "expected struct field count value, e.g. 2");
                        field.outtype_or_field_count_expr = Some(parse_quote_spanned! {
                            option_ident.span() =>
                            ::tracelogging::OutType::from_int(#tokens)
                        });
                    }

                    FieldStrategy::RawData => {
                        field_accepts_tag = false;
                        field_accepts_format = false;
                        field_wants_struct = false;
                    }
                }

                if field.option.strategy.data_count() != 0 {
                    field.value_tokens = Some(option_parser.parse()?);
                    eat_comma_opt(&option_parser);
                    // "expected field value");
                }

                loop {
                    match next_arg(&option_parser, field_wants_struct)? {
                        ArgResult::None => {
                            self.push_field(errors, field);
                            break;
                        }
                        ArgResult::Struct(mut struct_parser) => {
                            let struct_index = self.fields.len();

                            field.outtype_or_field_count_int = 1; // For metadata estimate, assume fields present.
                            self.push_field(errors, field);

                            let field_count =
                                self.parse_event_options(errors, &mut struct_parser, true)?;
                            self.fields[struct_index].outtype_or_field_count_int =
                                field_count & OutType::TypeMask;
                            break;
                        }
                        ArgResult::Option(field_option_ident, field_option_parser) => {
                            let field_option_name = field_option_ident.to_string();

                            match field_option_name.as_str() {
                                "tag" if field_accepts_tag => {
                                    if field.tag.is_some() {
                                        errors.add(field_option_ident.span(), "tag already set");
                                    }
                                    // "expected Tag value, e.g. 1 or 0x0FF00000",
                                    field.tag = Some(field_option_parser.parse()?);
                                }
                                "format" if field_accepts_format => {
                                    if field.outtype_or_field_count_expr.is_some() {
                                        errors.add(field_option_ident.span(), "format already set");
                                    }
                                    field.outtype_or_field_count_expr = Some(filter_enum_tokens(
                                        field_option_parser.parse()?,
                                        // &expected_enum_message("OutType", "String", 2),
                                        Ident::new("OutType", Span::call_site()),
                                        OUTTYPE_ENUMS,
                                    )?);
                                }
                                _ => {
                                    errors.add(field_option_ident.span(), "unrecognized option");
                                }
                            }
                        }

                        ArgResult::OptionStruct(_) => {
                            errors.add(option_parser.span(), "unrecognized option");
                        }
                    }
                }

                if field_has_metadata {
                    if in_struct && logical_fields_added == STRUCT_FIELDS_MAX {
                        errors.add(option_ident.span(), "too many fields in struct (limit 127)");
                    }

                    logical_fields_added = logical_fields_added.saturating_add(1);
                }
            } else {
                match option_name.as_str() {
                    "debug" if !in_struct => {
                        // ignored
                        continue;
                    }
                    "id_version" if !in_struct => {
                        if self.id_tokens.is_some() {
                            errors.add(option_ident.span(), "id_version already set");
                        }

                        // "expected Id value, e.g. 1 or 0x200F"
                        self.id_tokens = Some(option_parser.parse()?);
                        option_parser.parse::<Token![,]>()?;
                        // "expected Version value, e.g. 0 or 0x1F"
                        self.version_tokens = Some(option_parser.parse()?);
                    }
                    "channel" if !in_struct => {
                        if self.channel_tokens.is_some() {
                            errors.add(option_ident.span(), "channel already set");
                        }
                        self.channel_tokens = Some(filter_enum_tokens(
                            // &expected_enum_message("Channel", "TraceLogging", 11),
                            option_parser.parse()?,
                            Ident::new("Channel", Span::call_site()),
                            CHANNEL_ENUMS,
                        )?);
                    }
                    "level" if !in_struct => {
                        if self.level.is_some() {
                            errors.add(option_ident.span(), "level already set");
                        }
                        // &expected_enum_message("Level", "Verbose", 5),
                        self.level = Some(filter_enum_tokens(
                            option_parser.parse()?,
                            Ident::new("Level", Span::call_site()),
                            LEVEL_ENUMS,
                        )?);
                    }
                    "opcode" if !in_struct => {
                        if self.opcode_tokens.is_some() {
                            errors.add(option_ident.span(), "opcode already set");
                        }
                        self.opcode_tokens = Some(filter_enum_tokens(
                            // &expected_enum_message("Opcode", "Info", 0),
                            option_parser.parse()?,
                            Ident::new("Opcode", Span::call_site()),
                            OPCODE_ENUMS,
                        )?);
                    }
                    "task" if !in_struct => {
                        if self.task_tokens.is_some() {
                            errors.add(option_ident.span(), "task already set");
                        }
                        // "expected Task value, e.g. 1 or 0x2001");
                        self.task_tokens = Some(option_parser.parse()?);
                    }
                    "keyword" if !in_struct => {
                        // "expected Keyword value, e.g. 0x100F",
                        self.keywords.push(option_parser.parse()?);
                    }
                    "tag" if !in_struct => {
                        if self.tag.is_some() {
                            errors.add(option_ident.span(), "tag already set");
                        }
                        // "expected Tag value, e.g. 1 or 0x0FF00000",
                        self.tag = Some(option_parser.parse()?);
                    }
                    "activity_id" if !in_struct => {
                        if self.activity_id.is_some() {
                            errors.add(option_ident.span(), "activity_id already set");
                        }
                        // "expected Activity Id variable"
                        self.activity_id = Some(option_parser.parse()?);
                    }
                    "related_id" if !in_struct => {
                        if self.related_id.is_some() {
                            errors.add(option_ident.span(), "related_id already set");
                        }
                        // "expected Related Id variable"
                        self.related_id = Some(option_parser.parse()?);
                    }
                    _ => {
                        errors.add(option_ident.span(), "unrecognized option");
                        continue;
                    }
                }
            }

            eat_comma_opt(parent_parser);
        }

        Ok(logical_fields_added)
    }

    fn push_field(&mut self, errors: &mut Errors, field: FieldInfo) {
        let metadata_size = field.name.len()
            + 1 // name nul-termination
            + if field.tag.is_some() {
                6 // intype + outtype + tag
            } else if field.outtype_or_field_count_int != 0 {
                2 // intype + outtype
            } else {
                1 // intype
            };
        add_estimated_metadata(
            &mut self.estimated_metadata_bytes_used,
            errors,
            field.type_name_span,
            metadata_size,
        );
        self.add_data_desc_used(
            errors,
            field.type_name_span,
            field.option.strategy.data_count(),
        );

        if self.fields.len() == FIELDS_MAX {
            errors.add(
                field.type_name_span,
                "event has too many fields (limit is 128 fields)",
            );
        }

        self.fields.push(field);
    }

    fn add_data_desc_used(&mut self, errors: &mut Errors, span: Span, data_count: u8) {
        if self.data_desc_used == 0 {
            // Already emitted an error for this. Don't emit another.
        } else if DATA_DESC_MAX - self.data_desc_used >= data_count {
            self.data_desc_used += data_count;
        } else {
            self.data_desc_used = 0; // Don't give any additional size errors.
            errors.add(
                span,
                "event has too many blocks of data (1 block per fixed-length field, 2 blocks per variable-length field; limit is 128 blocks)");
        }
    }
}

fn add_estimated_metadata(
    estimated_metadata_bytes_used: &mut u16,
    errors: &mut Errors,
    span: Span,
    size: usize,
) {
    if *estimated_metadata_bytes_used == 0 {
        // Already emitted an error for this. Don't emit another.
    } else if (METADATA_BYTES_MAX - *estimated_metadata_bytes_used) as usize >= size {
        *estimated_metadata_bytes_used += size as u16;
    } else {
        *estimated_metadata_bytes_used = 0; // Don't give any additional size errors.
        errors.add(
            span,
            "event metadata is too large (includes event name string, field name strings, and field type codes; limit is 65535 bytes)");
    }
}

#[allow(dead_code)]
fn expected_enum_message(
    enum_name: &str,
    suggested_string_value: &str,
    suggested_integer_value: u8,
) -> String {
    return format!(
        "expected {0} value, e.g. {1}, tracelogging::{0}::{1}, or {2}",
        enum_name, suggested_string_value, suggested_integer_value,
    );
}

fn filter_enum_tokens(tokens: Expr, enum_name: Ident, known_values: &[&str]) -> syn::Result<Expr> {
    /*#[cfg(debug_assertions)]
    for i in 1..known_values.len() {
        debug_assert!(known_values[i - 1] < known_values[i]);
    }
    */

    let str = tokens.to_token_stream().to_string();
    if !str.is_empty() && str.as_bytes()[0].is_ascii_digit() {
        // If it starts with a number, wrap it in from_int.
        Ok(parse_quote! {
            ::tracelogging::#enum_name::from_int(#tokens)
        })
    } else if known_values.binary_search(&str.as_str()).is_ok() {
        // If it's an unqualified known enum value, fully-qualify it.
        let str_id = Ident::new(&str, Span::call_site());
        Ok(parse_quote! {
            ::tracelogging::#enum_name::#str_id
        })
    } else {
        Ok(tokens)
    }
}
