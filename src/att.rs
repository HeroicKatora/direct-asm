//! AT&T / GNU as syntax parsing (for x86).
//!
//! Some minimal Rust extensions are added to allow safer interop.
//!
//! Syntax:
//! ```text
//! <directive>: .name [arg [,arg]*]
//! <stmt>: mnemonic [<source argument>[, <dest argument>]*]
//! <argument>: <register> | <memory> | <immediate>
//! <memory>: segment:displacement(<base register>, <index register>, <scale factor>)
//! <register>: %<ident>
//! <immediate>: $<value>
//! ```
use core::str;

#[derive(Debug)]
pub struct Line {
    pub label: Option<String>,
    pub kind: LineKind,
    pub comment: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    ArgumentWithoutCode,
    DirectivesHaveSingleName,
    EmptyLabel,
    InvalidImmediateValue,
    NoClosingParen,
    NoOpcodeOnlyArguments,
    SecondLabel,
    OpcodeWithoutCode,
}

/// A single statement, an input line.
#[derive(Debug)]
pub enum LineKind {
    Directive(Directive),
    Statement(Statement),
    NoCode,
}

#[derive(Debug)]
pub struct Directive {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug)]
pub struct Statement {
    /// Mnemonic of the op code and prefixes.
    pub mnemonic: Vec<String>,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Argument {
    /// A register argument.
    Register(String),
    /// A memory reference (read or write or other is determined later).
    Memory(Memory),
    /// A value constant in code.
    Immediate(Value),
}

/// An access into memory.
#[derive(Debug, PartialEq, Eq)]
pub struct Memory {
    pub segment: Option<String>,
    pub displacement: Option<Value>,
    pub base: String,
    pub index: Option<String>,
    pub scale: Option<Value>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Const(Constant),
    Expr(Expression),
}

/// TODO: support external expressions.
#[derive(Debug, PartialEq, Eq)]
pub struct Constant {
    /// Value parsed as `i64`, can be converted to any other bitwidth.
    pub value: i64,
}

/// TODO: valid everywhere a value occurs?
#[derive(Debug, PartialEq, Eq)]
pub struct Expression {
    pub type_: Type,
    pub content: String,
}

/// Indicated integral bit type ascription of an expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    I128,
    U128,
    Isize,
    Usize,
    SShort,
    UShort,
    SInt,
    UInt,
    SLong,
    ULong,
}

impl str::FromStr for Line {
    type Err = Error;

    fn from_str(mut st: &str) -> Result<Self, Error> {
        #[derive(Default)]
        struct ParseContext {
            name_parts: Vec<String>,
            arguments: Vec<String>,
        }

        impl ParseContext {
            fn append_part(&mut self) -> &mut String {
                if self.arguments.is_empty() {
                    self.name_parts.push(String::new());
                    self.name_parts.last_mut().unwrap()
                } else {
                    let arg = self.arguments.last_mut().unwrap();
                    arg.push_str(" ");
                    arg
                }
            }

            fn add_argument(&mut self) -> Result<(), Error> {
                if self.arguments.is_empty() {
                    let last_name = self.name_parts.pop()
                        .ok_or_else(|| Error::NoOpcodeOnlyArguments)?;
                    self.arguments.push(last_name);
                }

                self.arguments.push(String::new());
                Ok(())
            }

            fn finalize(&mut self) {
                if self.arguments.is_empty() {
                    if self.name_parts.len() > 1 {
                        self.arguments.push(self.name_parts.pop().unwrap());
                    }
                }
            }
        }

        st = st.trim();

        let mut line = Line {
            label: None,
            kind: LineKind::NoCode,
            comment: None,
        };

        // The last component that might be an arg.
        let mut ctx = ParseContext::default();

        loop {
            match find_separator(st) {
                None => {
                    if !st.is_empty() {
                        ctx.append_part().push_str(st);
                    }
                    break;
                }
                Some((prev, Separator::Comment, after)) => {
                    line.comment = Some(after.to_string());
                    if !prev.is_empty() {
                        ctx.append_part().push_str(prev);
                    }
                    break;
                },
                Some((prev, Separator::Label, after)) => {
                    if prev.is_empty() {
                        return Err(Error::EmptyLabel);
                    }
                    let previous_label = line.label.replace(prev.to_string());
                    if previous_label.is_some() {
                        // No two labels in one statement.
                        return Err(Error::SecondLabel);
                    }
                    st = after;
                },
                Some((prev, Separator::Name, after)) => {
                    if !prev.is_empty() {
                        ctx.append_part().push_str(prev);
                    }
                    st = after;
                },
                Some((prev, Separator::Argument, after)) => {
                    if !prev.is_empty() {
                        ctx.append_part().push_str(prev);
                    }

                    ctx.add_argument()?;
                    st = after;
                },
                Some((prev, Separator::Memory, after)) => {
                    let closing = after.find(')').ok_or_else(|| Error::NoClosingParen)?;
                    let (memory_arg, after) = st.split_at(prev.len() + 1 + closing);

                    ctx.append_part().push_str(memory_arg);
                    st = after;
                },
            }
        }

        ctx.finalize();

        line.kind = LineKind::from_name_parts(ctx.name_parts)?;
        line.kind.add_argument(ctx.arguments)?;

        Ok(line)
    }
}

impl LineKind {
    pub fn as_statement(&self) -> Option<&Statement> {
        match self {
            LineKind::Statement(stmt) => Some(stmt),
            _ => None,
        }
    }

    pub fn as_directive(&self) -> Option<&Directive> {
        match self {
            LineKind::Directive(directive) => Some(directive),
            _ => None,
        }
    }

