// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::enums::{EnumToken, InType};
use crate::event_info::EventInfo;
use crate::field_info::FieldInfo;
use crate::field_option::{FieldOption, FieldStrategy};
use crate::ident_builder::IdentBuilder;
use crate::parser::check_parse;
use crate::strings::*;
use crate::tree::{borrowed_option_from_tokens, identity_call, scalar_type_path};
use core::mem::take;
use proc_macro2::*;
use quote::{quote, quote_spanned};
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Type;
use syn::{parse_quote, parse_quote_spanned, Expr};

pub struct EventGenerator {
    /// tokens for declaring the _TLG_TAGn constants.
    tags_tree: TokenStream,
    /// tokens in the _TlgMeta(...) type definition.
    meta_type_tree: TokenStream,
    /// tokens in the _TlgMeta(...) initializer.
    meta_init_tree: TokenStream,
    /// tokens in the _tlg_write(...) function signature.
    func_args_tree: TokenStream,
    /// tokens in the _tlg_write(...) function call.
    func_call_tree: TokenStream,
    /// tokens in the _tlg_lengths = [...] array initializer.
    lengths_init_tree: TokenStream,
    /// tokens in the EventDataDescriptor &[...] array initializer.
    data_desc_init_tree: TokenStream,
    /// Code that runs if the provider is enabled.
    enabled_tree: TokenStream,
    /// "_TLG_TAGn"
    tag_n: IdentBuilder,
    /// Buffered _TlgMeta bytes.
    meta_buffer: Vec<u8>,
    /// number of fields added so far
    field_count: u16,
    /// number of runtime lengths needed
    lengths_count: usize,
}

impl EventGenerator {
    pub fn new(span: Span) -> Self {
        Self {
            tags_tree: TokenStream::new(),
            meta_type_tree: TokenStream::new(),
            meta_init_tree: TokenStream::new(),
            func_args_tree: TokenStream::new(),
            func_call_tree: TokenStream::new(),
            lengths_init_tree: TokenStream::new(),
            data_desc_init_tree: TokenStream::new(),
            enabled_tree: TokenStream::new(),
            tag_n: IdentBuilder::new(TLG_TAG_CONST, span),
            meta_buffer: Vec::with_capacity(128),
            field_count: 0,
            lengths_count: 0,
        }
    }

