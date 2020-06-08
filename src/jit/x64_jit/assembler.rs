use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::RegisterMap;
use crate::jit::x64_jit::register_allocator::*;

impl MacroAssembler {
  pub fn emit_function_call(&mut self, function_pointer: i32, register_map: &RegisterMap, frame_pointer: i32) {
    let mut stack_pointer = frame_pointer;
    for i in MacroAssembler::caller_saved_regs() {
      stack_pointer += self.emit_conditional_push_reg(register_map, i);
    }
    let stack_misalignment = stack_pointer % 16;
    if stack_misalignment != 0 {
      self.emit_addq_ir(-stack_misalignment, X64_RSP);
      stack_pointer += stack_misalignment;
    }
    self.emit_callq_m64_offset(X64_RSP, function_pointer + stack_pointer);
    if stack_misalignment != 0 {
      self.emit_addq_ir(stack_misalignment, X64_RSP);
      stack_pointer -= stack_misalignment;
    }
    for &i in MacroAssembler::caller_saved_regs().iter().rev() {
      stack_pointer += self.emit_conditional_pop_reg(register_map, i);
    }
    assert_eq!(stack_pointer, frame_pointer);
  }
  pub fn emit_push_reg(&mut self, reg: u32) -> i32 {
    self.emit_push_r64(reg);
    8
  }
  pub fn emit_pop_reg(&mut self, reg: u32) -> i32 {
    self.emit_pop_r64(reg);
    -8
  }
  pub fn emit_conditional_pop_reg(&mut self, register_map: &RegisterMap, reg: u32) -> i32 {
    if register_map.gpr_is_bound(reg) {
      self.emit_pop_reg(reg)
    } else {
      0
    }
  }
  pub fn emit_conditional_push_reg(&mut self, register_map: &RegisterMap, reg: u32) -> i32 {
    if register_map.gpr_is_bound(reg) {
      self.emit_push_reg(reg)
    } else {
      0
    }
  }
}
