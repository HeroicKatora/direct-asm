//! Actual x86 assembler.
//!
//! Use 
use crate::Assembler;
use crate::att::Line;

use std::collections::HashMap;
use dynasm::{DynasmData, State, Stmt};
use dynasm::arch::{Arch, x64};

pub struct DynasmX86 {
    statements: Vec<Stmt>,
    data: DynasmData,
    arch: x64::Archx64,
}

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
}

impl DynasmLine {
    /// Convert an att input line to dynasm input statements.
    fn convert(att: Line) -> Option<DynasmLine> {
        unimplemented!()
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

        // TODO: convert statements to byte vec
        unimplemented!()
    }
}
