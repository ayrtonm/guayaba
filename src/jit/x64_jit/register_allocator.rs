use std::collections::HashSet;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisters;
use crate::jit::insn::MIPSRegister;

//the x64's r15 won't have fixed MIPS registers in them
//if we have to work with more than 28 MIPS registers in a block then r15 will
//point to the excess registers (up to 3) and they'll be swapped with the first
//14 registers as needed
enum X64RegNum {
  RAX = 0,
  RCX = 1,
  RDX = 2,
  RBX = 3,
  RSP = 4,
  RBP = 5,
  RSI = 6,
  RDI = 7,
  R8  = 8,
  R9  = 9,
  R10 = 10,
  R11 = 11,
  R12 = 12,
  R13 = 13,
  R14 = 14,
}

#[derive(Debug)]
pub struct X64Register {
  reg_num: u32,
  //shelved means that the MIPS register is held in the upper 32 bits of the x64 register
  shelved: bool,
}

#[derive(Debug)]
pub struct Mapping {
  x64_reg: X64Register,
  mips_reg: MIPSRegister,
}

impl X64Register {
  fn is_accessible(&self) -> bool {
    !self.shelved
  }
  pub fn num(&self) -> u32 {
    self.reg_num
  }
}

impl Mapping {
  fn new_from_tuple(tuple: (X64Register, MIPSRegister)) -> Mapping {
    Mapping {
      x64_reg: tuple.0,
      mips_reg: tuple.1,
    }
  }
  pub fn x64_reg(&self) -> &X64Register {
    &self.x64_reg
  }
  pub fn mips_reg(&self) -> MIPSRegister {
    self.mips_reg
  }
  fn is_accessible(&self) -> bool {
    self.x64_reg.is_accessible()
  }
}

#[derive(Debug)]
pub struct RegisterMap {
  mappings: Vec<Mapping>,
}

impl RegisterMap {
  pub fn new(tagged_opcodes: &Vec<Insn>) -> Self {
    let mips_registers = tagged_opcodes.registers_by_frequency();
    let mut x64_registers = Vec::new();
    for b in &[false, true] {
      let valid_regs: Vec<_> = (0..=14).filter(|&x| x != X64RegNum::RSP as u32).collect();
      for i in valid_regs {
        x64_registers.push(X64Register { reg_num: i, shelved: *b });
      }
    };
    let mappings: Vec<_> = x64_registers.into_iter()
                                        .zip(mips_registers.into_iter())
                                        .map(|t| Mapping::new_from_tuple(t))
                                        .collect();
    RegisterMap { mappings }
  }
  pub fn mappings(&self) -> &Vec<Mapping> {
    &self.mappings
  }
  fn mips_to_x64(&self, mips_reg: MIPSRegister) -> &X64Register {
    match self.mappings.iter().find(|&map| map.mips_reg == mips_reg) {
      Some(map) => &map.x64_reg,
      None => unreachable!("{:#?}", mips_reg),
    }
  }
  fn is_accessible(&self, mips_reg: MIPSRegister) -> bool {
    self.mips_to_x64(mips_reg).is_accessible()
  }
  //this returns whether the MIPS register was loaded or not to determine if we
  //need to emit JIT code to swap the 32-bit words in the x64 register
  pub fn load_mips(&mut self, mips_reg: MIPSRegister) -> Option<u32> {
    if !self.is_accessible(mips_reg) {
      let x64_reg = self.mips_to_x64(mips_reg).reg_num;
      self.swap_x64(x64_reg);
      Some(x64_reg)
    } else {
      None
    }
  }
  //this only updates the RegisterMap to reflect the state of the mapping after
  //swapping the x64 register halves. Note that this does not emit any JIT code
  fn swap_x64(&mut self, x64_reg_num: u32) {
    self.mappings
        .iter_mut()
        .filter(|map| map.x64_reg.reg_num == x64_reg_num)
        .for_each(|mut map| {
          map.x64_reg.shelved = !map.x64_reg.shelved;
        });
  }
}
