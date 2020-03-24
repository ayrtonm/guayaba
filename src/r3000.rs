use std::ops::Index;
use std::ops::IndexMut;

use crate::register::Register;

#[derive(Debug)]
pub enum RegisterName {
  general(GeneralRegisterName),
  pc,
  hi,
  lo,
}

#[derive(Debug)]
pub enum GeneralRegisterName {
  at,
  vn(usize),
  an(usize),
  tn0(usize),
  sn(usize),
  tn1(usize),
  kn(usize),
  gp,
  sp,
  fp,
  ra,
}

#[derive(Debug)]
pub struct Write {
  register_name: RegisterName,
  value: u32,
}

impl Write {
  pub fn new(register_name: RegisterName, value: u32) -> Self {
    Write {
      register_name,
      value: value,
    }
  }
}

#[derive(Debug,Default)]
struct GeneralRegisters([Register; 31]);

impl Index<&GeneralRegisterName> for GeneralRegisters {
  type Output = Register;

  fn index(&self, idx: &GeneralRegisterName) -> &Self::Output {
    match idx {
      GeneralRegisterName::at => {
        &(self.0)[0]
      },
      GeneralRegisterName::vn(n) => {
        &(self.0)[1 + n]
      },
      GeneralRegisterName::an(n) => {
        &(self.0)[1 + n]
      },
      GeneralRegisterName::tn0(n) => {
        &(self.0)[1 + n]
      },
      GeneralRegisterName::sn(n) => {
        &(self.0)[1 + n]
      },
      GeneralRegisterName::tn1(n) => {
        &(self.0)[1 + n]
      },
      GeneralRegisterName::kn(n) => {
        &(self.0)[1 + n]
      },
      GeneralRegisterName::gp => {
        &(self.0)[27]
      },
      GeneralRegisterName::sp => {
        &(self.0)[28]
      },
      GeneralRegisterName::fp => {
        &(self.0)[29]
      },
      GeneralRegisterName::ra => {
        &(self.0)[30]
      },
    }
  }
}
impl IndexMut<&GeneralRegisterName> for GeneralRegisters {
  fn index_mut(&mut self, idx: &GeneralRegisterName) -> &mut Self::Output {
    match idx {
      GeneralRegisterName::at => {
        &mut (self.0)[0]
      },
      GeneralRegisterName::vn(n) => {
        &mut (self.0)[1 + n]
      },
      GeneralRegisterName::an(n) => {
        &mut (self.0)[1 + n]
      },
      GeneralRegisterName::tn0(n) => {
        &mut (self.0)[1 + n]
      },
      GeneralRegisterName::sn(n) => {
        &mut (self.0)[1 + n]
      },
      GeneralRegisterName::tn1(n) => {
        &mut (self.0)[1 + n]
      },
      GeneralRegisterName::kn(n) => {
        &mut (self.0)[1 + n]
      },
      GeneralRegisterName::gp => {
        &mut (self.0)[27]
      },
      GeneralRegisterName::sp => {
        &mut (self.0)[28]
      },
      GeneralRegisterName::fp => {
        &mut (self.0)[29]
      },
      GeneralRegisterName::ra => {
        &mut (self.0)[30]
      },
    }
  }
}

#[derive(Debug,Default)]
pub struct R3000 {
  general_registers: GeneralRegisters,
  pc: Register,
  hi: Register,
  lo: Register,
}

impl R3000 {
  pub fn new() -> Self {
    let general_registers = Default::default();
    let pc = Register::new(0xbfc0_0000);
    let hi = Default::default();
    let lo = Default::default();
    R3000 {
      general_registers,
      pc,
      hi,
      lo,
    }
  }
  pub fn flush_write_cache(&mut self, operations: &Vec<Write>) {
    for write in operations {
      self.do_write(write);
    }
  }
  fn do_write(&mut self, operation: &Write) {
    match &operation.register_name {
      RegisterName::pc => {
        self.pc = Register::new(operation.value);
      },
      RegisterName::hi => {
        self.hi = Register::new(operation.value);
      },
      RegisterName::lo => {
        self.lo = Register::new(operation.value);
      },
      RegisterName::general(name) => {
        self.general_registers[name] = Register::new(operation.value);
      },
    }
  }
  pub fn pc(&mut self) -> &mut Register {
    &mut self.pc
  }
  pub fn ra(&mut self) -> &mut Register {
    &mut self.general_registers[&GeneralRegisterName::ra]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn initial_values() {
    let r3000 = R3000::new();
    assert_eq!(r3000.pc.get_value(), 0xbfc0_0000);
  }

  #[test]
  fn set_register() {
    let mut r3000 = R3000::new();
    *r3000.pc() = Register::new(2);
    assert_eq!(r3000.pc.get_value(), 2);
  }
}
