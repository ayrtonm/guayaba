use std::io;
use std::collections::HashMap;
use memmap::MmapMut;
use crate::register::BitTwiddle;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::x64_jit::macro_compiler::macro_assembler::registers::*;
use crate::common::WriteArray;

mod stack;
mod mov;
mod jump;
mod call;
mod alu;
mod bt;
mod xchg;
pub mod registers;

pub type Label = usize;
type JITOffset = usize;

pub struct MacroAssembler {
  buffer: Vec<u8>,
  next_label: usize,
  labels_used: HashMap<JITOffset, Label>,
  labels_defined: HashMap<Label, JITOffset>,
}

impl MacroAssembler {
  const REX: u8 = 0x40;
  const REXB: u8 = 0x41;
  const REXX: u8 = 0x42;
  const REXR: u8 = 0x44;
  const REXRB: u8 = 0x45;
  const REXW: u8 = 0x48;
  const REXWB: u8 = 0x49;
  const LABEL_PLACEHOLDER: u8 = 0x00;
  pub fn new() -> Self {
    let mut masm = MacroAssembler {
      buffer: Vec::new(),
      next_label: 0,
      labels_used: Default::default(),
      labels_defined: Default::default(),
    };
    masm.save_reserved_registers();
    masm
  }
  pub fn len(&self) -> usize {
    self.buffer.len()
  }
  pub fn create_undefined_label(&mut self) -> usize {
    self.next_label += 1;
    self.next_label - 1
  }
  pub fn define_label(&mut self, label: Label) {
    self.labels_defined.insert(label, self.buffer.len());
  }
  fn save_reserved_registers(&mut self) {
    self.emit_pushq_r(12);
    self.emit_pushq_r(13);
    self.emit_pushq_r(14);
    self.emit_pushq_r(15);
  }
  fn load_reserved_registers(&mut self) {
    self.emit_popq_r(15);
    self.emit_popq_r(14);
    self.emit_popq_r(13);
    self.emit_popq_r(12);
  }
  pub fn assemble(&mut self) -> io::Result<JIT_Fn> {
    self.load_reserved_registers();
    self.emit_ret();
    for (&label, &loc) in self.labels_used.iter() {
      match self.labels_defined.get(&label) {
        Some(&def) => {
          let rel_distance = (def as isize) - (loc as isize) - 1;
          match rel_distance {
            -0x80..=0x7f => {
              (&mut self.buffer[..]).write_byte(loc as u32, rel_distance as u32);
            },
            -0x8000..=0x7fff => {
              (&mut self.buffer[..]).write_half(loc as u32, rel_distance as u32);
            },
            _ => {
              (&mut self.buffer[..]).write_word(loc as u32, rel_distance as u32);
            },
          }
        },
        None => panic!("used undefined label {} at {}", label, loc),
      }
    }
    let mut mmap = MmapMut::map_anon(self.buffer.len())?;
    mmap.copy_from_slice(&self.buffer);
    let mmap = mmap.make_exec()?;
    let addr = mmap.as_ptr();
    unsafe {
      let function = std::mem::transmute::<*const u8, fn()>(addr);
      Ok(JIT_Fn::new(mmap, function))
    }
  }
  pub fn emit_ret(&mut self) {
    self.buffer.push(0xc3);
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
  fn emit_conditional_rexwrb(&mut self, reg1: u32, reg2: u32) {
    let r = reg1.nth_bit(3) as u8;
    let b = reg2.nth_bit(3) as u8;
    if (r | b) != 0 {
      self.buffer.push(MacroAssembler::REXW | b | r << 2);
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
  pub fn emit_nop(&mut self) {
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
  //note that this includes rax meaning any return value will be overwritten if
  //this is pushed and popped
  pub fn caller_saved_regs() -> Vec<u32> {
    (0..=15).filter(|&x| x != X64_RSP)
            .filter(|&x| x != X64_RBX)
            .filter(|&x| x != X64_RBP)
            .filter(|&x| x != X64_R12)
            .filter(|&x| x != X64_R13)
            .filter(|&x| x != X64_R14)
            .filter(|&x| x != X64_R15)
            .collect()
  }
}
