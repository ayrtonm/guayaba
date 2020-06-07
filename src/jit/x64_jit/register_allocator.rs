use std::cmp::max;
use std::collections::HashSet;
use crate::console::Console;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisterFrequency;
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
pub enum Location {
  X64Register(X64RegNum),
  Stack(i32),
}
#[derive(Debug)]
pub struct Mapping {
  x64_reg: Location,
  mips_reg: MIPSRegister,
}

impl Mapping {
  fn new_from_tuple(tuple: (Location, MIPSRegister)) -> Mapping {
    Mapping {
      x64_reg: tuple.0,
      mips_reg: tuple.1,
    }
  }
  pub fn x64_reg(&self) -> Option<X64RegNum> {
    match self.x64_reg {
      Location::X64Register(x) => Some(x),
      Location::Stack(_) => None,
    }
  }
  pub fn stack_location(&self) -> Option<i32> {
    match self.x64_reg {
      Location::Stack(offset) => Some(offset),
      Location::X64Register(_) => None,
    }
  }
  pub fn mips_reg(&self) -> MIPSRegister {
    self.mips_reg
  }
  fn remap_location(&mut self, mips_reg: MIPSRegister) {
    self.mips_reg = mips_reg;
  }
  fn mips_loaded(&self) -> bool {
    self.x64_reg().is_some()
  }
}

#[derive(Debug)]
pub struct RegisterMap {
  mappings: Vec<Mapping>,
}

impl RegisterMap {
  pub fn new(tagged_opcodes: &Vec<Insn>) -> Self {
    let stack_locations = (0..=15).map(|offset| Location::Stack(offset))
                                 .collect::<Vec<_>>();
    let mips_registers = tagged_opcodes.registers_by_frequency();
    let mappings: Vec<_> = MacroAssembler::free_regs().iter()
                                        .map(|&x| Location::X64Register(x))
                                        .chain(stack_locations)
                                        .zip(mips_registers.into_iter())
                                        .map(|t| Mapping::new_from_tuple(t))
                                        .collect();
    RegisterMap { mappings }
  }
  pub fn count_overflow_registers(&self) -> usize {
    self.mappings.iter().filter(|&map| !map.mips_loaded()).count()
  }
  pub fn loaded_mappings(&self) -> Vec<&Mapping> {
    self.mappings.iter().filter(|map| map.mips_loaded()).collect()
  }
  pub fn overflow_mappings(&self) -> Vec<&Mapping> {
    self.mappings.iter().filter(|map| !map.mips_loaded()).collect()
  }
  fn mappings_mut(&mut self) -> &mut Vec<Mapping> {
    &mut self.mappings
  }
  pub fn contains_x64(&self, x64_reg: X64RegNum) -> bool {
    self.mappings.iter().any(|map| map.x64_reg() == Some(x64_reg))
  }
  pub fn mips_to_x64(&self, mips_reg: MIPSRegister) -> Option<X64RegNum> {
    match self.mappings.iter().find(|&map| map.mips_reg == mips_reg) {
      Some(map) => map.x64_reg(),
      None => unreachable!("tried using unmapped MIPS register R{}", mips_reg),
    }
  }
  pub fn mips_stack_location(&self, mips_reg: MIPSRegister) -> Option<i32> {
    match self.mappings.iter().find(|&map| map.mips_reg == mips_reg) {
      Some(map) => map.stack_location(),
      None => unreachable!("tried using unmapped MIPS register R{}", mips_reg),
    }
  }
  fn location_to_mips(&self, location: &Location) -> Option<MIPSRegister> {
    match *location {
      Location::X64Register(reg) => {
        self.mappings.iter()
                     .find(|&map| map.x64_reg() == Some(reg))
                     .map(|mapping| mapping.mips_reg())
      },
      Location::Stack(offset) => {
        self.mappings.iter()
                     .find(|&map| map.stack_location() == Some(offset))
                     .map(|mapping| mapping.mips_reg())
      },
    }
  }
  fn remap_location(&mut self, location: &Location, mips_reg: MIPSRegister) {
    match *location {
      Location::X64Register(reg) => {
        self.mappings_mut()
            .into_iter()
            .find(|map| map.x64_reg() == Some(reg))
            .map(|mut map| {
              map.remap_location(mips_reg);
            });
      },
      Location::Stack(offset) => {
        self.mappings_mut()
            .into_iter()
            .find(|map| map.stack_location() == Some(offset))
            .map(|mut map| {
              map.remap_location(mips_reg);
            });
      },
    }
  }
  //TODO: add some check to make sure that loc1 or loc2 is loaded
  pub fn swap_mappings(&mut self, loc1: Location, loc2: Location) {
    let mips_reg1 = self.location_to_mips(&loc1);
    let mips_reg2 = self.location_to_mips(&loc2);
    mips_reg1.map(|value_reg1| {
      self.remap_location(&loc2, value_reg1);
    });
    mips_reg2.map(|value_reg2| {
      self.remap_location(&loc1, value_reg2);
    });
  }
}

