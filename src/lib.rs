use std::process;
use std::io::Write;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn assemble(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let (head, body) = split_function(input);
    let head = parse_head(head);
    let asm_input = get_body(body);

    let function_def = proc_macro2::TokenStream::from(head.function_def);
    let symbol_name = head.name.to_string();
    let raw = nasmify(&asm_input);
    let len = raw.len();
    let definition = format!("{:?}", raw.as_slice());

    let mut binary_symbol = quote! {
        #[link_section=".text"]
        #[no_mangle]
        static #symbol_name: [u8; #len] = #definition;
    };

    let function_symbol = quote! {
        extern "C" {
            #function_def;
        }
    };

    binary_symbol.extend(function_symbol);
    binary_symbol.into()
}

/// Split the function head and body.
fn split_function(input: TokenStream) -> (TokenStream, TokenStream) {
    unimplemented!()
}

fn get_body(body: TokenStream) -> String {
    unimplemented!()
}

fn parse_head(head: TokenStream) -> Head {
    unimplemented!()
}

struct Head {
    function_def: TokenStream,
    name: proc_macro::Ident,
}

fn nasmify(input: &str) -> Vec<u8> {
    let mut nasm = process::Command::new("nasm")
        .stdin(process::Stdio::piped())
        .args(&["-f", "bin", "-o", "/proc/self/fd/1", "/proc/self/fd/0"])
        .spawn()
        .expect("Failed to spawn assembler");
    let stdin = nasm.stdin.as_mut().expect("Nasm must accept piped input");
    stdin.write_all(input.as_bytes()).expect("Failed to supply nasm with input");

    let output = nasm.wait_with_output().expect("Failed to wait for nasm");
    if !output.status.success() {
        panic!("Nasm failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    output.stdout
}
