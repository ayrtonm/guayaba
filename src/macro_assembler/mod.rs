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
  fn push_imm16(&mut self, imm16: u16) {
    self.buffer.push((imm16 & 0xff) as u8);
    self.buffer.push(((imm16 >> 8) & 0xff) as u8);
  }
  fn push_imm32(&mut self, imm32: u32) {
    self.buffer.push((imm32 & 0xff) as u8);
    self.buffer.push(((imm32 >> 8) & 0xff) as u8);
    self.buffer.push(((imm32 >> 16) & 0xff) as u8);
    self.buffer.push(((imm32 >> 24) & 0xff) as u8);
  }
  fn push_imm64(&mut self, imm64: u64) {
    self.buffer.push((imm64 & 0xff) as u8);
    self.buffer.push(((imm64 >> 8) & 0xff) as u8);
    self.buffer.push(((imm64 >> 16) & 0xff) as u8);
    self.buffer.push(((imm64 >> 24) & 0xff) as u8);
    self.buffer.push(((imm64 >> 32) & 0xff) as u8);
    self.buffer.push(((imm64 >> 40) & 0xff) as u8);
    self.buffer.push(((imm64 >> 48) & 0xff) as u8);
    self.buffer.push(((imm64 >> 56) & 0xff) as u8);
  }
  pub fn test(&mut self, addr: u64) {
    self.buffer.push(0x48);
    self.buffer.push(0xb8);
    self.push_imm64(addr);
    self.buffer.push(0xff);
    self.buffer.push(0xd0);
  }
  pub fn compile_buffer(&mut self) -> io::Result<fn() -> u64> {
    self.buffer.push(0xc3);//RET
    let mut mmap = MmapMut::map_anon(self.buffer.len())?;
    mmap.copy_from_slice(&self.buffer);
    self.mmap = Some(mmap.make_exec()?);
    let addr = self.mmap.as_ref().map(|mmap| mmap.as_ptr()).unwrap();
    unsafe {
      Ok(std::mem::transmute::<*const u8, fn() -> u64>(addr))
    }
  }
}
