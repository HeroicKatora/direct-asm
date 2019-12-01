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

pub struct Line {
    pub label: Option<String>,
    pub kind: LineKind,
    pub comment: Option<String>,
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
