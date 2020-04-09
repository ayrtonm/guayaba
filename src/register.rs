use std::ops::*;

pub type Register = u32;

pub trait Parts {
  fn word(&self) -> Register;
  fn half(&self) -> Register;
  fn byte(&self) -> Register;
  fn half_sign_extended(&self) -> Register;
  fn byte_sign_extended(&self) -> Register;
}

pub trait Aliases {
  fn sra(&self, rhs: Register) -> Register;
  fn and(&self, rhs: Register) -> Register;
  fn or(&self, rhs: Register) -> Register;
  fn xor(&self, rhs: Register) -> Register;
  fn nor(&self, rhs: Register) -> Register;
  fn compare(&self, rhs: Register) -> Register;
  fn signed_compare(&self, rhs: Register) -> Register;
  fn nth_bit(&self, n: Register) -> Register;
}

impl Parts for Register{
  fn word(&self) -> Register {
    *self
  }
  fn half(&self) -> Register {
    self & 0x0000_ffff
  }
  fn byte(&self) -> Register {
    self & 0x0000_00ff
  }
  fn half_sign_extended(&self) -> Register {
    let ret = self.half();
    match ret >> 15 {
      0 => {
        ret
      },
      1 => {
        ret | 0xffff_0000
      },
      _ => {
        unreachable!("")
      },
    }
  }
  fn byte_sign_extended(&self) -> Register {
    let ret = self.byte();
    match ret >> 7 {
      0 => {
        ret
      },
      1 => {
        ret | 0xffff_ff00
      },
      _ => {
        unreachable!("")
      },
    }
  }
}

impl Aliases for Register {
  fn nth_bit(&self, n: Register) -> Register {
    (self >> n) & 1
  }
  fn sra(&self, rhs: Register) -> Register {
    (*self as i32).shr(rhs) as Register
  }
  fn and(&self, rhs: Register) -> Register {
    self.bitand(rhs)
  }
  fn or(&self, rhs: Register) -> Register {
    self.bitor(rhs)
  }
  fn xor(&self, rhs: Register) -> Register {
    self.bitxor(rhs)
  }
  fn nor(&self, rhs: Register) -> Register {
    self.bitor(rhs).not()
  }
  fn compare(&self, rhs: Register) -> Register {
    if *self < rhs {
      1
    } else {
      0
    }
  }
  fn signed_compare(&self, rhs: Register) -> Register {
    if (*self as i32) < (rhs as i32) {
      1
    } else {
      0
    }
  }
}
