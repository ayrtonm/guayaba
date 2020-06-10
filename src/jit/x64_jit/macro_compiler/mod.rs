use std::io;
use crate::jit::insn::Insn;
use crate::jit::jit_fn::JIT_Fn;
use crate::console::Console;
use crate::jit::x64_jit::macro_compiler::macro_assembler::registers::*;
use crate::jit::x64_jit::macro_compiler::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::macro_compiler::register_manager::RegisterManager;

mod register_manager;
mod macro_assembler;

#[must_use]
pub struct StackOffset(isize);

#[deny(unused_must_use)]
pub struct MacroCompiler {
  register_manager: RegisterManager,
  masm: MacroAssembler,
  stack_offset: usize,
}

impl MacroCompiler {
  pub fn new(tagged_opcodes: &Vec<Insn>) -> Self {
    let register_manager = RegisterManager::new(tagged_opcodes);
    let masm = MacroAssembler::new();
    MacroCompiler {
      register_manager,
      masm,
      stack_offset: 0,
    }
  }
  pub fn create_pointers(&mut self, console: &Console) -> StackOffset {
    let mut stack_offset = StackOffset(0);
    stack_offset.0 += self.masm.emit_pushq_i(Console::read_byte_sign_extended as u64);
    stack_offset.0 += self.masm.emit_pushq_i(Console::read_half_sign_extended as u64);
    stack_offset.0 += self.masm.emit_pushq_i(Console::read_byte as u64);
    stack_offset.0 += self.masm.emit_pushq_i(Console::read_half as u64);
    stack_offset.0 += self.masm.emit_pushq_i(Console::read_word as u64);
    stack_offset.0 += self.masm.emit_pushq_i(Console::write_byte as u64);
    stack_offset.0 += self.masm.emit_pushq_i(Console::write_half as u64);
    stack_offset.0 += self.masm.emit_pushq_i(Console::write_word as u64);
    stack_offset.0 += self.masm.emit_pushq_i(console as * const Console as u64);
    stack_offset.0 += self.masm.emit_pushq_i(console.cop0.reg_ptr() as u64);
    stack_offset.0 += self.masm.emit_pushq_i(console.r3000.reg_ptr() as u64);
    stack_offset
  }
  pub fn destroy_pointers(&mut self, num_bytes: isize) -> StackOffset {
    assert!(num_bytes >= 0);
    self.masm.emit_addq_ir(num_bytes as i32, X64_RSP);
    StackOffset(-num_bytes)
  }
  pub fn compile(&mut self) -> io::Result<JIT_Fn> {
    assert_eq!(self.stack_offset, 0);
    self.masm.assemble()
  }
}
