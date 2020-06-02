use std::io;
use memmap::MmapMut;
use crate::register::BitTwiddle;
use crate::console::Console;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::x64_jit::register_allocator::RegisterMap;
use crate::jit::x64_jit::register_allocator::X64RegNum;

pub struct MacroAssembler {
  buffer: Vec<u8>,
}

impl MacroAssembler {
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
    println!("compiled a {} byte function", self.buffer.len());
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
      let mips_reg_idx = 4 * mapping.mips_reg() as u64;
      let mips_reg_addr = console.r3000.reg_ptr() as u64 + mips_reg_idx;
      let x64_reg = mapping.x64_reg().num();
      //movq mips_reg_addr, %r14
      self.emit_movq_ir(mips_reg_addr, 0);
      //movl (%r14), %exx
      self.emit_movl_mr(x64_reg, 0);
    }
    println!("added {} bytes to the function in load_registers", self.buffer.len() - init_size);
  }
  pub fn save_registers(&mut self, register_map: &RegisterMap, console: &Console) {
    let init_size = self.buffer.len();
    for mapping in register_map.mappings() {
      let mips_reg_idx = 4 * mapping.mips_reg() as u64;
      let mips_reg_addr = console.r3000.reg_ptr() as u64 + mips_reg_idx;
      let x64_reg = mapping.x64_reg().num();
      //movq mips_reg_addr, %r14
      self.emit_movq_ir(mips_reg_addr, 0);
      //movl %exx, (%r14)
      self.emit_movl_rm(x64_reg, 0);
    }
    println!("added {} bytes to the function in save_registers", self.buffer.len() - init_size);
  }
  pub fn emit_xorw_ir(&mut self, imm16: u16, reg: u32) {
    self.buffer.push(0x66);
    if reg == X64RegNum::RAX as u32 {
      self.buffer.push(0x35);
    } else {
      if reg.nth_bit_bool(3) {
        self.buffer.push(0x41);
      };
      self.buffer.push(0x81);
      let specify_reg = if reg.nth_bit_bool(3) {
        0xc8 + (reg as u8 - 8)
      } else {
        0xc8 + (reg as u8)
      };
      self.buffer.push(specify_reg);
    }
    self.emit_imm16(imm16);
  }
  pub fn emit_orw_ir(&mut self, imm16: u16, reg: u32) {
    self.buffer.push(0x66);
    if reg == X64RegNum::RAX as u32 {
      self.buffer.push(0x0d);
    } else {
      if reg.nth_bit_bool(3) {
        self.buffer.push(0x41);
      };
      self.buffer.push(0x81);
      let specify_reg = if reg.nth_bit_bool(3) {
        0xc8 + (reg as u8 - 8)
      } else {
        0xc8 + (reg as u8)
      };
      self.buffer.push(specify_reg);
    }
    self.emit_imm16(imm16);
  }
  pub fn emit_movl_rr(&mut self, src: u32, dest: u32) {
    self.buffer.push(0x89);
    let specify_reg = 0xc0 + (dest as u8) + (src as u8 * 8);
    self.buffer.push(specify_reg);
  }
  pub fn emit_movl_ir(&mut self, imm32: u32, reg: u32) {
    let specify_reg = if reg.nth_bit_bool(3) {
      self.buffer.push(0x41);
      0xb8 + (reg as u8 - 8)
    } else {
      0xb8 + (reg as u8)
    };
    self.buffer.push(specify_reg);
    self.emit_imm32(imm32);
  }
  fn emit_movq_ir(&mut self, imm64: u64, reg: u32) {
    //reg is hardcoded to %r14 for now
    self.buffer.push(0x49);
    self.buffer.push(0xbe);
    self.emit_imm64(imm64);
  }
  fn emit_movl_rm(&mut self, reg: u32, idx: u32) {
    //idx register is hardcoded to %r14 for now
    let specify_reg = if reg.nth_bit_bool(3) {
      self.buffer.push(0x45);
      0x07 + ((reg - 8) as u8 * 8)
    } else {
      self.buffer.push(0x41);
      0x07 + (reg as u8 * 8)
    };
    self.buffer.push(0x89);
    self.buffer.push(specify_reg - 1);
  }
  //emit movl (%reg), %idx
  fn emit_movl_mr(&mut self, reg: u32, idx: u32) {
    //idx register is hardcoded to %r14 for now
    let specify_reg = if reg.nth_bit_bool(3) {
      self.buffer.push(0x45);
      0x07 + ((reg - 8) as u8 * 8)
    } else {
      self.buffer.push(0x41);
      0x07 + (reg as u8 * 8)
    };
    self.buffer.push(0x8b);
    self.buffer.push(specify_reg - 1);
  }
  //emit return
  fn emit_ret(&mut self) {
    self.buffer.push(0xc3);
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
