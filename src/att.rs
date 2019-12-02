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
pub enum LineKind {
    Directive(Directive),
    Statement(Statement),
    NoCode,
}

pub struct Directive {
    pub name: String,
    pub arguments: Vec<String>,
}

pub struct Statement {
    /// Mnemonic of the op code and prefixes.
    pub mnemonic: Vec<String>,
    pub arguments: Vec<Argument>,
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
                    self.arguments.last_mut().unwrap().push_str(" ");
                    self.arguments.last_mut().unwrap()
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
        base.add_name_parts(args)?;
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
            let value = st.parse().ok().ok_or_else(|| Error::InvalidImmediateValue)?;
            Ok(Argument::Immediate(Value { value }))
        }
    }
}

impl str::FromStr for Memory {
    type Err = Error;

    fn from_str(st: &str) -> Result<Self, Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        let simple: Line = "syscall".parse().unwrap();
        let mov: Line = "mov %eax, %ebx".parse().unwrap();
        let locked: Line = "lock mov %eax, %ebx".parse().unwrap();
        let label: Line = "1: mov %eax, %ebx".parse().unwrap();
        let comment: Line = "mov %eax, %ebx ; Oh my".parse().unwrap();
    }
}
