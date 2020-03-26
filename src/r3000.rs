use crate::register::Register;

//different types of register names
//these are for improved readability when doing delayed register writes
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Name {
  pc,
  hi,
  lo,
  rn(u32),
}

//this represents a delayed write operation
#[derive(Debug)]
pub struct DelayedWrite {
  register_name: Name,
  value: Register,
  cycles: u32,
}

impl DelayedWrite {
  pub fn new(register_name: Name, value: Register, cycles: u32) -> Self {
    DelayedWrite {
      register_name,
      value,
      cycles,
    }
  }
  pub fn decrease_cycles(&mut self) {
    self.cycles -= 1;
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
  pub fn nth_reg(&self, idx: u32) -> &Register {
    assert!(idx < 32);
    let idx = idx as usize;
    match idx {
      0 => {
        &R3000::ZERO
      },
      _ => {
        &self.general_registers[idx - 1]
      },
    }
  }
  //this methods returns a mutable reference to R1 through R31
  //R0 is always mapped to zero so it doesn't make sense here
  pub fn nth_reg_mut(&mut self, idx: u32) -> &mut Register {
    assert!((idx < 32) && (idx > 0));
    let idx = idx as usize;
    &mut self.general_registers[idx - 1]
  }
  //general purpose MIPS registers also have names we can use
  pub fn ra(&self) -> &Register {
    self.nth_reg(31)
  }
  pub fn ra_mut(&mut self) -> &mut Register {
    self.nth_reg_mut(31)
  }
  //these are the special purpose MIPS registers
  pub fn pc(&self) -> &Register {
    &self.pc
  }
  pub fn pc_mut(&mut self) -> &mut Register {
    &mut self.pc
  }
  pub fn flush_write_cache(&mut self, operations: &mut Vec<DelayedWrite>) {
    operations.iter()
              .filter(|write| write.cycles == 0)
              .for_each(|write| self.do_write(write));
    operations.retain(|write| write.cycles != 0);
    operations.iter_mut()
              .for_each(|write| write.decrease_cycles());
  }
  fn do_write(&mut self, operation: &DelayedWrite) {
    match operation.register_name {
      Name::pc => {
        self.pc = operation.value;
      },
      Name::hi => {
        self.hi = operation.value;
      },
      Name::lo => {
        self.lo = operation.value;
      },
      Name::rn(idx) => {
        assert!((idx < 32) && (idx > 0));
        let idx = idx as usize;
        self.general_registers[idx - 1] = operation.value;
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
      *r3000.nth_reg_mut(i) = i + 31;
    }
    for i in 1..=31 {
      assert_eq!(*r3000.nth_reg(i), (31 + i) as u32);
    }
  }

  #[test]
  fn flush_cache() {
    let mut cache = Vec::new();
    cache.push(DelayedWrite::new(Name::rn(1), 4, 10));
    cache.push(DelayedWrite::new(Name::rn(3), 6, 0));
    cache.push(DelayedWrite::new(Name::rn(2), 5, 1));
    let mut r3000 = R3000::new();
    r3000.flush_write_cache(&mut cache);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache[0].cycles, 9);
    assert_eq!(cache[0].value, 4);
    assert_eq!(cache[1].cycles, 0);
    assert_eq!(cache[1].value, 5);
  }
}
