use std::ops::Index;
use std::ops::IndexMut;

use crate::register::Register;

#[derive(Debug)]
pub enum Name {
  pc,
  hi,
  lo,
  gpr(General),
}

#[derive(Debug)]
pub enum General {
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
  register_name: Name,
  value: u32,
}

impl Write {
  pub fn new(register_name: Name, value: u32) -> Self {
    Write {
      register_name,
      value: value,
    }
  }
}

#[derive(Debug,Default)]
struct GeneralRegisters([Register; 31]);

impl Index<General> for GeneralRegisters {
  type Output = Register;

  fn index(&self, idx: General) -> &Self::Output {
    match idx {
      General::at => {
        &(self.0)[0]
      },
      General::vn(n) => {
        &(self.0)[1 + n]
      },
      General::an(n) => {
        &(self.0)[3 + n]
      },
      General::tn0(n) => {
        &(self.0)[7 + n]
      },
      General::sn(n) => {
        &(self.0)[15 + n]
      },
      General::tn1(n) => {
        &(self.0)[23 + n]
      },
      General::kn(n) => {
        &(self.0)[25 + n]
      },
      General::gp => {
        &(self.0)[27]
      },
      General::sp => {
        &(self.0)[28]
      },
      General::fp => {
        &(self.0)[29]
      },
      General::ra => {
        &(self.0)[30]
      },
    }
  }
}
impl IndexMut<General> for GeneralRegisters {
  fn index_mut(&mut self, idx: General) -> &mut Self::Output {
    match idx {
      General::at => {
        &mut (self.0)[0]
      },
      General::vn(n) => {
        assert!(n < 2);
        &mut (self.0)[1 + n]
      },
      General::an(n) => {
        assert!(n < 4);
        &mut (self.0)[3 + n]
      },
      General::tn0(n) => {
        assert!(n < 8);
        &mut (self.0)[7 + n]
      },
      General::sn(n) => {
        assert!(n < 8);
        &mut (self.0)[15 + n]
      },
      General::tn1(n) => {
        assert!((n < 10) && (n > 7));
        &mut (self.0)[15 + n]
      },
      General::kn(n) => {
        assert!(n < 2);
        &mut (self.0)[25 + n]
      },
      General::gp => {
        &mut (self.0)[27]
      },
      General::sp => {
        &mut (self.0)[28]
      },
      General::fp => {
        &mut (self.0)[29]
      },
      General::ra => {
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
  //general purpose MIPS registers are referred to as R0..R31
  pub fn nth_reg(&mut self, idx: usize) -> &mut Register {
    assert!((idx < 32) && (idx > 0));
    &mut self.general_registers.0[idx - 1]
  }
  //general purpose MIPS registers also have names we can use
  pub fn ra(&mut self) -> &mut Register {
    &mut self.general_registers[General::ra]
  }
  //these are the special purpose MIPS registers
  pub fn pc(&mut self) -> &mut Register {
    &mut self.pc
  }
  pub fn flush_write_cache(&mut self, operations: Vec<Write>) {
    for write in operations {
      self.do_write(write);
    }
  }
  fn do_write(&mut self, operation: Write) {
    match operation.register_name {
      Name::pc => {
        self.pc = Register::new(operation.value);
      },
      Name::hi => {
        self.hi = Register::new(operation.value);
      },
      Name::lo => {
        self.lo = Register::new(operation.value);
      },
      Name::gpr(name) => {
        self.general_registers[name] = Register::new(operation.value);
      },
    }
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

  #[test]
  fn general_registers() {
    let mut r3000 = R3000::new();
    for i in 1..=31 {
      *r3000.nth_reg(i) = Register::new((i + 31) as u32);
    }
    assert_eq!(r3000.general_registers[General::at].get_value(), 32);
    assert_eq!(r3000.general_registers[General::vn(0)].get_value(), 33);
    assert_eq!(r3000.general_registers[General::vn(1)].get_value(), 34);
    assert_eq!(r3000.general_registers[General::an(0)].get_value(), 35);
    assert_eq!(r3000.general_registers[General::an(1)].get_value(), 36);
    assert_eq!(r3000.general_registers[General::an(2)].get_value(), 37);
    assert_eq!(r3000.general_registers[General::an(3)].get_value(), 38);
    assert_eq!(r3000.general_registers[General::tn0(0)].get_value(), 39);
    assert_eq!(r3000.general_registers[General::tn0(1)].get_value(), 40);
    assert_eq!(r3000.general_registers[General::tn0(2)].get_value(), 41);
    assert_eq!(r3000.general_registers[General::tn0(3)].get_value(), 42);
    assert_eq!(r3000.general_registers[General::tn0(4)].get_value(), 43);
    assert_eq!(r3000.general_registers[General::tn0(5)].get_value(), 44);
    assert_eq!(r3000.general_registers[General::tn0(6)].get_value(), 45);
    assert_eq!(r3000.general_registers[General::tn0(7)].get_value(), 46);
    assert_eq!(r3000.general_registers[General::sn(0)].get_value(), 47);
    assert_eq!(r3000.general_registers[General::sn(1)].get_value(), 48);
    assert_eq!(r3000.general_registers[General::sn(2)].get_value(), 49);
    assert_eq!(r3000.general_registers[General::sn(3)].get_value(), 50);
    assert_eq!(r3000.general_registers[General::sn(4)].get_value(), 51);
    assert_eq!(r3000.general_registers[General::sn(5)].get_value(), 52);
    assert_eq!(r3000.general_registers[General::sn(6)].get_value(), 53);
    assert_eq!(r3000.general_registers[General::sn(7)].get_value(), 54);
    assert_eq!(r3000.general_registers[General::tn1(0)].get_value(), 55);
    assert_eq!(r3000.general_registers[General::tn1(1)].get_value(), 56);
    assert_eq!(r3000.general_registers[General::kn(0)].get_value(), 57);
    assert_eq!(r3000.general_registers[General::kn(1)].get_value(), 58);
    assert_eq!(r3000.general_registers[General::gp].get_value(), 59);
    assert_eq!(r3000.general_registers[General::sp].get_value(), 60);
    assert_eq!(r3000.general_registers[General::fp].get_value(), 61);
    assert_eq!(r3000.general_registers[General::ra].get_value(), 62);
  }
}
