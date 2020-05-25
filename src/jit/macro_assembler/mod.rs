use std::io;
use memmap::MmapMut;
use crate::register::BitTwiddle;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::x64_jit::register_allocator::RegisterMap;

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
    self.emit_ret();
    let mut mmap = MmapMut::map_anon(self.buffer.len())?;
    mmap.copy_from_slice(&self.buffer);
    let mmap = mmap.make_exec()?;
    let addr = mmap.as_ptr();
    let name = addr as u64;
    Ok(JIT_Fn::new(mmap, name))
  }
  pub fn emit_swap(&mut self, x64_reg: u32) {
    //rolq $32, %r
    let prefix = match x64_reg.nth_bit_bool(3) {
      true => 0x48,
      false => 0x49,
    };
    let specify_reg = 0xc0 + (x64_reg as u8 & 0x07);
    self.buffer.push(prefix);
    self.buffer.push(0xc1);
    self.buffer.push(specify_reg);
    self.buffer.push(0x20);
  }
  pub fn load_registers(&mut self, register_map: &RegisterMap) {
  }
  pub fn save_registers(&mut self, register_map: &RegisterMap) {
  }
  fn emit_ret(&mut self) {
    self.buffer.push(0xc3);//RET
  }


  pub fn emit_call(&mut self, function_addr: u64, arg_addr: u64) {
    self.emit_mov_r64(function_addr);
    self.emit_mov_rdi(arg_addr);
    //subq $8 %rsp for stack alignment
    self.buffer.push(0x48);
    self.buffer.push(0x83);
    self.buffer.push(0xec);
    self.buffer.push(0x08);
    self.emit_call_r();
    //addq $8 %rsp to undo stack alignment fix
    self.buffer.push(0x48);
    self.buffer.push(0x83);
    self.buffer.push(0xc4);
    self.buffer.push(0x08);
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
