// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![allow(non_snake_case)]

use proc_macro2::TokenStream;
use quote::quote;

use crate::enums::{InType as I, OutType as O};
use crate::field_option::{FieldOption as Opt, FieldStrategy::*};

#[allow(dead_code)]
pub fn check_field_options() {
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
}

fn U8_PATH() -> TokenStream {
    quote!(u8)
}

fn I32_PATH() -> TokenStream {
    quote!(i32)
}

fn BOOL_PATH() -> TokenStream {
    quote!(bool)
}

fn F32_PATH() -> TokenStream {
    quote!(f32)
}

fn GUID_PATH() -> TokenStream {
    quote!(::tracelogging::Guid)
}

fn I16_PATH() -> TokenStream {
    quote!(i16)
}

fn I64_PATH() -> TokenStream {
    quote!(i64)
}

fn I8_PATH() -> TokenStream {
    quote!(i8)
}

fn ISIZE_PATH() -> TokenStream {
    quote!(isize)
}

fn U16_PATH() -> TokenStream {
    quote!(u16)
}

fn U32_PATH() -> TokenStream {
    quote!(u32)
}

fn U64_PATH() -> TokenStream {
    quote!(u64)
}

fn USIZE_PATH() -> TokenStream {
    quote!(usize)
}

fn F64_PATH() -> TokenStream {
    quote!(f64)
}

fn EMPTY_SLICE() -> TokenStream {
    quote!(&[])
}

