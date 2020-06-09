use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::RegisterMap;
use crate::jit::x64_jit::register_allocator::*;

#[deny(unused_must_use)]
impl MacroAssembler {
  pub fn emit_save_caller_regs(&mut self, register_map: &RegisterMap, frame_pointer: i32) -> (i32, i32) {
    let mut stack_pointer = frame_pointer;
    for i in MacroAssembler::caller_saved_regs() {
      stack_pointer += self.emit_conditional_push_reg(register_map, i);
    }
    let stack_misalignment = stack_pointer % 16;
    if stack_misalignment != 0 {
      self.emit_addq_ir(-stack_misalignment, X64_RSP);
      stack_pointer += stack_misalignment;
    }
    (stack_pointer, stack_misalignment)
  }
  pub fn emit_load_caller_regs(&mut self, register_map: &RegisterMap, frame_pointer: i32, stack_misalignment: i32) -> i32 {
    let mut stack_pointer = frame_pointer;
    if stack_misalignment != 0 {
      self.emit_addq_ir(stack_misalignment, X64_RSP);
      stack_pointer -= stack_misalignment;
    }
    for &i in MacroAssembler::caller_saved_regs().iter().rev() {
      stack_pointer += self.emit_conditional_pop_reg(register_map, i);
    }
    stack_pointer
  }
  #[must_use]
  pub fn emit_load_arg_from_mips_mut(&mut self, arg_num: u32, mips_reg: u32, register_map: &mut RegisterMap, stack_pointer: i32) -> i32 {
    self.emit_load_arg_from_mips(arg_num, mips_reg, register_map, stack_pointer);
    self.emit_push_reg(arg_num)
  }
  pub fn emit_load_arg_from_mips(&mut self, arg_num: u32, mips_reg: u32,register_map: &mut RegisterMap, stack_pointer: i32) {
    self.emit_swap_mips_registers(mips_reg, arg_num, register_map, stack_pointer);
  }
  #[must_use]
  pub fn emit_load_arg_from_memory(&mut self, arg_num: u32, ptr: i32, register_map: &RegisterMap) -> i32 {
    let mut stack_offset = 0;
    stack_offset += self.emit_conditional_push_reg(register_map, arg_num);
    self.emit_movq_mr_offset(X64_RSP, arg_num, ptr + stack_offset);
    stack_offset
  }
  #[must_use]
  pub fn emit_push_reg(&mut self, reg: u32) -> i32 {
    self.emit_push_r64(reg);
    8
  }
  #[must_use]
  pub fn emit_pop_reg(&mut self, reg: u32) -> i32 {
    self.emit_pop_r64(reg);
    -8
  }
  #[must_use]
  pub fn emit_conditional_pop_reg(&mut self, register_map: &RegisterMap, reg: u32) -> i32 {
    if register_map.gpr_is_bound(reg) {
      self.emit_pop_reg(reg)
    } else {
      0
    }
  }
  #[must_use]
  pub fn emit_conditional_push_reg(&mut self, register_map: &RegisterMap, reg: u32) -> i32 {
    if register_map.gpr_is_bound(reg) {
      self.emit_push_reg(reg)
    } else {
      0
    }
  }
}