    fn from_solo_name(st: String) -> Self {
        if st.starts_with('.') {
            LineKind::Directive(Directive {
                name: st[1..].to_string(),
                arguments: vec![]
            })
        } else {
            LineKind::Statement(Statement {
                mnemonic: vec![st],
                arguments: vec![],
            })
        }
    }

    fn from_name_parts(mut args: Vec<String>) -> Result<Self, Error> {
        if args.is_empty() {
            return Ok(LineKind::NoCode);
        }

        let mut base = Self::from_solo_name(args[0].clone());
        args.drain(..1).for_each(drop);
        if !args.is_empty() {
            base.add_name_parts(args)?;
        }
        Ok(base)
    }

    fn add_name_parts(&mut self, arg: impl IntoIterator<Item=String>) -> Result<(), Error> {
        match self {
            // Directives have no prefix part.
            LineKind::Directive(_) => Err(Error::DirectivesHaveSingleName),
            LineKind::Statement(stmt) => Ok(stmt.add_prefix_or_name(arg)),
            LineKind::NoCode => Err(Error::OpcodeWithoutCode),
        }
    }

    fn add_argument(&mut self, arg: impl IntoIterator<Item=String>) -> Result<(), Error> {
        match self {
            LineKind::Directive(directive) => Ok(directive.add_argument(arg)),
            LineKind::Statement(stmt) => stmt.add_argument(arg),
            LineKind::NoCode => Err(Error::ArgumentWithoutCode),
        }
    }
}

impl Directive {
    fn add_argument(&mut self, descriptor: impl IntoIterator<Item=String>) {
        self.arguments.extend(descriptor)
    }
}
impl Statement {
    fn add_prefix_or_name(&mut self, name: impl IntoIterator<Item=String>) {
        self.mnemonic.extend(name)
    }

    fn add_argument(&mut self, descriptor: impl IntoIterator<Item=String>) -> Result<(), Error> {
        descriptor
            .into_iter()
            .try_for_each(|descriptor| {
                let argument = descriptor.parse()?;
                Ok(self.arguments.push(argument))
            })
    }
}

fn separator(ch: char) -> bool {
    false
    || ch == ','
    || ch == ';'
    || ch == ':'
    || ch == '('
    || ch == ' '
}

#[derive(Debug)]
enum Separator {
    Argument,
    Comment,
    Label,
    Memory,
    Name,
}

fn find_separator(st: &str) -> Option<(&str, Separator, &str)> {
    st.find(separator).map(|idx| {
        let sep = match &st[idx..idx+1] {
            "," => Separator::Argument,
            ";" => Separator::Comment,
            ":" => Separator::Label,
            "(" => Separator::Memory,
            " " => Separator::Name,
            _ => unreachable!(),
        };

        (&st[..idx], sep, &st[idx+1..])
    })
}

impl str::FromStr for Argument {
    type Err = Error;

    fn from_str(st: &str) -> Result<Self, Error> {
        let st = st.trim();
        if st.contains('(') {
            Ok(Argument::Memory(st.parse()?))
        } else if st.starts_with('%') {
            Ok(Argument::Register(st[1..].to_string()))
        } else {
            let constant = st.parse()?;
            Ok(Argument::Immediate(Value::Const(constant)))
        }
    }
}

impl str::FromStr for Memory {
    type Err = Error;

    fn from_str(st: &str) -> Result<Self, Error> {
        unimplemented!("Memory argument parsing")
    }
}

impl str::FromStr for Constant {
    type Err = Error;
    fn from_str(st: &str) -> Result<Self, Error> {
        let value = st.parse()
            .map_err(|_| Error::InvalidImmediateValue)?;
        Ok(Constant { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        let simple: Line = "syscall".parse().unwrap();
        let statement = simple.kind.as_statement().unwrap();
        assert_eq!(statement.mnemonic, vec!["syscall".to_string()]);
        assert_eq!(&statement.arguments[..], &[][..]);

        let mov: Line = "mov %eax, %ebx".parse().unwrap();
        let statement = mov.kind.as_statement().unwrap();
        assert_eq!(statement.mnemonic, vec!["mov".to_string()]);
        assert_eq!(statement.arguments, vec![Argument::Register("eax".into()), Argument::Register("ebx".into())]);

        let locked: Line = "lock mov %eax, %ebx".parse().unwrap();
        let statement = locked.kind.as_statement().unwrap();
        assert_eq!(statement.mnemonic, vec!["lock".to_string(), "mov".to_string()]);
        assert_eq!(statement.arguments, vec![Argument::Register("eax".into()), Argument::Register("ebx".into())]);

        let label: Line = "1: mov %eax, %ebx".parse().unwrap();
        let statement = label.kind.as_statement().unwrap();
        assert_eq!(label.label, Some("1".into()));
        assert_eq!(statement.mnemonic, vec!["mov".to_string()]);
        assert_eq!(statement.arguments, vec![Argument::Register("eax".into()), Argument::Register("ebx".into())]);

        let comment: Line = "mov %eax, %ebx ;Oh my".parse().unwrap();
        let statement = comment.kind.as_statement().unwrap();
        assert_eq!(comment.comment, Some("Oh my".into()));
        assert_eq!(statement.mnemonic, vec!["mov".to_string()]);
        assert_eq!(statement.arguments, vec![Argument::Register("eax".into()), Argument::Register("ebx".into())]);

        let directive: Line = ".word 1, 2".parse().unwrap();
        let directive = directive.kind.as_directive().unwrap();
        assert_eq!(directive.name, "word".to_string());
        assert_eq!(directive.arguments, vec!["1".to_string(), " 2".to_string()]);
    }
}
