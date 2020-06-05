use std::cmp::max;
use std::collections::HashSet;
use crate::console::Console;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisters;
use crate::jit::insn::MIPSRegister;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::register::BitTwiddle;


pub type X64RegNum = u32;
pub const X64_RAX: X64RegNum = 0;
pub const X64_RCX: X64RegNum = 1;
pub const X64_RDX: X64RegNum = 2;
pub const X64_RBX: X64RegNum = 3;
pub const X64_RSP: X64RegNum = 4;
pub const X64_RBP: X64RegNum = 5;
pub const X64_RSI: X64RegNum = 6;
pub const X64_RDI: X64RegNum = 7;
pub const X64_R8: X64RegNum = 8;
pub const X64_R9: X64RegNum = 9;
pub const X64_R10: X64RegNum = 10;
pub const X64_R11: X64RegNum = 11;
pub const X64_R12: X64RegNum = 12;
pub const X64_R13: X64RegNum = 13;
pub const X64_R14: X64RegNum = 14;
pub const X64_R15: X64RegNum = 15;

#[derive(Debug)]
pub struct Mapping {
  x64_reg: X64RegNum,
  mips_reg: MIPSRegister,
}

impl Mapping {
  fn new_from_tuple(tuple: (X64RegNum, MIPSRegister)) -> Mapping {
    Mapping {
      x64_reg: tuple.0,
      mips_reg: tuple.1,
    }
  }
  pub fn x64_reg(&self) -> X64RegNum {
    self.x64_reg
  }
  pub fn mips_reg(&self) -> MIPSRegister {
    self.mips_reg
  }
}

#[derive(Debug)]
pub struct RegisterMap {
  mappings: Vec<Mapping>,
}

impl RegisterMap {
  pub fn new(tagged_opcodes: &Vec<Insn>) -> Self {
    let mips_registers = tagged_opcodes.registers_by_frequency();
    let mappings: Vec<_> = MacroAssembler::free_regs().into_iter()
                                        .zip(mips_registers.into_iter())
                                        .map(|t| Mapping::new_from_tuple(t))
                                        .collect();
    RegisterMap { mappings }
  }
  pub fn overflow_registers(&self) -> i32 {
    if self.mappings.len() < 16 {
      0
    } else {
      (self.mappings.len() - 15) as i32
    }
  }
  pub fn mappings(&self) -> &Vec<Mapping> {
    &self.mappings
  }
  pub fn contains_x64(&self, x64_reg: X64RegNum) -> bool {
    self.mappings.iter().any(|map| map.x64_reg == x64_reg)
  }
  pub fn mips_to_x64(&self, mips_reg: MIPSRegister) -> X64RegNum {
    match self.mappings.iter().find(|&map| map.mips_reg == mips_reg) {
      Some(map) => map.x64_reg,
      None => unreachable!("{:#?}", mips_reg),
    }
  }
}

impl MacroAssembler {
  pub fn load_registers(&mut self, register_map: &RegisterMap, console: &Console) {
    let mips_reg_addr = console.r3000.reg_ptr() as u64;
    self.emit_movq_ir(mips_reg_addr, X64_R15);
    self.emit_push_r64(15);
    for mapping in register_map.mappings() {
      let mips_reg_idx = 4 * (mapping.mips_reg() as u64 - 1);
      let x64_reg = mapping.x64_reg();
      self.emit_movl_mr_offset(X64_R15, x64_reg, mips_reg_idx as i32);
    }
  }
  pub fn save_registers(&mut self, register_map: &RegisterMap, console: &Console) {
    self.emit_pop_r64(15);
    for mapping in register_map.mappings() {
      let mips_reg_idx = 4 * (mapping.mips_reg() as u64 - 1);
      let x64_reg = mapping.x64_reg();
      self.emit_movl_rm_offset(x64_reg, X64_R15, mips_reg_idx as i32);
    }
  }
}
