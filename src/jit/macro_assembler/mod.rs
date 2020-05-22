use std::io;
use memmap::MmapMut;
use crate::jit::jit_fn::JIT_Fn;

pub struct MacroAssembler {
  buffer: Vec<u8>,
}

impl MacroAssembler {
  pub fn new() -> Self {
    MacroAssembler {
      buffer: Vec::new(),
    }
  }
  pub fn compile_buffer(&mut self) -> io::Result<JIT_Fn> {
    self.buffer.push(0xc3);//RET
    let mut mmap = MmapMut::map_anon(self.buffer.len())?;
    mmap.copy_from_slice(&self.buffer);
    let mmap = mmap.make_exec()?;
    let addr = mmap.as_ptr();
    unsafe {
      let f = std::mem::transmute::<*const u8, fn()>(addr);
      Ok(JIT_Fn::new(mmap, f))
    }
  }
  pub fn emit_call(&mut self, function_addr: u64, arg_addr: u64) {
    self.emit_mov_r64(function_addr);
    self.emit_mov_rdi(arg_addr);
    self.emit_call_r();
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
  //hardcoded to mov rcx, imm64 for now
  fn emit_mov_r64(&mut self, imm64: u64) {
    self.buffer.push(0x48);
    self.buffer.push(0xb8);
    self.emit_imm64(imm64);
  }
  fn emit_mov_rdi(&mut self, imm64: u64) {
    self.buffer.push(0x48);
    self.buffer.push(0xbf);
    self.emit_imm64(imm64);
  }
  //hardcoded to call rcx for now
  fn emit_call_r(&mut self) {
    self.buffer.push(0xff);
    self.buffer.push(0xd0);
  }
}