/// List must be strcmp-sorted by option_name (for binary search).
/// (Verified by debug_assert in EventInfo::try_from_tokens.)
#[rustfmt::skip]
pub static FIELD_OPTIONS: &[Opt] = &[
    Opt::new("binary",                  U8_PATH,    I::Binary,     O::Default,       Counted,    0),
    Opt::new("binaryc",                 U8_PATH,    I::BinaryC,    O::Default,       Counted,    0),
    Opt::new("bool32",                  I32_PATH,   I::Bool32,     O::Default,       Scalar,     0),
    Opt::new("bool32_slice",            I32_PATH,   I::Bool32,     O::Default,       Slice,      0),
    Opt::new("bool8",                   BOOL_PATH,  I::U8,         O::Boolean,       Scalar,     0),
    Opt::new("bool8_slice",             BOOL_PATH,  I::U8,         O::Boolean,       Slice,      0),
    Opt::new("char16",                  U16_PATH,   I::U16,        O::String,        Scalar,     0),
    Opt::new("char16_slice",            U16_PATH,   I::U16,        O::String,        Slice,      0),
    Opt::new("char8_cp1252",            U8_PATH,    I::U8,         O::String,        Scalar,     0),
    Opt::new("char8_cp1252_slice",      U8_PATH,    I::U8,         O::String,        Slice,      0),
    Opt::new("codepointer",             USIZE_PATH, I::HexSize,    O::CodePointer,   Scalar,     0),
    Opt::new("codepointer_slice",       USIZE_PATH, I::HexSize,    O::CodePointer,   Slice,      0),
    Opt::new("cstr16",                  U16_PATH,   I::CStr16,     O::Default,       CStr,       0),
    Opt::new("cstr16_json",             U16_PATH,   I::CStr16,     O::Json,          CStr,       0),
    Opt::new("cstr16_xml",              U16_PATH,   I::CStr16,     O::Xml,           CStr,       0),
    Opt::new("cstr8",                   U8_PATH,    I::CStr8,      O::Utf8,          CStr,       0),
    Opt::new("cstr8_cp1252",            U8_PATH,    I::CStr8,      O::Default,       CStr,       0),
    Opt::new("cstr8_json",              U8_PATH,    I::CStr8,      O::Json,          CStr,       0),
    Opt::new("cstr8_xml",               U8_PATH,    I::CStr8,      O::Xml,           CStr,       0),
    Opt::new("errno",                   I32_PATH,   I::I32,        O::Default,       Scalar,     0),
    Opt::new("errno_slice",             I32_PATH,   I::I32,        O::Default,       Slice,      0),
    Opt::new("f32",                     F32_PATH,   I::F32,        O::Default,       Scalar,     0),
    Opt::new("f32_slice",               F32_PATH,   I::F32,        O::Default,       Slice,      0),
    Opt::new("f64",                     F64_PATH,   I::F64,        O::Default,       Scalar,     0),
    Opt::new("f64_slice",               F64_PATH,   I::F64,        O::Default,       Slice,      0),
    Opt::new("guid",                    GUID_PATH,  I::Guid,       O::Default,       Scalar,     0),
    Opt::new("guid_slice",              GUID_PATH,  I::Guid,       O::Default,       Slice,      0),
    Opt::new("hresult",                 I32_PATH,   I::I32,        O::HResult,       Scalar,     0),
    Opt::new("hresult_slice",           I32_PATH,   I::I32,        O::HResult,       Slice,      0),
    Opt::new("i16",                     I16_PATH,   I::I16,        O::Default,       Scalar,     0),
    Opt::new("i16_hex",                 I16_PATH,   I::U16,        O::Hex,           Scalar,     0),
    Opt::new("i16_hex_slice",           I16_PATH,   I::U16,        O::Hex,           Slice,      0),
    Opt::new("i16_slice",               I16_PATH,   I::I16,        O::Default,       Slice,      0),
    Opt::new("i32",                     I32_PATH,   I::I32,        O::Default,       Scalar,     0),
    Opt::new("i32_hex",                 I32_PATH,   I::Hex32,      O::Default,       Scalar,     0),
    Opt::new("i32_hex_slice",           I32_PATH,   I::Hex32,      O::Default,       Slice,      0),
    Opt::new("i32_slice",               I32_PATH,   I::I32,        O::Default,       Slice,      0),
    Opt::new("i64",                     I64_PATH,   I::I64,        O::Default,       Scalar,     0),
    Opt::new("i64_hex",                 I64_PATH,   I::Hex64,      O::Default,       Scalar,     0),
    Opt::new("i64_hex_slice",           I64_PATH,   I::Hex64,      O::Default,       Slice,      0),
    Opt::new("i64_slice",               I64_PATH,   I::I64,        O::Default,       Slice,      0),
    Opt::new("i8",                      I8_PATH,    I::I8,         O::Default,       Scalar,     0),
    Opt::new("i8_hex",                  I8_PATH,    I::U8,         O::Hex,           Scalar,     0),
    Opt::new("i8_hex_slice",            I8_PATH,    I::U8,         O::Hex,           Slice,      0),
    Opt::new("i8_slice",                I8_PATH,    I::I8,         O::Default,       Slice,      0),
    Opt::new("ipv4",                    U8_PATH,    I::U32,        O::IPv4,          Scalar,     4),
    Opt::new("ipv4_slice",              U8_PATH,    I::U32,        O::IPv4,          Slice,      4),
    Opt::new("ipv6",                    U8_PATH,    I::Binary,     O::IPv6,          Counted,    16),
    Opt::new("ipv6c",                   U8_PATH,    I::BinaryC,    O::IPv6,          Counted,    16),
    Opt::new("isize",                   ISIZE_PATH, I::ISize,      O::Default,       Scalar,     0),
    Opt::new("isize_hex",               ISIZE_PATH, I::HexSize,    O::Default,       Scalar,     0),
    Opt::new("isize_hex_slice",         ISIZE_PATH, I::HexSize,    O::Default,       Slice,      0),
    Opt::new("isize_slice",             ISIZE_PATH, I::ISize,      O::Default,       Slice,      0),
    Opt::new("pid",                     U32_PATH,   I::U32,        O::Pid,           Scalar,     0),
    Opt::new("pid_slice",               U32_PATH,   I::U32,        O::Pid,           Slice,      0),
    Opt::new("pointer",                 USIZE_PATH, I::HexSize,    O::Default,       Scalar,     0),
    Opt::new("pointer_slice",           USIZE_PATH, I::HexSize,    O::Default,       Slice,      0),
    Opt::new("port",                    U16_PATH,   I::U16,        O::Port,          Scalar,     0),
    Opt::new("port_slice",              U16_PATH,   I::U16,        O::Port,          Slice,      0),
    Opt::new("raw_data",                U8_PATH,    I::Invalid,    O::Default,       RawData,        0),
    Opt::new("raw_field",               U8_PATH,    I::Invalid,    O::Default,       RawField,       0),
    Opt::new("raw_field_slice",         U8_PATH,    I::Invalid,    O::Default,       RawFieldSlice,  0),
    Opt::new("raw_meta",                EMPTY_SLICE,I::Invalid,    O::Default,       RawMeta,        0),
    Opt::new("raw_meta_slice",          EMPTY_SLICE,I::Invalid,    O::Default,       RawMetaSlice,   0),
    Opt::new("raw_struct",              EMPTY_SLICE,I::Struct,     O::Default,       RawStruct,      0),
    Opt::new("raw_struct_slice",        EMPTY_SLICE,I::Struct,     O::Default,       RawStructSlice, 0),
    Opt::new("socketaddress",           U8_PATH,    I::Binary,     O::SocketAddress, Counted,        0),
    Opt::new("socketaddressc",          U8_PATH,    I::BinaryC,    O::SocketAddress, Counted,    0),
    Opt::new("str16",                   U16_PATH,   I::Str16,      O::Default,       Counted,    0),
    Opt::new("str16_json",              U16_PATH,   I::Str16,      O::Json,          Counted,    0),
    Opt::new("str16_xml",               U16_PATH,   I::Str16,      O::Xml,           Counted,    0),
    Opt::new("str8",                    U8_PATH,    I::Str8,       O::Utf8,          Counted,    0),
    Opt::new("str8_cp1252",             U8_PATH,    I::Str8,       O::Default,       Counted,    0),
    Opt::new("str8_json",               U8_PATH,    I::Str8,       O::Json,          Counted,    0),
    Opt::new("str8_xml",                U8_PATH,    I::Str8,       O::Xml,           Counted,    0),
    Opt::new("struct",                  EMPTY_SLICE,I::Struct,     O::Default,       Struct,     0),
    Opt::new("systemtime",              I64_PATH,   I::FileTime,   O::Default,       SystemTime, 0),
    Opt::new("tid",                     U32_PATH,   I::U32,        O::Tid,           Scalar,     0),
    Opt::new("tid_slice",               U32_PATH,   I::U32,        O::Tid,           Slice,      0),
    Opt::new("time32",                  I64_PATH,   I::FileTime,   O::Default,       Time32,     0),
    Opt::new("time64",                  I64_PATH,   I::FileTime,   O::Default,       Time64,     0),
    Opt::new("u16",                     U16_PATH,   I::U16,        O::Default,       Scalar,     0),
    Opt::new("u16_hex",                 U16_PATH,   I::U16,        O::Hex,           Scalar,     0),
    Opt::new("u16_hex_slice",           U16_PATH,   I::U16,        O::Hex,           Slice,      0),
    Opt::new("u16_slice",               U16_PATH,   I::U16,        O::Default,       Slice,      0),
    Opt::new("u32",                     U32_PATH,   I::U32,        O::Default,       Scalar,     0),
    Opt::new("u32_hex",                 U32_PATH,   I::Hex32,      O::Default,       Scalar,     0),
    Opt::new("u32_hex_slice",           U32_PATH,   I::Hex32,      O::Default,       Slice,      0),
    Opt::new("u32_slice",               U32_PATH,   I::U32,        O::Default,       Slice,      0),
    Opt::new("u64",                     U64_PATH,   I::U64,        O::Default,       Scalar,     0),
    Opt::new("u64_hex",                 U64_PATH,   I::Hex64,      O::Default,       Scalar,     0),
    Opt::new("u64_hex_slice",           U64_PATH,   I::Hex64,      O::Default,       Slice,      0),
    Opt::new("u64_slice",               U64_PATH,   I::U64,        O::Default,       Slice,      0),
    Opt::new("u8",                      U8_PATH,    I::U8,         O::Default,       Scalar,     0),
    Opt::new("u8_hex",                  U8_PATH,    I::U8,         O::Hex,           Scalar,     0),
    Opt::new("u8_hex_slice",            U8_PATH,    I::U8,         O::Hex,           Slice,      0),
    Opt::new("u8_slice",                U8_PATH,    I::U8,         O::Default,       Slice,      0),
    Opt::new("usize",                   USIZE_PATH, I::USize,      O::Default,       Scalar,     0),
    Opt::new("usize_hex",               USIZE_PATH, I::HexSize,    O::Default,       Scalar,     0),
    Opt::new("usize_hex_slice",         USIZE_PATH, I::HexSize,    O::Default,       Slice,      0),
    Opt::new("usize_slice",             USIZE_PATH, I::USize,      O::Default,       Slice,      0),
    Opt::new("win_error",               U32_PATH,   I::U32,        O::Win32Error,    Scalar,     0),
    Opt::new("win_error_slice",         U32_PATH,   I::U32,        O::Win32Error,    Slice,      0),
    Opt::new("win_filetime",            I64_PATH,   I::FileTime,   O::Default,       Scalar,     0),
    Opt::new("win_filetime_slice",      I64_PATH,   I::FileTime,   O::Default,       Slice,      0),
    Opt::new("win_ntstatus",            I32_PATH,   I::Hex32,      O::NtStatus,      Scalar,     0),
    Opt::new("win_ntstatus_slice",      I32_PATH,   I::Hex32,      O::NtStatus,      Slice,      0),
    Opt::new("win_sid",                 U8_PATH,    I::Sid,        O::Default,       Sid,        0),
    Opt::new("win_systemtime",          U16_PATH,   I::SystemTime, O::Default,       Scalar,     8),
    Opt::new("win_systemtime_slice",    U16_PATH,   I::SystemTime, O::Default,       Slice,      8),
    Opt::new("win_systemtime_utc",      U16_PATH,   I::SystemTime, O::DateTimeUtc,   Scalar,     8),
    Opt::new("win_systemtime_utc_slice",U16_PATH,   I::SystemTime, O::DateTimeUtc,   Slice,      8),
];
