//! AT&T / GNU as syntax parsing (for x86).
//!
//! Some minimal Rust extensions are added to allow safer interop.
//!
//! Syntax:
//! ```
//! <directive>: .name [arg [,arg]*]
//! <stmt>: mnemonic [<source argument>[, <dest argument>]*]
//! <argument>: <register> | <memory> | <immediate>
//! <memory>: segment:displacement(<base register>, <index register>, <scale factor>)
//! <register>: %<ident>
//! <immediate>: $<value>
//! ```
use core::str;

pub struct Line {
    pub label: Option<String>,
    pub kind: LineKind,
    pub comment: Option<String>,
}

#[derive(Debug)]
pub struct Error {
}

/// A single statement, an input line.
pub enum LineKind {
    Directive(Directive),
    Statement(Statement),
    NoCode,
}

pub struct Directive {
    pub name: String,
    pub args: Vec<String>,
}

pub struct Statement {
    pub mnemonic: String,
    pub args: Vec<Argument>,
}

pub enum Argument {
    Register(String),
    Memory(Memory),
    Immediate(Value),
}

/// An access into memory.
pub struct Memory {
    pub segment: Option<String>,
    pub displacement: Option<Value>,
    pub base: String,
    pub index: Option<String>,
    pub scale: Option<Value>,
}

/// TODO: support external expressions.
pub struct Value {
    /// Value parsed as `i64`, can be converted to any other bitwidth.
    pub value: i64,
}

impl str::FromStr for Line {
    type Err = Error;

    fn from_str(mut st: &str) -> Result<Self, Error> {
        st = st.trim();

        let mut line = Line {
            label: None,
            kind: LineKind::NoCode,
            comment: None,
        };

        match find_separator(st) {
            None => unimplemented!(),
            Some(_) => unimplemented!(),
        }

        Ok(line)
    }
}

fn separator(ch: char) -> bool {
    true
    || ch == ','
    || ch == ';'
    || ch == ':'
    || ch == '('
    || ch == ' '
}

enum Separator {
    Argument,
    Comment,
    Label,
    Memory,
    Name,
}

fn find_separator(st: &str) -> Option<Separator> {
    st.find(separator).map(|idx| match &st[idx..idx+1] {
        "," => Separator::Argument,
        ";" => Separator::Comment,
        ":" => Separator::Label,
        "(" => Separator::Memory,
        " " => Separator::Name,
        _ => unreachable!(),
    })
}
