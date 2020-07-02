use std::process;
use std::io::Write;

extern crate proc_macro;

mod att;
mod x86;

use proc_macro::{Delimiter, Literal, Group, Punct, Spacing, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use rand::{thread_rng, Rng};

#[proc_macro_attribute]
pub fn assemble(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mut assembler: Box<dyn Assembler> = choose_backed(&attr);

    let (head, body) = split_function(input);
    let asm_input = get_body(body);


    let raw = assembler.assemble(&asm_input);
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

    let unique_name = choose_link_name();
    let unique_ident = syn::Ident::new(&unique_name, proc_macro2::Span::call_site());
    let mut binary_symbol = quote! {
        mod #unique_ident {
            #[link_section=".text"]
            #[no_mangle]
            static #unique_ident: [u8; #len] = #definition;
        }
    };

    let function_def = syn::ForeignItem::Fn(syn::ForeignItemFn {
        attrs: vec![syn::parse_quote!(#[link_name=#unique_name])],
        vis: head.visibility,
        sig: head.function_def,
        semi_token: syn::token::Semi::default(),
    });

    let function_symbol = quote! {
        extern "C" {
            #function_def
        }
    };

    binary_symbol.extend(function_symbol);
    binary_symbol.into()
}

fn choose_backed(attr: &[syn::NestedMeta]) -> Box<dyn Assembler> {
    enum Backend {
        GnuAs,
        Nasm,
        Dynasm,
    }

    let backend = match &attr {
        [syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { path , lit, .. }))] => {
            if path.is_ident("backend") {
                if let syn::Lit::Str(st) = lit {
                    match st.value().as_str() {
                        "nasm" => Backend::Nasm,
                        "dynasm" => Backend::Dynasm,
                        "gnu-as" | "gnuas" | "gas" | "as" => Backend::GnuAs,
                        _ => panic!("Unknown backend (nasm, dynasm, gnuas, gnu-as, gas, as)"),
                    }
                } else {
                    panic!("Expected string value identifying backend");
                }
            } else {
                panic!("Unexpected keyword")
            }
        },
        [] => Backend::Dynasm,
        _ => panic!("Backend is unknown"),
    };

    match backend {
        Backend::GnuAs => Box::new(GnuAs {}),
        Backend::Nasm => Box::new(Nasm),
        Backend::Dynasm => Box::new(x86::DynasmX86::new()),
    }
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
        function_def: fn_item.sig,
        visibility: fn_item.vis,
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

    parts.collect()
}

/// Generate a random (196-bit) unique identifier for the symbol link in the proc macro.
///
/// To execute the trick of re-interpreting a byte stream as a function we must choose a common
/// link name between the symbol and the later function definition that imports that symbol. This
/// should not collide with other defined symbols, as that might silently be unsafe.
fn choose_link_name() -> String {
    const CHOICES: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789ZZ";
    let mut randoms = [0u8; 32];
    thread_rng().fill(&mut randoms);

    let random = randoms
        .iter()
        .map(|idx| usize::from(idx & 63))
        .map(|idx| std::char::from_u32(CHOICES[idx].into()).unwrap())
        .collect::<String>();

    format!("_direct_asm_{}", random)
}

struct Head {
    function_def: syn::Signature,
    visibility: syn::Visibility,
}

trait Assembler {
    fn assemble(&mut self, input: &str) -> Vec<u8>;
}

struct Nasm;

struct GnuAs {
}

fn nasmify(input: &str) -> Vec<u8> {
    let input = format!("[BITS 64]\n{}", input);
    std::fs::write("target/indirection.in", &input).unwrap();

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

impl Assembler for Nasm {
    fn assemble(&mut self, input: &str) -> Vec<u8> {
        nasmify(input)
    }
}

impl Assembler for GnuAs {
    fn assemble(&mut self, original_input: &str) -> Vec<u8> {
        let newlined;
        let input: &str;

        if original_input.chars().rev().next() != Some('\n') {
            newlined = format!("{}\n", original_input);
            input = &newlined;
        } else {
            input = original_input;
        }

        const ASSEMBLED_FILE: &str = "target/gnu-as.out";
        // Some arguments for reference:
        // target selection: -march=<name>
        // --32, --64, --x32 for isa qualification
        // -n do not optimize alignment
        // -mmnemonic/-msyntax=[att|intel]
        let mut as_ = process::Command::new("as")
            // We act as if this was safe, the least we can do is check thoroughly.
            .arg("-msse-check=error")
            .arg("-moperand-check=error")
            .arg("-mmnemonic=intel")
            .arg("-msyntax=intel")
            .args(&["-o", ASSEMBLED_FILE])
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn assembler");

        let stdin = as_.stdin.as_mut().expect("As must accept piped input");
        stdin.write_all(input.as_bytes()).expect("Failed to supply as with input");
        stdin.flush().expect("Failed to flush");

        let output = as_.wait_with_output().expect("Failed to wait for as");
        if !output.status.success() || !output.stderr.is_empty() {
            panic!("Gnu As failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        // gnu as will always output ELF. We only need the binary from it. Better hope you didn't
        // use any tables or so, as those will be dropped in the process.
        // TODO: fail loudly.
        let status = process::Command::new("objcopy")
            .args(&["-O", "binary"])
            .arg(ASSEMBLED_FILE)
            .status()
            .expect("Failed to spawn `objcopy`");
        assert!(status.success(), "`objcopy` failed");

        std::fs::read(ASSEMBLED_FILE).expect("No output produced")
    }
}
