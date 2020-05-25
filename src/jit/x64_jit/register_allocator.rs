use std::collections::HashSet;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisters;
use crate::jit::insn::MIPSRegister;

#[derive(Debug)]
struct X64Register {
  //registers are somewhat arbitrarily mapped as follows
  //rax - 1
  //rbx - 2
  //rcx - 3
  //rdx - 4
  //rsi - 5
  //rdi - 6
  //rbp - 7
  //r8  - 8
  //r9  - 9
  //r10 - 10
  //r11 - 11
  //r12 - 12
  //r13 - 13
  //r14 - 14
  //r15 - 15
  reg_num: u32,
  //shelved means that the MIPS register is held in the upper 32 bits of the x86 register
  shelved: bool,
}

#[derive(Debug)]
struct Mapping {
  x64_reg: X64Register,
  mips_reg: MIPSRegister,
}

#[derive(Debug)]
pub struct RegisterMap {
  mappings: HashSet<Mapping>,
}

impl RegisterMap {
  const x64_registers_by_priority: [X64Register; 28] = [
    X64Register { reg_num: 1, shelved: false },
    X64Register { reg_num: 2, shelved: false },
    X64Register { reg_num: 3, shelved: false },
    X64Register { reg_num: 4, shelved: false },
    X64Register { reg_num: 5, shelved: false },
    X64Register { reg_num: 6, shelved: false },
    X64Register { reg_num: 7, shelved: false },
    X64Register { reg_num: 8, shelved: false },
    X64Register { reg_num: 9, shelved: false },
    X64Register { reg_num: 10, shelved: false },
    X64Register { reg_num: 11, shelved: false },
    X64Register { reg_num: 12, shelved: false },
    X64Register { reg_num: 13, shelved: false },
    X64Register { reg_num: 14, shelved: false },
    X64Register { reg_num: 1, shelved: true },
    X64Register { reg_num: 2, shelved: true },
    X64Register { reg_num: 3, shelved: true },
    X64Register { reg_num: 4, shelved: true },
    X64Register { reg_num: 5, shelved: true },
    X64Register { reg_num: 6, shelved: true },
    X64Register { reg_num: 7, shelved: true },
    X64Register { reg_num: 8, shelved: true },
    X64Register { reg_num: 9, shelved: true },
    X64Register { reg_num: 10, shelved: true },
    X64Register { reg_num: 11, shelved: true },
    X64Register { reg_num: 12, shelved: true },
    X64Register { reg_num: 13, shelved: true },
    X64Register { reg_num: 14, shelved: true },
  ];
  pub fn new(tagged_opcodes: &Vec<Insn>) -> Self {
    let mips_registers = tagged_opcodes.registers_by_frequency();
    let zipped: Vec<_> = mips_registers.iter().zip(&RegisterMap::x64_registers_by_priority).collect();
    panic!("{:#?}", zipped);
    RegisterMap {
      mappings: Default::default(),
    }
  }
}
