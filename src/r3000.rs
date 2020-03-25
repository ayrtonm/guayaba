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
pub struct Write {
  register_name: Name,
  value: Register,
}

impl Write {
  pub fn new(register_name: Name, value: Register) -> Self {
    Write {
      register_name,
      value: value,
    }
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
  pub fn flush_write_cache(&mut self, operations: Vec<Write>) {
    for write in operations {
      self.do_write(write);
    }
  }
  fn do_write(&mut self, operation: Write) {
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
      Name::rn(name) => {
        let idx = name as usize;
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
}
