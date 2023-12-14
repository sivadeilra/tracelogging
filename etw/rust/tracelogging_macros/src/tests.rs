use core::mem::take;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::io::{Read, Write};
use syn::parse::{Parse, ParseStream};
use syn::Expr;

use crate::event_generator::EventGenerator;
use crate::event_info::EventInfo;
use crate::provider_generator::ProviderGenerator;
use crate::provider_info::ProviderInfo;

fn provider_case(input: TokenStream) {
    match syn::parse2::<ProviderInfo>(input) {
        Err(error_tokens) => panic!("parsing failed: {error_tokens:?}"),
        Ok(info) => {
            let output = ProviderGenerator::generate(info);

            let wrapped_output = quote! {
                pub fn foo() {
                    #output
                }
            };

            let s = format(&wrapped_output);
            print!("{s}");
        }
    }
}

#[test]
fn provider_simple() {
    provider_case(quote! {
        PROV1, "TestProvider"
    });
}

#[test]
fn provider_with_id() {
    provider_case(quote! {
        PROV2,
        "TestProvider2",
        id("97c801ee-c28b-5bb6-2ae4-11e18fe6137a"),
    });
}

#[test]
fn provider_with_group_id() {
    provider_case(quote! {
        PROV3,
        "TestProvider3",
        group_id("12345678-9abc-def0-1234-56789abcdef0"),
    })
}

#[test]
fn provider_with_both_ids() {
    provider_case(quote! {
        PROV4,
        "TestProvider4",
        id("97c801ee-c28b-5bb6-2ae4-11e18fe6137a"),
        group_id("12345678-9abc-def0-1234-56789abcdef0"),
    })
}

struct EventInfoDbg(EventInfo);

impl Parse for EventInfoDbg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<EventInfo>() {
            Err(e) => {
                println!("failed to parse EventInfo: {e:?}");
                println!("input: {input}");
                return Err(e);
            }
            Ok(event) => Ok(Self(event)),
        }
    }
}

fn event_case(input: TokenStream) {
    match syn::parse2::<EventInfoDbg>(input) {
        Err(error_tokens) => panic!("parsing failed: {error_tokens:?}"),
        Ok(EventInfoDbg(prov)) => {
            let output = EventGenerator::new(Span::call_site()).generate(prov);

            match syn::parse2::<Expr>(output.clone()) {
                Ok(_) => {
                    println!("successfully parsed output as Expr");
                }
                Err(e) => {
                    println!("failed to parse output as Expr: {e:?}");
                }
            }

            let wrapped_output = quote! {
                pub fn foo() {
                    #output
                }
            };

            let s = format(&wrapped_output);
            print!("{s}");
        }
    }
}

#[test]
fn event_no_args() {
    event_case(quote!(PROV1, "MyEventName"));
}

#[test]
fn event_no_args_with_comma() {
    event_case(quote!(PROV1, "MyEventName"));
}

#[test]
fn event_no_args_with_activity_id() {
    event_case(quote! {
        PROV1, "MyEventName",
        activity_id(&guid1)
    });
}

#[test]
fn event_with_level() {
    event_case(quote! {
        PROVIDER,
        "Foo",
        level(Informational),
    });
}

#[test]
fn event_with_opcode() {
    event_case(quote! {
        PROVIDER,
        "Foo",
        opcode(0),
    });
}

#[test]
fn event_with_ipv4() {
    event_case(quote! {
        PROVIDER,
        "Foo",
        // ipv4(0x12345678),
        ipv4("ipv4", &[127, 0, 0, 1]),
    });
}

#[test]
fn event_with_pointer() {
    event_case(quote! {
        PROVIDER,
        "Foo",
        pointer("pointer", &1234),
    });
}

#[test]
fn event_with_time32() {
    event_case(quote! {
        PROVIDER,
        "Foo",
        time32("time32", &100),
    });
}

fn format(t: &TokenStream) -> String {
    let tokens_string = t.to_string();

    use std::process::{Command, Stdio};

    let mut cmd = Command::new("rustfmt");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());

    let mut child = match cmd.spawn() {
        Err(_) => return tokens_string,
        Ok(child) => child,
    };

    let mut stdin = take(&mut child.stdin).unwrap();
    let mut stdout = take(&mut child.stdout).unwrap();

    let stdout_result = std::thread::scope(|scope| {
        scope.spawn(|| {
            let _ = stdin.write_all(tokens_string.as_bytes());
            drop(stdin);
        });

        let mut formatted_output = String::new();
        match stdout.read_to_string(&mut formatted_output) {
            Ok(_) => Ok(formatted_output),
            Err(e) => Err(e),
        }
    });

    let _ = child.wait();

    match stdout_result {
        Ok(formatted_output) => formatted_output,
        Err(_) => tokens_string,
    }
}
