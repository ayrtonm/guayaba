use std::io;
use memmap::MmapMut;
use crate::register::BitTwiddle;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::x64_jit::register_allocator::X64RegNum;

mod stack;
mod mov;
mod alu;

pub struct MacroAssembler {
  buffer: Vec<u8>,
}

impl MacroAssembler {
  pub const MOD11: u8 = 0b1100_0000;
  pub const REXB: u8 = 0x41;
  pub const REXX: u8 = 0x42;
  pub const REXR: u8 = 0x44;
  pub const REXRB: u8 = 0x45;
  pub const REXW: u8 = 0x48;
  pub const REXWB: u8 = 0x49;
  pub fn new() -> Self {
    let mut masm = MacroAssembler {
      buffer: Vec::new(),
    };
    masm.save_reserved_registers();
    masm
  }
  fn save_reserved_registers(&mut self) {
    self.emit_push_r32(12);
    self.emit_push_r32(13);
    self.emit_push_r32(14);
    self.emit_push_r32(15);
  }
  fn load_reserved_registers(&mut self) {
    self.emit_pop_r32(15);
    self.emit_pop_r32(14);
    self.emit_pop_r32(13);
    self.emit_pop_r32(12);
  }
  pub fn len(&self) -> usize {
    self.buffer.len()
  }
  pub fn compile_buffer(&mut self) -> io::Result<JIT_Fn> {
    self.load_reserved_registers();
    self.emit_ret();
    println!("compiled a {} byte function", self.buffer.len());
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
}
