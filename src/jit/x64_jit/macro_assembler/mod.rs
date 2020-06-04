use std::io;
use memmap::MmapMut;
use crate::register::BitTwiddle;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::x64_jit::register_allocator::X64_RSP;

mod stack;
mod mov;
mod alu;

pub struct MacroAssembler {
  buffer: Vec<u8>,
}

impl MacroAssembler {
  const REX: u8 = 0x40;
  const REXB: u8 = 0x41;
  const REXX: u8 = 0x42;
  const REXR: u8 = 0x44;
  const REXRB: u8 = 0x45;
  const REXW: u8 = 0x48;
  const REXWB: u8 = 0x49;
  pub fn new() -> Self {
    let mut masm = MacroAssembler {
      buffer: Vec::new(),
    };
    masm.save_reserved_registers();
    masm
  }
  fn save_reserved_registers(&mut self) {
    self.emit_push_r64(12);
    self.emit_push_r64(13);
    self.emit_push_r64(14);
    self.emit_push_r64(15);
  }
  fn load_reserved_registers(&mut self) {
    self.emit_pop_r64(15);
    self.emit_pop_r64(14);
    self.emit_pop_r64(13);
    self.emit_pop_r64(12);
  }
  pub fn len(&self) -> usize {
    self.buffer.len()
  }
  pub fn compile_buffer(&mut self) -> io::Result<JIT_Fn> {
    self.load_reserved_registers();
    self.emit_ret();
    //println!("compiled a {} byte function", self.buffer.len());
    let mut mmap = MmapMut::map_anon(self.buffer.len())?;
    mmap.copy_from_slice(&self.buffer);
    let mmap = mmap.make_exec()?;
    let addr = mmap.as_ptr();
    unsafe {
      let function = std::mem::transmute::<*const u8, fn()>(addr);
      Ok(JIT_Fn::new(mmap, function))
    }
  }
  fn emit_conditional_rexb(&mut self, reg: u32) {
    if reg.nth_bit_bool(3) {
      self.buffer.push(MacroAssembler::REXB);
    };
  }
  fn emit_conditional_rexrb(&mut self, reg1: u32, reg2: u32) {
    let r = reg1.nth_bit(3) as u8;
    let b = reg2.nth_bit(3) as u8;
    if (r | b) != 0 {
      self.buffer.push(MacroAssembler::REX | b | r << 2);
    };
  }
  fn conditional_rexb(reg: u32) -> u8 {
    reg.nth_bit(3) as u8
  }
  fn conditional_rexr(reg: u32) -> u8 {
    (reg.nth_bit(3) << 2) as u8
  }
  fn emit_16bit_prefix(&mut self) {
    self.buffer.push(0x66);
  }
  fn emit_ret(&mut self) {
    self.buffer.push(0xc3);
  }
  fn emit_nop(&mut self) {
    self.buffer.push(0x90);
  }
  fn emit_imm16(&mut self, imm16: u16) {
    imm16.to_ne_bytes().iter().for_each(|&b| {
      self.buffer.push(b);
    });
  }
  fn emit_imm32(&mut self, imm32: u32) {
    imm32.to_ne_bytes().iter().for_each(|&b| {
      self.buffer.push(b);
    });
  }
  fn emit_imm64(&mut self, imm64: u64) {
    imm64.to_ne_bytes().iter().for_each(|&b| {
      self.buffer.push(b);
    });
  }
  #[cfg(test)]
  fn all_regs() -> Vec<u32> {
    (0..=15).collect()
  }
  pub fn free_regs() -> Vec<u32> {
    (0..=15).filter(|&x| x != X64_RSP).collect()
  }
}