impl MacroAssembler {
  pub fn load_registers(&mut self, register_map: &RegisterMap, console: &Console) {
    let mips_reg_addr = console.r3000.reg_ptr() as u64;
    self.emit_movq_ir(mips_reg_addr, X64_R15);
    self.emit_push_r64(X64_R15);
    self.emit_addq_ir(-(register_map.count_overflow_registers() as i32) * 4, X64_RSP);
    for mapping in register_map.overflow_mappings() {
      let stack_offset = mapping.stack_location().expect("MIPS register should be mapped to the stack") * 4;
      let mips_reg_idx = 4 * (mapping.mips_reg() - 1) as i32;
      self.emit_movl_mr_offset(X64_R15, X64_R14, mips_reg_idx);
      self.emit_movl_rm_offset(X64_R14, X64_RSP, stack_offset);
    }
    for mapping in register_map.loaded_mappings() {
      let x64_reg = mapping.x64_reg().expect("MIPS register should be mapped to an x64 register");
      let mips_reg_idx = 4 * (mapping.mips_reg() - 1) as i32;
      self.emit_movl_mr_offset(X64_R15, x64_reg, mips_reg_idx);
    }
  }
  pub fn save_registers(&mut self, register_map: &RegisterMap, console: &Console) {
    let mut stack_offset = register_map.count_overflow_registers() as i32 * 4;
    let r15_in_use = register_map.loaded_mappings().iter().any(|&map| map.x64_reg() == Some(X64_R15));
    if r15_in_use {
      self.emit_push_r64(X64_R15);
      stack_offset += 8;
    }
    self.emit_movq_mr_offset(X64_RSP, X64_R15, stack_offset);
    for &mapping in register_map.loaded_mappings().iter().filter(|&map| map.x64_reg() != Some(X64_R15)) {
      let x64_reg = mapping.x64_reg().expect("MIPS register should be mapped to an x64 register");
      let mips_reg_idx = 4 * (mapping.mips_reg() - 1) as i32;
      self.emit_movl_rm_offset(x64_reg, X64_R15, mips_reg_idx);
    }
    for mapping in register_map.overflow_mappings() {
      let mut stack_location = mapping.stack_location().expect("MIPS register should be mapped to the stack") * 4;
      if r15_in_use {
        stack_location += 8;
      }
      let mips_reg_idx = 4 * (mapping.mips_reg() - 1) as i32;
      self.emit_movl_mr_offset(X64_RSP, X64_R14, stack_location);
      self.emit_movl_rm_offset(X64_R14, X64_R15, mips_reg_idx);
    }
    match register_map.loaded_mappings().iter().find(|&map| map.x64_reg() == Some(X64_R15)) {
      Some(mapping) => {
        let mips_reg_idx = 4 * (mapping.mips_reg() - 1) as i32;
        self.emit_pop_r64(X64_R14);
        stack_offset -= 8;
        self.emit_movl_rm_offset(X64_R14, X64_R15, mips_reg_idx);
      },
      _ => (),
    }
    self.emit_addq_ir(stack_offset + 8, X64_RSP);
  }
}
