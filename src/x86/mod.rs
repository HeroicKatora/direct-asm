//! Actual x86 assembler.
//!
//! Use 
use crate::Assembler;
use crate::att;

use std::collections::HashMap;
use dynasm::{DynasmData, Ident, Number, NumericRepr, State, Stmt, Value};
use dynasm::arch::{Arch, x64::{self, ast}};

pub struct DynasmX86 {
    statements: Vec<Stmt>,
    data: DynasmData,
    arch: x64::Archx64,
}

#[derive(Debug)]
pub enum Error {
    InvalidX64Register,
    UnsupportedArgument,
    UnsupportedDirective,
}

#[derive(Debug, Default)]
struct DynasmLine {
    set_features: Option<Vec<String>>,
    instruction: Option<x64::InstructionX64>,
    // FIXME: labels as static relocations.
}

impl DynasmX86 {
    pub fn new() -> Self {
        DynasmX86 {
            statements: vec![],
            data: DynasmData {
                current_arch: Box::new(x64::Archx64::default()),
                aliases: HashMap::default(),
            },
            // FIXME: this or the other x64::Archx64 is probably stale
            arch: x64::Archx64::default(),
        }
    }

    fn state(&mut self) -> (State<'_>, &'_ x64::Archx64) {
        let state = State {
            stmts: &mut self.statements,
            target: self.arch.name(),
            file_data: &mut self.data,
        };

        (state, &self.arch)
    }

    fn generate_instruction_bytes(&self) -> Vec<u8> {
        let mut instructions = Vec::new();
        for stmt in &self.statements {
            match stmt {
                Stmt::Const(Value::Number(value)) => value.write_le_bytes(&mut instructions),
                Stmt::Extend(slice) => instructions.extend_from_slice(slice),
                // Injected expressions can not yet occur.
                | Stmt::Const(Value::Expr(_)) 
                | Stmt::ExprExtend(_) 
                | Stmt::DynamicLabel(_) 
                | Stmt::Stmt(_) => unreachable!(),
                _ => unimplemented!(),
            }
        }
        instructions
    }
}

impl DynasmLine {
    /// Convert an att input line to dynasm input statements.
    fn convert(att: att::Line) -> Result<DynasmLine, Error> {
        let mut line = DynasmLine::default();
        match att.kind {
            att::LineKind::Directive(directive) => {
                match directive.name.as_str() {
                    "features" => line.set_features = Some(directive.arguments.clone()),
                    _ => return Err(Error::UnsupportedDirective),
                }
            },
            att::LineKind::Statement(stmt) => {
                let idents = stmt.mnemonic
                    .into_iter()
                    .map(|name| Ident { name })
                    .collect();
                let args = stmt.arguments
                    .into_iter()
                    .map(Self::convert_argument)
                    .collect::<Result<Vec<_>, _>>()?;
                line.instruction = Some(x64::InstructionX64 {
                    inst: x64::ast::Instruction { idents },
                    args,
                });
            },
            att::LineKind::NoCode => (),
        }
        Ok(line)
    }

    fn convert_argument(arg: att::Argument) -> Result<ast::CleanArg, Error> {
        match arg {
            att::Argument::Register(reg) => {
                let (reg_id, size) = x64::parser::X64_REGISTER_MAP.get(reg.as_str())
                    .copied()
                    .ok_or_else(|| Error::InvalidX64Register)?;
                let kind = ast::RegKind::Static(reg_id);
                let reg = ast::Register { size, kind };
                Ok(ast::CleanArg::Direct { reg })
            },
            att::Argument::Memory(_) => {
                // FIXME.
                Err(Error::UnsupportedArgument)
            },
            att::Argument::Immediate(att::Value { value }) => {
                Ok(ast::CleanArg::Immediate {
                    value: Value::Number(Number::from_u64_and_repr(value as u64, NumericRepr::I64)),
                })
            },
        }
    }
}

impl Assembler for DynasmX86 {
    fn assemble(&mut self, input: &str) -> Vec<u8> {
        use x64::AssembleX64;

        let (mut state, arch) = self.state();

        for line in input.lines() {
            let line = DynasmLine::convert(line.parse().unwrap()).unwrap();

            if let Some(features) = line.set_features {
                state.file_data.current_arch.set_features(&features);
            }

            if let Some(instruction) = line.instruction {
                state.compile_instruction(arch, instruction).unwrap();
            }
        }

        self.generate_instruction_bytes()
    }
}
