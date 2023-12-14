// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![allow(non_snake_case)]

use proc_macro2::TokenStream;
use quote::quote;

/// Channel names special-cased by channel(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const CHANNEL_ENUMS: &[&str] = &["ProviderMetadata", "TraceClassic", "TraceLogging"];

/// Level names special-cased by level(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const LEVEL_ENUMS: &[&str] = &[
    "Critical",
    "Error",
    "Informational",
    "LogAlways",
    "Verbose",
    "Warning",
];

/// Opcode names special-cased by opcode(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const OPCODE_ENUMS: &[&str] = &[
    "DC_Start",
    "DC_Stop",
    "Extension",
    "Info",
    "Receive",
    "Reply",
    "Resume",
    "Send",
    "Start",
    "Stop",
    "Suspend",
];

/// InType names special-cased by type(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const INTYPE_ENUMS: &[&str] = &[
    "Binary",
    "Bool32",
    "CStr16",
    "CStr8",
    "CountedBinary",
    "F32",
    "F64",
    "FileTime",
    "Guid",
    "Hex32",
    "Hex64",
    "HexSize",
    "I16",
    "I32",
    "I64",
    "I8",
    "ISize",
    "Invalid",
    "Sid",
    "Str16",
    "Str8",
    "Struct",
    "SystemTime",
    "U16",
    "U32",
    "U64",
    "U8",
    "USize",
];

/// OutType names special-cased by format(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const OUTTYPE_ENUMS: &[&str] = &[
    "Boolean",
    "CodePointer",
    "DateTime",
    "DateTimeCultureInsensitive",
    "DateTimeUtc",
    "Default",
    "HResult",
    "Hex",
    "IPv4",
    "IPv6",
    "Json",
    "NoPrint",
    "NtStatus",
    "Pid",
    "Pkcs7WithTypeInfo",
    "Port",
    "Signed",
    "SocketAddress",
    "String",
    "Tid",
    "Unsigned",
    "Utf8",
    "Win32Error",
    "Xml",
];

pub const TLG_TAG_CONST: &str = "_TLG_TAG";
pub const TLG_ARG_VAR: &str = "_tlg_arg";

pub fn COUNTED_SIZE_PATH() -> TokenStream {
    quote! {
        ::tracelogging::_internal::counted_size
    }
}

pub fn SLICE_COUNT_PATH() -> TokenStream {
    quote! {
        ::tracelogging::_internal::slice_count
    }
}

pub fn DATADESC_FROM_VALUE_PATH() -> TokenStream {
    quote! {
        ::tracelogging::_internal::EventDataDescriptor::from_value
    }
}

pub fn DATADESC_FROM_SID_PATH() -> TokenStream {
    quote! {
        ::tracelogging::_internal::EventDataDescriptor::from_sid
    }
}

pub fn DATADESC_FROM_CSTR_PATH() -> TokenStream {
    quote! {
        ::tracelogging::_internal::EventDataDescriptor::from_cstr
    }
}

pub fn DATADESC_FROM_SLICE_PATH() -> TokenStream {
    quote! {
        ::tracelogging::_internal::EventDataDescriptor::from_slice
    }
}

pub fn DATADESC_FROM_COUNTED_PATH() -> TokenStream {
    quote! {
        ::tracelogging::_internal::EventDataDescriptor::from_counted
    }
}