    pub fn generate(&mut self, event: EventInfo) -> TokenStream {
        self.meta_buffer.clear();
        self.field_count = 0;
        self.lengths_count = 0;

        // Before-field stuff:

        // metadata size: u16 = size_of::<_TlgMeta>() as u16
        self.meta_type_tree.extend(quote!(u16,));
        self.meta_init_tree.extend(quote! {
            ::core::mem::size_of::<_TlgMeta>() as u16,
        });

        // event tag
        let tag_ident = Ident::new("_TLG_TAG", Span::call_site());
        self.add_tag(&tag_ident, event.tag.as_ref().unwrap());

        // event name
        self.meta_buffer.extend(event.name.as_bytes());
        self.meta_buffer.push(0);

        // data descriptors for provider metadata and event metadata
        // const EVENT_DATA_DESCRIPTOR_TYPE_PROVIDER_METADATA: u32 = 2;
        // const EVENT_DATA_DESCRIPTOR_TYPE_EVENT_METADATA: u32 = 1;
        self.data_desc_init_tree.extend(quote! {
            ::tracelogging::_internal::EventDataDescriptor::from_raw_bytes(
                _tlg_prov.raw_meta(),
                2, // EVENT_DATA_DESCRIPTOR_TYPE_PROVIDER_METADATA
            ),

            ::tracelogging::_internal::EventDataDescriptor::from_raw_bytes(
                _tlg_meta, 1, // EVENT_DATA_DESCRIPTOR_TYPE_EVENT_METADATA
            ),
        });

        // always-present args for the helper function's prototype
        self.func_args_tree.extend(quote! {
            _tlg_prov: &::tracelogging::Provider,
            _tlg_meta: &[u8],
            _tlg_desc: &::tracelogging::_internal::EventDescriptor,
            _tlg_aid: Option<&[u8; 16]>,
            _tlg_rid: Option<&[u8; 16]>,
        });

        // always-present args for the helper function's call site
        let provider_ident = &event.provider_symbol;
        let activity_id_tokens = borrowed_option_from_tokens(event.activity_id.as_ref());
        let related_id_tokens = borrowed_option_from_tokens(event.related_id.as_ref());
        self.func_call_tree.extend(quote! {
            &#provider_ident,
            ::tracelogging::_internal::meta_as_bytes(&_TLG_META),
            &_TLG_DESC,
            #activity_id_tokens,
            #related_id_tokens,
        });

        // Add the per-field stuff:

        for field in event.fields.iter() {
            self.add_field(field);
        }

        self.flush_meta_buffer();

        // code that runs if the provider is enabled:
        /*
        const _TLG_DESC = EventDescriptor::from_raw_parts(...);
        tags_tree...
        struct _TlgMeta(meta_type_tree...);
        const _TLG_META = _TlgMeta(meta_init_tree...);
        fn _tlg_write(func_args_tree...) -> u32 {
            let _tlg_lengths = [lengths_init_tree...];
            provider_write_transfer(prov, desc, aid, rid, &[data_desc_init_tree...]);
        }
        _tlg_write(func_call_tree)
        */

        let event_id_tokens = &event.id_tokens;
        let event_version_tokens = &event.version_tokens;
        let event_channel_tokens = &event.channel_tokens;
        let event_opcode_tokens = &event.opcode_tokens;
        let event_task_tokens = &event.task_tokens;

        let tags_tree = core::mem::take(&mut self.tags_tree);
        let meta_type_tree = take(&mut self.meta_type_tree);
        let meta_init_tree = take(&mut self.meta_init_tree);

        let func_args_tree = &self.func_args_tree;
        let lengths_init_tree = &self.lengths_init_tree;
        let lengths_count = self.lengths_count;
        let data_desc_init_tree = &self.data_desc_init_tree;
        let func_call_tree = &self.func_call_tree;

        check_parse::<Expr>(quote! {
            call_a_function(#func_call_tree)
        });

        check_parse::<Expr>(quote! {
            _TlgMeta(#meta_init_tree)
        });

        self.enabled_tree.extend(quote! {
            static _TLG_DESC: ::tracelogging::_internal::EventDescriptor =
                ::tracelogging::_internal::EventDescriptor::from_parts(
                    #event_id_tokens,
                    #event_version_tokens,
                    #event_channel_tokens,
                    _TLG_LEVEL,
                    #event_opcode_tokens,
                    #event_task_tokens,
                    _TLG_KEYWORD,
                );

            // const _TLG_TAG: u32 = EVENT_TAG; const _TLG_TAG3: u32 = FIELD3_TAG;
            #tags_tree
            #[repr(packed)]
            struct _TlgMeta(#meta_type_tree);

            static _TLG_META: _TlgMeta = _TlgMeta(#meta_init_tree);

            // Make a helper function and then call it. This does the following:
            // - Keep temporaries alive (this could also be done with a match expression).
            // - Give the optimizer the option to merge identical helpers.
            // fn _tlg_write(prov, meta, desc, aid, rid, args...) -> { ... }
            #[allow(clippy::too_many_arguments)]
            fn _tlg_write(#func_args_tree) -> u32 {
                let _tlg_lengths: [u16; #lengths_count] = [ #lengths_init_tree ];

                // provider_write_transfer(_tlg_prov, meta, &_TLG_DESC, activity_id, related_id, &[data...])
                ::tracelogging::_internal::provider_write_transfer(
                    _tlg_prov,
                    _tlg_desc,
                    _tlg_aid,
                    _tlg_rid,
                    &[#data_desc_init_tree]
                )
            }

            // _tlg_write(prov, meta, aid, rid, values...)
            _tlg_write(#func_call_tree)
        });

        // put it all together:
        /*
        const _TLG_KEYWORD = keywords...;
        const _TLG_LEVEL = level...;
        if(!tlg_prov_var.enabled(_TLG_LEVEL, _TLG_KEYWORD)) {
            0
        } else {
            enabled_tree...
        }
        */

        let mut event_tree = quote!(); // Alias tree2 to save a tree.

        // _TLG_KEYWORD
        if event.keywords.len() == 1 {
            // Generate simple output if only one keyword.
            // const _TLG_KEYWORD: u64 = KEYWORDS[0];
            let keyword = event.keywords.last().unwrap();
            event_tree.extend(quote! {
                const _TLG_KEYWORD: u64 = #keyword;
            });
        } else {
            // More-complex output needed in other cases.
            //
            // We have suboptimal results if we combine the subexpressions ourselves,
            // e.g. doing "const X = (KEYWORDS0) | (KEYWORDS1);"" would result in
            // suboptimal error reporting for syntax errors in the user-supplied
            // expressions as well as warnings for unnecessary parentheses. Instead,
            // evaluate the subexpressions separately then combine the resulting
            // constants. This works for any number of keywords.
            //
            // const _TLG_KEYWORD0: u64 = KEYWORDS0;
            // const _TLG_KEYWORD1: u64 = KEYWORDS1;
            // const _TLG_KEYWORD: u64 = 0u64 | _TLG_KEYWORD0 | _TLG_KEYWORD1;

            let mut keyword_tokens = TokenStream::new();

            for (i, keyword) in event.keywords.iter().enumerate() {
                let keyword_i = Ident::new(&format!("_TLG_KEYWORD_{i}"), keyword.span());
                event_tree.extend(quote!(const #keyword_i: u64 = #keyword;));
                keyword_tokens.extend(quote!(| #keyword_i));
            }

            // event_tree += "const _TLG_KEYWORD: u64 = 0u64 | _TLG_KEYWORD0 | _TLG_KEYWORD1;"
            event_tree.extend(quote! {
                const _TLG_KEYWORD: u64 = 0 #keyword_tokens;
            });
        }

        let event_level = &event.level;
        let enabled_tree = core::mem::take(&mut self.enabled_tree);
        event_tree.extend(quote! {
            const _TLG_LEVEL: ::tracelogging::Level = #event_level;

            if !#provider_ident.enabled(_TLG_LEVEL, _TLG_KEYWORD) {
                0
            } else {
                #enabled_tree
            }
        });

        // Wrap the event in "{...}":
        quote! {
            {
                #event_tree
            }
        }
    }

    fn add_field(&mut self, field: &FieldInfo) {
        // Metadata

        if field.option.strategy.has_metadata() {
            self.meta_buffer.extend(field.name.as_bytes());
            self.meta_buffer.push(0);

            let has_out = field.outtype_or_field_count_expr.is_some()
                || field.outtype_or_field_count_int != 0;
            let has_tag = field.tag.is_some();

            let inflags = (if has_out || has_tag { 0x80 } else { 0 })
                | (if field.option.strategy.is_slice() {
                    InType::VariableCountFlag
                } else {
                    0
                });
            self.add_typecode_meta(
                check_parse(quote!(::tracelogging::InType)),
                field.intype_tokens.as_ref(),
                field.option.intype.to_token(),
                inflags,
            );

            if has_out || has_tag {
                let outflags = if has_tag { 0x80 } else { 0 };
                self.add_typecode_meta(
                    check_parse(quote!(::tracelogging::OutType)),
                    field.outtype_or_field_count_expr.as_ref(),
                    EnumToken::U8(field.outtype_or_field_count_int),
                    outflags,
                );
            }

            if let Some(field_tag) = &field.tag {
                self.tag_n.set_suffix(self.field_count as usize);
                let tag_ident = self.tag_n.current();
                self.add_tag(&tag_ident, field_tag);
            }
        }

        // Data

        let mut arg_n = IdentBuilder::new(TLG_ARG_VAR, field.span);
        arg_n.set_suffix(self.field_count as usize);
        let arg_ident = arg_n.current();

        match field.option.strategy {
            FieldStrategy::Scalar => {
                // Prototype: , _tlg_argN: &value_type
                // Call site: , identity::<&value_type>(value_tokens...)
                self.add_func_scalar_arg(
                    field.span,
                    &arg_ident,
                    field.option,
                    identity_call(
                        field.option.value_type(),
                        field.option.value_array_count,
                        field.value_tokens.clone().unwrap(),
                    ),
                );

                // EventDataDescriptor::from_value(_tlg_argN),
                self.add_data_desc_for_arg_n(
                    &arg_ident,
                    quote_spanned!(field.span => ::tracelogging::_internal::EventDataDescriptor::from_value),
                );
            }

            FieldStrategy::Time32 | FieldStrategy::Time64 => {
                let filetime_from_time_path = if let FieldStrategy::Time64 = field.option.strategy {
                    quote_spanned!(field.span => ::tracelogging::_internal::filetime_from_time64)
                } else {
                    quote_spanned!(field.span => ::tracelogging::_internal::filetime_from_time32)
                };

                let field_value_tokens = &field.value_tokens.as_ref().unwrap();

                // Prototype: , _tlg_argN: &value_type
                // Call site: , &filetime_from_timeNN(value_tokens...)
                self.add_func_scalar_arg(
                    field.span,
                    &arg_ident,
                    field.option,
                    check_parse(quote_spanned! {
                        field.span =>
                        // Use filetime_from_timeNN(...) as a target for error messages.
                        & #filetime_from_time_path(#field_value_tokens)
                    }),
                );

                // EventDataDescriptor::from_value(_tlg_argN),
                self.add_data_desc_for_arg_n(
                    &arg_ident,
                    quote_spanned!(field.span => ::tracelogging::_internal::EventDataDescriptor::from_value),
                );
            }

            FieldStrategy::SystemTime => {
                let field_value_tokens = &field.value_tokens;
                // Use duration_since(...) as a target for error messages.
                // Prototype: , _tlg_argN: &i64
                // Call site: , &match SystemTime::duration_since(value_tokens, SystemTime::UNIX_EPOCH) { ... }
                self.add_func_scalar_arg(
                    field.span,
                    &arg_ident,
                    field.option,
                    parse_quote_spanned! {
                        field.span =>
                        &match ::std::time::SystemTime::duration_since(
                            #field_value_tokens,
                            ::std::time::SystemTime::UNIX_EPOCH
                        ) {
                            Ok(_tlg_dur) => ::tracelogging::_internal::filetime_from_duration_after_1970(_tlg_dur),
                            Err(_tlg_dur) => ::tracelogging::_internal::filetime_from_duration_before_1970(_tlg_dur.duration()),
                        }
                    }
                );

                // EventDataDescriptor::from_value(_tlg_argN),
                self.add_data_desc_for_arg_n(&arg_ident, DATADESC_FROM_VALUE_PATH());
            }

            FieldStrategy::RawData | FieldStrategy::RawField | FieldStrategy::RawFieldSlice => {
                // Prototype: , _tlg_argN: &[value_type]
                // Call site: , AsRef::<[value_type]>::as_ref(value_tokens...)
                self.add_func_slice_arg(
                    field.span,
                    &arg_ident,
                    field.option,
                    field.value_tokens.as_ref().unwrap(),
                );

                // EventDataDescriptor::from_counted(_tlg_argN),
                self.add_data_desc_for_arg_n(&arg_ident, DATADESC_FROM_COUNTED_PATH());
            }

            FieldStrategy::Sid => {
                // Prototype: , _tlg_argN: &[value_type]
                // Call site: , AsRef::<[value_type]>::as_ref(value_tokens...)
                self.add_func_slice_arg(
                    field.span,
                    &arg_ident,
                    field.option,
                    field.value_tokens.as_ref().unwrap(),
                );

                // EventDataDescriptor::from_sid(_tlg_argN),
                self.add_data_desc_for_arg_n(&arg_ident, DATADESC_FROM_SID_PATH());
            }

            FieldStrategy::CStr => {
                // Prototype: , _tlg_argN: &[value_type]
                // Call site: , AsRef::<[value_type]>::as_ref(value_tokens...)
                self.add_func_slice_arg(
                    field.span,
                    &arg_ident,
                    field.option,
                    field.value_tokens.as_ref().unwrap(),
                );

                // EventDataDescriptor::from_cstr(_tlg_argN),
                self.add_data_desc_for_arg_n(&arg_ident, DATADESC_FROM_CSTR_PATH());

                // value_type is u8 or u16
                let field_value_type = field.option.value_type();

                self.data_desc_init_tree.extend(quote_spanned! {
                    field.span =>
                    // EventDataDescriptor::from_value<value_type>(&0),
                    ::tracelogging::_internal::EventDataDescriptor::from_value::<#field_value_type>(&0),
                });
            }

            FieldStrategy::Counted => {
                if field.option.value_array_count == 0 {
                    // Prototype: , _tlg_argN: &[value_type]
                    // Call site: , AsRef::<[value_type]>::as_ref(value_tokens...)
                    self.add_func_slice_arg(
                        field.span,
                        &arg_ident,
                        field.option,
                        field.value_tokens.as_ref().unwrap(),
                    );
                } else {
                    // e.g. ipv6 takes a fixed-length array, not a variable-length slice
                    // , identity::<&value_type>(value_tokens...)

                    // Prototype: , _tlg_argN: &[value_type; value_array_count]
                    // Call site: , identity::<&[value_type; value_array_count]>(value_tokens...)
                    self.add_func_scalar_arg(
                        field.span,
                        &arg_ident,
                        field.option,
                        // Use identity(...) as a target for error messages.
                        identity_call(
                            field.option.value_type(),
                            field.option.value_array_count,
                            field.value_tokens.clone().unwrap(),
                        ),
                    );
                }

                // EventDataDescriptor::from_value(&_tlg_lengths[N]),
                // EventDataDescriptor::from_counted(_tlg_argN),
                self.add_data_desc_with_length(
                    &arg_ident,
                    COUNTED_SIZE_PATH(),
                    DATADESC_FROM_COUNTED_PATH(),
                );
            }

            FieldStrategy::Slice => {
                self.add_func_slice_arg(
                    field.span,
                    &arg_ident,
                    field.option,
                    field.value_tokens.as_ref().unwrap(),
                );

                // EventDataDescriptor::from_value(&_tlg_lengths[N]),
                // EventDataDescriptor::from_slice(_tlg_argN),
                self.add_data_desc_with_length(
                    &arg_ident,
                    SLICE_COUNT_PATH(),
                    DATADESC_FROM_SLICE_PATH(),
                );
            }

            FieldStrategy::Struct
            | FieldStrategy::RawStruct
            | FieldStrategy::RawStructSlice
            | FieldStrategy::RawMeta
            | FieldStrategy::RawMetaSlice => {}
        }

        // Common

        self.field_count += 1;
    }

    fn add_data_desc_for_arg_n(&mut self, arg_ident: &Ident, new_desc_path: TokenStream) {
        self.data_desc_init_tree.extend(quote! {
            // EventDataDescriptor::new_desc_path(_tlg_argN),
            #new_desc_path(#arg_ident),
        });
    }

    fn add_data_desc_with_length(
        &mut self,
        arg_ident: &Ident,
        get_length_path: TokenStream,
        new_desc_path: TokenStream,
    ) {
        // get_length_path(_tlg_argN),

        self.lengths_init_tree.extend(quote! {
            #get_length_path(#arg_ident),
        });

        // EventDataDescriptor::from_value(&_tlg_lengths[N]),
        // EventDataDescriptor::new_desc_path(_tlg_argN),
        let lengths_count = self.lengths_count;
        self.data_desc_init_tree.extend(quote! {
            ::tracelogging::_internal::EventDataDescriptor::from_value(
                &_tlg_lengths[#lengths_count],
            ),
        });
        self.add_data_desc_for_arg_n(arg_ident, new_desc_path);

        self.lengths_count += 1;
    }

    // We wrap all input expressions in adapter<T>(expression) because it allows
    // us to get MUCH better error messages. We attribute the adapter<T>() tokens
    // to the type_name_span so that if the expression is the wrong type, the
    // error message says "your expression didn't match the type expected by -->"
    // and the arrow points at the type_name, which is great. In cases where
    // as_ref() can be used, we use as_ref() as the adapter. Otherwise, we use
    // identity().

    /// Prototype: , _tlg_argN: &VALUE_TYPE
    /// Call site: , value...
    fn add_func_scalar_arg(
        &mut self,
        field_span: Span,
        arg_ident: &Ident,
        field_option: &FieldOption,
        value: Expr,
    ) {
        // _tlg_argN: &VALUE_TYPE,
        let ty = crate::tree::scalar_type_path(
            field_option.value_type(),
            field_option.value_array_count,
        );
        self.func_args_tree.extend(quote_spanned! {
            field_span =>
            #arg_ident: &#ty,
        });

        // We do not apply AsRef for non-slice types. AsRef provides a no-op mapping
        // for slices (i.e. AsRef<[u8]>::as_ref(&u8_slice) returns &u8_slice), but
        // there is not a no-op mapping for non-slice types (i.e.
        // AsRef<u8>::as_ref(&u8_val) will be a compile error). While this is a bit
        // inconsistent, I don't think it's a problem in practice. The non-slice
        // types don't get much value from as_ref. Most of their needs are handled
        // by the Deref trait, which the compiler applies automatically.

        // value_tokens... ,
        self.func_call_tree
            .extend(quote_spanned!(field_span => #value,));
    }

    /// Prototype: _tlg_argN: &[VALUE_TYPE],
    /// Call site: AsRef::<[VALUE_TYPE]>::as_ref(value_tokens...),
    fn add_func_slice_arg(
        &mut self,
        field_span: Span,
        arg_ident: &Ident,
        field_option: &FieldOption,
        field_value: &Expr,
    ) {
        // , _tlg_argN: &[VALUE_TYPE]
        let field_value_ty =
            scalar_type_path(field_option.value_type(), field_option.value_array_count);
        self.func_args_tree.extend(quote_spanned! {
            field_span =>
            #arg_ident: &[ #field_value_ty ],
        });

        // For cases where the expected input is a slice &[T], we apply the
        // core::convert::AsRef<[T]> trait to unwrap the provided value. This is
        // most important for strings because otherwise the str functions would only
        // accept &[u8] (they wouldn't be able to accept &str or &String). This also
        // applies to 3rd-party types, e.g. widestring's U16String implements
        // AsRef<[u16]> so it just works as a value for the str16 field types.

        // , AsRef::<[VALUE_TYPE]>::as_ref(value_tokens...)
        let ty = scalar_type_path(field_option.value_type(), field_option.value_array_count);
        self.func_call_tree.extend(quote_spanned! {
            field_span =>
            // Use as_ref(...) as a target for error messages.
            ::core::convert::AsRef::<[#ty]>::as_ref(#field_value),
        });
    }

    fn add_typecode_meta(
        &mut self,
        enum_type_path: Type,
        expr: Option<&Expr>,
        type_token: EnumToken,
        flags: u8,
    ) {
        let mut init_value = if let Some(expr) = expr {
            self.flush_meta_buffer();
            // Use identity(...) as a target for error messages.
            quote!(::core::convert::identity::<#enum_type_path>(#expr))
        } else {
            match type_token {
                EnumToken::U8(enum_int) => {
                    self.meta_buffer.push(enum_int | flags);
                    return;
                }

                EnumToken::Str(enum_name) => {
                    self.flush_meta_buffer();
                    let enum_ident = Ident::new(&enum_name, Span::call_site());
                    quote!(#enum_type_path :: #enum_ident)
                }
            }
        };

        self.meta_type_tree.extend(quote!(u8,));

        init_value.extend(quote!(.as_int()));

        if flags != 0 {
            init_value.extend(quote!(| #flags));
        }

        let meta_type_tree = &self.meta_type_tree;
        check_parse::<Expr>(quote! {
            Foo(#meta_type_tree)
        });

        let _ = check_parse::<Expr>(init_value.clone());
        self.meta_init_tree.extend(quote! {
            #init_value,
        });
    }

    fn add_tag(&mut self, tag_ident: &Ident, expression: &Expr) {
        // Implicitly uses self.tag_const as the name for the tag's constant.

        self.flush_meta_buffer();

        // const _TLG_TAGn: u32 = TAG;
        self.tags_tree.extend(quote! {
            const #tag_ident: u32 = #expression;

            // _TLG_TAGn <= 0x0FFFFFFF, "...");
            #[allow(clippy::assertions_on_constants)]
            const _: () = assert!(#tag_ident <= 0x0FFFFFFF, "tag must not be greater than 0x0FFFFFFF");
        });

        // , [u8; tag_size(_TLG_TAGn)]
        self.meta_type_tree.extend(quote! {
            [u8; ::tracelogging::_internal::tag_size(#tag_ident)],
        });

        // , tag_encode(_TLG_TAGn)
        self.meta_init_tree.extend(quote! {
            ::tracelogging::_internal::tag_encode(#tag_ident),
        });
    }

    /// If `meta_buffer` is empty, does nothing, otherwise, if there are `N` bytes of
    /// metadata in meta_buffer, adds a `[u8;N]` field to `meta_type_tree`, adds a binary
    /// literal containing the data to `meta_init_tree`, then clears `meta_buffer`.
    fn flush_meta_buffer(&mut self) {
        if !self.meta_buffer.is_empty() {
            // [u8; LEN] = , b"VAL"
            let meta_buffer_len = self.meta_buffer.len();
            self.meta_type_tree.extend(quote! {
                [u8; #meta_buffer_len],
            });

            let meta_buffer: &[u8] = &self.meta_buffer;
            self.meta_init_tree.extend(quote! {
                [#(#meta_buffer),*],
            });
            self.meta_buffer.clear();
        }
    }
}
