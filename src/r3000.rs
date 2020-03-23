use crate::register::Register;

#[derive(Debug)]
pub enum RegisterName {
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
  pc,
  hi,
  lo,
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
pub struct R3000 {
  at: Register,
  vn: [Register; 2],
  an: [Register; 4],
  tn0: [Register; 8],
  sn: [Register; 8],
  tn1: [Register; 2],
  kn: [Register; 2],
  gp: Register,
  sp: Register,
  fp: Register,
  ra: Register,
  pc: Register,
  hi: Register,
  lo: Register,
}

impl R3000 {
  pub fn new() -> Self {
    let at = Default::default();
    let vn = Default::default();
    let an = Default::default();
    let tn0 = Default::default();
    let sn = Default::default();
    let tn1 = Default::default();
    let kn = Default::default();
    let gp = Default::default();
    let sp = Default::default();
    let fp = Default::default();
    let ra = Default::default();
    let pc = Register::new(0xbfc0_0000);
    let hi = Default::default();
    let lo = Default::default();
    R3000 {
      at, vn, an, tn0, sn, tn1,
      kn, gp, sp, fp, ra, pc,
      hi, lo,
    }
  }
  pub fn flush_write_cache(&mut self, operations: &Vec<Write>) {
    for write in operations {
      self.write_register(write);
    }
  }
  fn write_register(&mut self, operation: &Write) {
    match operation.register_name {
      RegisterName::at => {
        self.at = Register::new(operation.value);
      },
      RegisterName::vn(idx) => {
        self.vn[idx] = Register::new(operation.value);
      },
      RegisterName::an(idx) => {
        self.an[idx] = Register::new(operation.value);
      },
      RegisterName::tn0(idx) => {
        self.tn0[idx] = Register::new(operation.value);
      },
      RegisterName::sn(idx) => {
        self.sn[idx] = Register::new(operation.value);
      },
      RegisterName::tn1(idx) => {
        self.tn1[idx] = Register::new(operation.value);
      },
      RegisterName::kn(idx) => {
        self.kn[idx] = Register::new(operation.value);
      },
      RegisterName::gp => {
        self.gp = Register::new(operation.value);
      },
      RegisterName::sp => {
        self.sp = Register::new(operation.value);
      },
      RegisterName::fp => {
        self.fp = Register::new(operation.value);
      },
      RegisterName::ra => {
        self.ra = Register::new(operation.value);
      },
      RegisterName::pc => {
        self.pc = Register::new(operation.value);
      },
      RegisterName::hi => {
        self.hi = Register::new(operation.value);
      },
      RegisterName::lo => {
        self.lo = Register::new(operation.value);
      },
    }
  }
  pub fn pc(&mut self) -> &mut Register {
    &mut self.pc
  }
  pub fn ra(&mut self) -> &mut Register {
    &mut self.ra
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
