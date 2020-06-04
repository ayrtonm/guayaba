use std::io;
use memmap::MmapMut;
use crate::register::BitTwiddle;
use crate::console::Console;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::x64_jit::register_allocator::RegisterMap;
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
    MacroAssembler {
      buffer: Vec::new(),
    }
  }
  pub fn len(&self) -> usize {
    self.buffer.len()
  }
  pub fn compile_buffer(&mut self) -> io::Result<JIT_Fn> {
    self.emit_ret();
    //println!("compiled a {} byte function", self.buffer.len());
    let mut mmap = MmapMut::map_anon(self.buffer.len())?;
    mmap.copy_from_slice(&self.buffer);
    let mmap = mmap.make_exec()?;
    let addr = mmap.as_ptr();
    let name = addr as u64;
    Ok(JIT_Fn::new(mmap, name))
  }
  pub fn emit_rotate(&mut self, x64_reg: u32) {
    //rolq $32, %r
    let prefix = 0x49 - x64_reg.nth_bit(3) as u8;
    let specify_reg = 0xc0 + (x64_reg as u8 & 0x07);
    self.buffer.push(prefix);
    self.buffer.push(0xc1);
    self.buffer.push(specify_reg);
    self.buffer.push(0x20);
  }
  pub fn load_registers(&mut self, register_map: &RegisterMap, console: &Console) {
    let init_size = self.buffer.len();
    for mapping in register_map.mappings() {
      let mips_reg_idx = 4 * (mapping.mips_reg() as u64 - 1);
      let mips_reg_addr = console.r3000.reg_ptr() as u64 + mips_reg_idx;
      let x64_reg = mapping.x64_reg().num();
      //movq mips_reg_addr, %r14
      self.emit_movq_ir(mips_reg_addr, X64RegNum::R14 as u32);
      //movl (%r14), %exx
      self.emit_movl_mr(x64_reg, X64RegNum::R14 as u32);
    }
    println!("added {} bytes to the function in load_registers", self.buffer.len() - init_size);
  }
  pub fn save_registers(&mut self, register_map: &RegisterMap, console: &Console) {
    let init_size = self.buffer.len();
    self.emit_movq_ir(console.r3000.reg_ptr() as u64, X64RegNum::R14 as u32);
    for mapping in register_map.mappings() {
      let mips_reg_idx = 4 * (mapping.mips_reg() - 1);
      self.emit_movl_ir(mips_reg_idx, X64RegNum::R13 as u32);
      //let mips_reg_addr = console.r3000.reg_ptr() as u64 + mips_reg_idx;
      let x64_reg = mapping.x64_reg().num();
      self.emit_movl_rm_sib_offset(x64_reg, X64RegNum::R14 as u32, X64RegNum::R13 as u32, 1, 0);
      ////movq mips_reg_addr, %r14
      //self.emit_movq_ir(mips_reg_addr, X64RegNum::R14 as u32);
      //movl %exx, (%r14)
      //self.emit_movl_rm(x64_reg, X64RegNum::R14 as u32);
    }
    println!("added {} bytes to the function in save_registers", self.buffer.len() - init_size);
  }
  fn emit_rexb(&mut self) {
    self.buffer.push(MacroAssembler::REXB);
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
  //emit an immediate value
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
