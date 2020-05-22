use std::io;
use crate::jit::insn::Insn;
use crate::jit::x64_jit::block::Block;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::macro_assembler::MacroAssembler;
use crate::console::Console;
use crate::common::*;

impl Block {
  pub(super) fn create_optimized_function(tagged_opcodes: &Vec<Insn>, console: &Console,
                                       logging: bool) -> io::Result<JIT_Fn> {
    let mut masm = MacroAssembler::new();
    Ok(masm.compile_buffer()?)
  }
}
