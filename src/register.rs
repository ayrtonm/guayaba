use std::ops::*;

pub trait BitTwiddle {
  fn word(&self) -> u32;
  fn half(&self) -> u32;
  fn byte(&self) -> u32;
  fn half_sign_extended(&self) -> u32;
  fn byte_sign_extended(&self) -> u32;
  fn word_align(&self) -> u32;
  fn half_align(&self) -> u32;
  fn range(&self, n1: u32, n2: u32) -> u32;
  fn lowest_bits(&self, n: u32) -> u32;
  fn upper_bits(&self, n: u32) -> u32;
  fn upper_bits_in_place(&self, n: u32) -> u32;

  fn set(&mut self, n: u32) -> &mut Self;
  fn set_mask(&mut self, mask: u32) -> &mut Self;
  fn clear(&mut self, n: u32) -> &mut Self;
  fn clear_mask(&mut self, mask: u32) -> &mut Self;

  fn sra(&self, rhs: u32) -> u32;
  fn and(&self, rhs: u32) -> u32;
  fn or(&self, rhs: u32) -> u32;
  fn xor(&self, rhs: u32) -> u32;
  fn nor(&self, rhs: u32) -> u32;
  fn compare(&self, rhs: u32) -> u32;
  fn signed_compare(&self, rhs: u32) -> u32;
  fn nth_bit(&self, n: u32) -> u32;
  fn nth_bit_bool(&self, n: u32) -> bool;
}

impl BitTwiddle for u32{
  fn word(&self) -> u32 {
    *self
  }
  fn half(&self) -> u32 {
    self & 0x0000_ffff
  }
  fn byte(&self) -> u32 {
    self & 0x0000_00ff
  }
  fn half_sign_extended(&self) -> u32 {
    (self.half() as i16) as u32
  }
  fn byte_sign_extended(&self) -> u32 {
    (self.byte() as i8) as u32
  }
  fn word_align(&self) -> u32 {
    self & 0xffff_fffc
  }
  fn half_align(&self) -> u32 {
    self & 0xffff_fffe
  }
  fn range(&self, n1: u32, n2: u32) -> u32 {
    assert!(n2 > n1);
    self.lowest_bits(n2 + 1).upper_bits(32 - n1)
  }
  //ands self with the lowest n bits
  fn lowest_bits(&self, n: u32) -> u32 {
    *self & (((1 as u64) << n) - 1) as u32
  }
  fn upper_bits(&self, n: u32) -> u32 {
    *self >> (32 - n)
  }
  //ands self with the highest n bits
  fn upper_bits_in_place(&self, n: u32) -> u32 {
    *self & !(((1 as u64) << (32 - n)) - 1) as u32
  }


  fn set(&mut self, n: u32) -> &mut Self {
    *self |= 1 << n;
    self
  }
  fn set_mask(&mut self, mask: u32) -> &mut Self {
    *self |= mask;
    self
  }
  fn clear(&mut self, n: u32) -> &mut Self {
    *self &= !(1 << n);
    self
  }
  fn clear_mask(&mut self, mask: u32) -> &mut Self {
    *self &= !mask;
    self
  }
  fn nth_bit(&self, n: u32) -> u32 {
    (self >> n) & 1
  }
  fn nth_bit_bool(&self, n: u32) -> bool {
    self.nth_bit(n) == 1
  }
  fn sra(&self, rhs: u32) -> u32 {
    (*self as i32).shr(rhs) as u32
  }
  fn and(&self, rhs: u32) -> u32 {
    self.bitand(rhs)
  }
  fn or(&self, rhs: u32) -> u32 {
    self.bitor(rhs)
  }
  fn xor(&self, rhs: u32) -> u32 {
    self.bitxor(rhs)
  }
  fn nor(&self, rhs: u32) -> u32 {
    self.bitor(rhs).not()
  }
  fn compare(&self, rhs: u32) -> u32 {
    if *self < rhs {
      1
    } else {
      0
    }
  }
  fn signed_compare(&self, rhs: u32) -> u32 {
    if (*self as i32) < (rhs as i32) {
      1
    } else {
      0
    }
  }
}
