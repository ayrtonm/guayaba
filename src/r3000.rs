use std::collections::VecDeque;
use crate::register::Register;

//different types of register names
//these are for improved readability when doing delayed register writes
#[derive(Debug,PartialEq)]
pub enum Name {
  Pc,
  Hi,
  Lo,
  Rn(u32),
}

//this represents a delayed write operation
#[derive(Debug)]
pub struct DelayedWrite {
  register_name: Name,
  value: Register,
}

impl DelayedWrite {
  pub fn new(register_name: Name, value: Register) -> Self {
    DelayedWrite {
      register_name,
      value,
    }
  }
  pub fn name(&self) -> &Name {
    &self.register_name
  }
  pub fn value(&self) -> Register {
    self.value
  }
}

pub struct MutRegister<'a> {
  value: &'a mut Register,
  name: Name,
}

impl<'a> MutRegister<'a> {
  pub fn new(value: &'a mut Register, name: Name) -> Self {
    MutRegister {
      value, name
    }
  }
}

pub trait MaybeSet {
  fn maybe_set(self, value: Register) -> Option<Name>;
}

//this is for the main MIPS processor registers
impl<'a> MaybeSet for Option<MutRegister<'a>> {
  fn maybe_set(self, value: Register) -> Option<Name> {
    self.map(|reg| {*reg.value = value; reg.name})
  }
}

//this is for the coprocessor registers
impl MaybeSet for Option<&mut Register> {
  fn maybe_set(self, value: Register) -> Option<Name> {
    self.map(|reg| *reg = value);
    None
  }
}

#[derive(Debug,Default)]
pub struct R3000 {
  general_registers: [Register; 31],
  pc: Register,
  hi: Register,
  lo: Register,
}

impl R3000 {
  const ZERO: Register = 0;
  pub fn new() -> Self {
    let general_registers = Default::default();
    let pc = 0xbfc0_0000;
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
  pub fn nth_reg(&self, idx: u32) -> Register {
    assert!(idx < 32);
    let idx = idx as usize;
    match idx {
      0 => {
        R3000::ZERO
      },
      _ => {
        self.general_registers[idx - 1]
      },
    }
  }
  //this methods returns a mutable reference to R1 through R31
  //R0 is always mapped to zero so it doesn't make sense here
  pub fn nth_reg_mut(&mut self, idx: u32) -> Option<MutRegister> {
    assert!(idx < 32);
    let idx = idx as usize;
    match idx {
      0 => {
        None
      },
      _ => {
        Some(MutRegister::new(&mut self.general_registers[idx - 1], Name::Rn(idx as u32)))
      },
    }
  }
  //general purpose MIPS registers also have names we can use
  pub fn ra(&self) -> Register {
    self.nth_reg(31)
  }
  pub fn ra_mut(&mut self) -> Option<MutRegister> {
    self.nth_reg_mut(31)
  }
  //these are the special purpose MIPS registers
  pub fn pc(&self) -> Register {
    self.pc
  }
  pub fn pc_mut(&mut self) -> &mut Register {
    &mut self.pc
  }
  pub fn lo(&self) -> Register {
    self.lo
  }
  pub fn lo_mut(&mut self) -> &mut Register {
    &mut self.lo
  }
  pub fn hi(&self) -> Register {
    self.hi
  }
  pub fn hi_mut(&mut self) -> &mut Register {
    &mut self.hi
  }
  pub fn flush_write_cache(&mut self, operations: &mut VecDeque<DelayedWrite>,
                           modified_register: &mut Option<Name>) {
    match operations.pop_front() {
      Some(write) => {
        match modified_register {
          Some(name) => {
            if *name != write.register_name {
              self.do_write(&write);
            }
          },
          None => {
            self.do_write(&write);
          },
        }
      },
      None => {
      },
    };
    *modified_register = None;
  }
  fn do_write(&mut self, operation: &DelayedWrite) {
    match operation.register_name {
      Name::Pc => {
        self.pc = operation.value;
      },
      Name::Hi => {
        self.hi = operation.value;
      },
      Name::Lo => {
        self.lo = operation.value;
      },
      Name::Rn(idx) => {
        self.nth_reg_mut(idx).maybe_set(operation.value);
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
    assert_eq!(r3000.pc, 0xbfc0_0000);
  }

  #[test]
  fn set_register() {
    let mut r3000 = R3000::new();
    *r3000.pc_mut() = 2;
    assert_eq!(r3000.pc, 2);
  }

  #[test]
  fn general_registers() {
    let mut r3000 = R3000::new();
    for i in 1..=31 {
      r3000.nth_reg_mut(i).maybe_set(i + 31);
    }
    for i in 1..=31 {
      assert_eq!(r3000.nth_reg(i), (31 + i) as u32);
    }
  }

  #[test]
  fn flush_cache() {
    let mut cache = VecDeque::new();
    cache.push_back(DelayedWrite::new(Name::Rn(1), 4));
    cache.push_back(DelayedWrite::new(Name::Rn(3), 6));
    cache.push_back(DelayedWrite::new(Name::Rn(2), 5));
    let mut r3000 = R3000::new();
    r3000.flush_write_cache(&mut cache, &mut None);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache[0].value, 6);
    assert_eq!(cache[1].value, 5);
  }

  #[test]
  fn delayed_load_priority() {
    let mut cache = VecDeque::new();
    let mut r3000 = R3000::new();

    cache.push_back(DelayedWrite::new(Name::Rn(4), 10));
    r3000.nth_reg_mut(4).maybe_set(20);
    let mut modified = Some(Name::Rn(4));
    r3000.flush_write_cache(&mut cache, &mut modified);
    assert_eq!(r3000.nth_reg(4), 20);

    cache.push_back(DelayedWrite::new(Name::Rn(4), 30));
    modified = Some(Name::Rn(3));
    r3000.flush_write_cache(&mut cache, &mut modified);
    assert_eq!(r3000.nth_reg(4), 30);

    cache.push_back(DelayedWrite::new(Name::Rn(4), 40));
    modified = None;
    r3000.flush_write_cache(&mut cache, &mut modified);
    assert_eq!(r3000.nth_reg(4), 40);
  }
}
