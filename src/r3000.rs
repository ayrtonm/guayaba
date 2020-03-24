use std::ops::Index;
use std::ops::IndexMut;
use crate::register::Register;

//different types of register names
//these are for improved readability when doing delayed register writes
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Name {
  pc,
  hi,
  lo,
  gpr(General),
}

//these are names for the registers in the general purpose register array
#[allow(non_camel_case_types)]
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

//this represents a delayed write operation
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

fn name_to_idx(name: General) -> u32 {
  let ret = match name {
    General::at => {
      1
    },
    General::vn(n) => {
      assert!(n < 2);
      2 + n
    },
    General::an(n) => {
      assert!(n < 4);
      4 + n
    },
    General::tn0(n) => {
      assert!(n < 8);
      8 + n
    },
    General::sn(n) => {
      assert!(n < 8);
      16 + n
    },
    General::tn1(n) => {
      assert!((n < 10) && (n > 7));
      16 + n
    },
    General::kn(n) => {
      assert!(n < 2);
      26 + n
    },
    General::gp => {
      28
    },
    General::sp => {
      29
    },
    General::fp => {
      30
    },
    General::ra => {
      31
    },
  };
  ret as u32
}

pub fn idx_to_name(idx: u32) -> General {
  let idx = idx as usize;
  match idx {
    1  => {
     General::at
    },
    2..=3 => {
      General::vn(idx - 2)
    },
    4..=7 => {
      General::an(idx - 4)
    },
    8..=15 => {
      General::tn0(idx - 8)
    },
    16..=23 => {
      General::sn(idx - 16)
    },
    24..=25 => {
      General::tn1(idx - 24 + 8)
    },
    26..=27 => {
      General::kn(idx - 26)
    },
    28 => {
     General::gp
    },
    29 => {
      General::sp
    },
    30 => {
      General::fp
    },
    31 => {
      General::ra
    },
    _ => {
      panic!("tried to get name of invalid R{} register", idx);
    }
  }
}

//allow indexing the general purpose register array by name
impl Index<General> for GeneralRegisters {
  type Output = Register;

  fn index(&self, name: General) -> &Self::Output {
    &(self.0)[name_to_idx(name) as usize - 1]
  }
}
impl IndexMut<General> for GeneralRegisters {
  fn index_mut(&mut self, name: General) -> &mut Self::Output {
    &mut (self.0)[name_to_idx(name) as usize - 1]
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
  const ZERO: Register = Register::new(0);
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
  //this method is used to address registers R0 through R31
  pub fn nth_reg(&self, idx: u32) -> &Register {
    assert!(idx < 32);
    let idx = idx as usize;
    match idx {
      0 => {
        &R3000::ZERO
      },
      _ => {
        &self.general_registers.0[idx - 1]
      },
    }
  }
  //this methods returns a mutable reference to R1 through R31
  //R0 is always mapped to zero so it doesn't make sense here
  pub fn nth_reg_mut(&mut self, idx: u32) -> &mut Register {
    assert!((idx < 32) && (idx > 0));
    let idx = idx as usize;
    &mut self.general_registers.0[idx - 1]
  }
  //general purpose MIPS registers also have names we can use
  //these methods are shorthands for using Name and General to address the general
  //purpose register array
  pub fn at(&mut self) -> &mut Register {
    &mut self.general_registers[General::at]
  }
  pub fn vn(&mut self, idx: usize) -> &mut Register {
    &mut self.general_registers[General::vn(idx)]
  }
  pub fn an(&mut self, idx: usize) -> &mut Register {
    &mut self.general_registers[General::an(idx)]
  }
  pub fn tn0(&mut self, idx: usize) -> &mut Register {
    &mut self.general_registers[General::tn0(idx)]
  }
  pub fn sn(&mut self, idx: usize) -> &mut Register {
    &mut self.general_registers[General::sn(idx)]
  }
  pub fn tn1(&mut self, idx: usize) -> &mut Register {
    &mut self.general_registers[General::tn1(idx)]
  }
  pub fn kn(&mut self, idx: usize) -> &mut Register {
    &mut self.general_registers[General::kn(idx)]
  }
  pub fn gp(&mut self) -> &mut Register {
    &mut self.general_registers[General::gp]
  }
  pub fn sp(&mut self) -> &mut Register {
    &mut self.general_registers[General::sp]
  }
  pub fn fp(&mut self) -> &mut Register {
    &mut self.general_registers[General::fp]
  }
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
      *r3000.nth_reg_mut(i) = Register::new((i + 31) as u32);
    }
    for i in 1..=31 {
      assert_eq!(r3000.general_registers[idx_to_name(i)].get_value(), (31 + i) as u32);
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
    assert_eq!(r3000.general_registers[General::tn1(8)].get_value(), 55);
    assert_eq!(r3000.general_registers[General::tn1(9)].get_value(), 56);
    assert_eq!(r3000.general_registers[General::kn(0)].get_value(), 57);
    assert_eq!(r3000.general_registers[General::kn(1)].get_value(), 58);
    assert_eq!(r3000.general_registers[General::gp].get_value(), 59);
    assert_eq!(r3000.general_registers[General::sp].get_value(), 60);
    assert_eq!(r3000.general_registers[General::fp].get_value(), 61);
    assert_eq!(r3000.general_registers[General::ra].get_value(), 62);
  }

  #[test]
  fn register_name_conversion() {
    for i in 1..=31 {
      assert_eq!(i, name_to_idx(idx_to_name(i)));
    }
  }

  #[test]
  #[should_panic]
  fn invalid_register() {
    idx_to_name(32);
  }
}
