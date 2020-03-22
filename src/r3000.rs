use crate::register::Register;

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
  pub fn reset(&mut self) {
    self.pc = Register::new(0xbfc0_0000);
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
