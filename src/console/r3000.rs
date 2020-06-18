use std::collections::VecDeque;
use super::Name;
use super::DelayedWrite;
use super::MaybeSet;

pub struct MutReg<'a> {
  value: &'a mut u32,
  name: Name,
}

impl<'a> MutReg<'a> {
  pub fn new(value: &'a mut u32, name: Name) -> Self {
    MutReg {
      value, name
    }
  }
}

impl<'a> MaybeSet for Option<MutReg<'a>> {
  fn maybe_set(self, value: u32) -> Option<Name> {
    self.map(|reg| {*reg.value = value; reg.name})
  }
}

pub struct R3000 {
  //R0, R1-R31, PC, HI, LO in that order
  registers: [u32; 35],
}

impl R3000 {
  const ZERO: u32 = 0;
  pub const PC_IDX: usize = 32;
  const HI_IDX: usize = 33;
  const LO_IDX: usize = 34;
  pub fn new() -> Self {
    let mut registers = [0; 35];
    registers[R3000::PC_IDX] = 0xbfc0_0000;
    R3000 {
      registers,
    }
  }
  //general purpose MIPS registers are referred to as R0..R31
  //this method is used to address registers R0 through R31
  pub fn nth_reg(&self, idx: u32) -> u32 {
    assert!(idx < 32);
    let idx = idx as usize;
    match idx {
      0 => {
        R3000::ZERO
      },
      _ => {
        self.registers[idx]
      },
    }
  }
  //this should only be used in the JIT
  pub fn reg_ptr(&self) -> *const u32 {
    &self.registers[0] as *const u32
  }
  //this methods returns a mutable reference to R1 through R31
  //R0 is always mapped to zero so it doesn't make sense here
  pub fn nth_reg_mut(&mut self, idx: u32) -> Option<MutReg> {
    assert!(idx < 32);
    let idx = idx as usize;
    match idx {
      0 => {
        None
      },
      _ => {
        Some(MutReg::new(&mut self.registers[idx], Name::Rn(idx as u32)))
      },
    }
  }
  //general purpose MIPS registers also have names we can use
  pub fn ra_mut(&mut self) -> Option<MutReg> {
    self.nth_reg_mut(31)
  }
  //these are the special purpose MIPS registers
  pub fn pc(&self) -> u32 {
    self.registers[R3000::PC_IDX]
  }
  pub fn pc_mut(&mut self) -> &mut u32 {
    &mut self.registers[R3000::PC_IDX]
  }
  pub fn lo(&self) -> u32 {
    self.registers[R3000::LO_IDX]
  }
  pub fn lo_mut(&mut self) -> &mut u32 {
    &mut self.registers[R3000::LO_IDX]
  }
  pub fn hi(&self) -> u32 {
    self.registers[R3000::HI_IDX]
  }
  pub fn hi_mut(&mut self) -> &mut u32 {
    &mut self.registers[R3000::HI_IDX]
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
      Name::Hi => {
        *self.hi_mut() = operation.value;
      },
      Name::Lo => {
        *self.lo_mut() = operation.value;
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
    assert_eq!(r3000.pc(), 0xbfc0_0000);
  }

  #[test]
  fn set_register() {
    let mut r3000 = R3000::new();
    *r3000.pc_mut() = 2;
    assert_eq!(r3000.pc(), 2);
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
