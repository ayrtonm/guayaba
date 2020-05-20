use std::io;
use memmap::{Mmap, MmapMut};

pub struct MacroAssembler {
  buffer: Vec<u8>,
  mmap: Option<Mmap>,
}

impl MacroAssembler {
  pub fn new() -> Self {
    MacroAssembler {
      buffer: Vec::new(),
      mmap: None,
    }
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
  fn emit_mov_r64(&mut self, r: u32, imm64: u64) {
    self.buffer.push(0x48);
    self.buffer.push(0xb8);
    self.emit_imm64(imm64);
  }
  fn emit_call_r(&mut self, r: u32) {
    self.buffer.push(0xff);
    self.buffer.push(0xd0);
  }
  pub fn test(&mut self, addr: u64) {
    self.emit_mov_r64(0, addr);
    self.emit_call_r(0);
  }
  pub fn compile_buffer(&mut self) -> io::Result<fn()> {
    self.buffer.push(0xc3);//RET
    let mut mmap = MmapMut::map_anon(self.buffer.len())?;
    mmap.copy_from_slice(&self.buffer);
    self.mmap = Some(mmap.make_exec()?);
    let addr = self.mmap.as_ref().map(|mmap| mmap.as_ptr()).unwrap();
    unsafe {
      Ok(std::mem::transmute::<*const u8, fn()>(addr))
    }
  }
}
