use std::ops::*;

pub type Register = u32;

pub trait Parts {
  fn word(&self) -> Register;
  fn half(&self) -> Register;
  fn byte(&self) -> Register;
  fn half_sign_extended(&self) -> Register;
  fn byte_sign_extended(&self) -> Register;
}

pub trait BitManipulation {
  fn set(&mut self, n: Register) -> &mut Self;
  fn set_mask(&mut self, mask: Register) -> &mut Self;
  fn clear(&mut self, n: Register) -> &mut Self;
  fn clear_mask(&mut self, mask: Register) -> &mut Self;
  fn lowest_bits(&self, n: Register) -> Register;
  fn upper_bits(&self, n: Register) -> Register;
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
  fn nth_bit_bool(&self, n: Register) -> bool;
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
    (self.half() as i16) as Register
  }
  fn byte_sign_extended(&self) -> Register {
    (self.byte() as i8) as Register
  }
}

impl BitManipulation for Register {
  fn set(&mut self, n: Register) -> &mut Self {
    *self |= (1 << n);
    self
  }
  fn set_mask(&mut self, mask: Register) -> &mut Self {
    *self |= mask;
    self
  }
  fn clear(&mut self, n: Register) -> &mut Self {
    *self &= !(1 << n);
    self
  }
  fn clear_mask(&mut self, mask: Register) -> &mut Self {
    *self &= !mask;
    self
  }
  //ands self with the lowest n bits
  fn lowest_bits(&self, n: Register) -> Register {
    *self & (((1 as u64) << n) - 1) as u32
  }
  //ands self with the highest n bits
  fn upper_bits(&self, n: Register) -> Register {
    *self & !(((1 as u64) << (32 - n)) - 1) as u32
  }
}

impl Aliases for Register {
  fn nth_bit(&self, n: Register) -> Register {
    (self >> n) & 1
  }
  fn nth_bit_bool(&self, n: Register) -> bool {
    self.nth_bit(n) == 1
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
