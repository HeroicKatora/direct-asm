use std::process;
use std::io::Write;

extern crate proc_macro;

use proc_macro::{Delimiter, Literal, Group, Punct, Spacing, TokenStream, TokenTree};
use quote::{quote, ToTokens};

#[proc_macro_attribute]
pub fn assemble(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let (head, body) = split_function(input);
    let asm_input = get_body(body);

    let function_def = proc_macro2::TokenStream::from(head.function_def);
    let symbol_name = head.name;
    let raw = nasmify(&asm_input);
    let len = raw.len();
    let definition = {
        let mut items = TokenStream::new();
        for byte in &raw {
            if !items.is_empty() {
                items.extend(Some(TokenTree::Punct(Punct::new(',', Spacing::Alone))));
            }
            items.extend(Some(TokenTree::Literal(Literal::u8_unsuffixed(*byte))));
        }
        let tree = TokenTree::Group(Group::new(Delimiter::Bracket, items));
        let stream = TokenStream::from(tree);
        proc_macro2::TokenStream::from(stream)
    };

    let mut binary_symbol = quote! {
        mod _no_matter {
            #[link_section=".text"]
            #[no_mangle]
            static #symbol_name: [u8; #len] = #definition;
        }
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
fn split_function(input: TokenStream) -> (Head, TokenStream) {
    let mut fn_item = syn::parse::<syn::ItemFn>(input)
        .expect("Must annotate a method definition");
    // It should be declared as such because we put it into an `extern "C"` block ..
    assert!(fn_item.sig.abi.is_some(), "Must specify function as having C abi");
    // .. but remove it since the actual definition we output can not have it.
    fn_item.sig.abi = None;
    fn_item.sig.unsafety = None;
    let head = Head {
        function_def: fn_item.sig.to_token_stream().into(),
        name: fn_item.sig.ident,
    };
    (head, fn_item.block.to_token_stream().into())
}

fn get_body(block: TokenStream) -> String {
    let body;
    match &block.into_iter().next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            body = group.stream();
        },
        _ => panic!("Expected function body"),
    };

    let parts = body.into_iter().map(|item| match &item {
        TokenTree::Literal(literal) => {
            let stream = TokenTree::Literal(literal.clone()).into();
            let litstr = syn::parse::<syn::LitStr>(stream)
                .expect("Body only contain string literals");
            litstr.value()
        },
        TokenTree::Punct(punc) if punc.as_char() == ';' => "\n".to_string(),
        other => panic!("Unexpected body content: {:?}", other),
    });

    let specified: String = parts.collect();
    format!("[BITS 64]\n{}", specified)
}

struct Head {
    function_def: TokenStream,
    name: syn::Ident,
}

fn nasmify(input: &str) -> Vec<u8> {
    use std::fs;
    fs::write("target/indirection.in", input).unwrap();

    let mut nasm = process::Command::new("nasm")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .args(&["-f", "bin", "-o", "/proc/self/fd/1", "target/indirection.in"])
        .spawn()
        .expect("Failed to spawn assembler");

    let stdin = nasm.stdin.as_mut().expect("Nasm must accept piped input");
    stdin.write_all(input.as_bytes()).expect("Failed to supply nasm with input");
    stdin.flush().expect("Failed to flush");

    let output = nasm.wait_with_output().expect("Failed to wait for nasm");
    if !output.status.success() || !output.stderr.is_empty() {
        panic!("Nasm failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    output.stdout
}
