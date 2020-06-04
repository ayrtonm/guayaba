use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::X64RegNum;

impl MacroAssembler {
  pub fn emit_movl_rr(&mut self, src: u32, dest: u32) {
    self.emit_conditional_rexrb(src, dest);
    self.buffer.push(0x89);
    self.buffer.push(0xc0 | (src.lowest_bits(3) << 3) as u8 | dest.lowest_bits(3) as u8);
  }
  pub fn emit_movl_ir(&mut self, imm32: u32, dest: u32) {
    self.emit_conditional_rexb(dest);
    self.buffer.push(0xb8 | dest.lowest_bits(3) as u8);
    self.emit_imm32(imm32);
  }
  pub fn emit_movq_rr(&mut self, src: u32, dest: u32) {
    let rex_prefix = MacroAssembler::REXW |
                     MacroAssembler::conditional_rexb(dest) |
                     MacroAssembler::conditional_rexr(src);
    self.buffer.push(rex_prefix);
    self.buffer.push(0x89);
    self.buffer.push(0xc0 | (src.lowest_bits(3) << 3) as u8 | dest.lowest_bits(3) as u8);
  }
  pub fn emit_movq_ir(&mut self, imm64: u64, dest: u32) {
    let rex_prefix = MacroAssembler::REXW | MacroAssembler::conditional_rexb(dest);
    self.buffer.push(rex_prefix);
    self.buffer.push(0xb8 | dest.lowest_bits(3) as u8);
    self.emit_imm64(imm64);
  }
  pub fn emit_movl_mr(&mut self, ptr: u32, dest: u32) {
    self.emit_conditional_rexrb(dest, ptr);
    self.buffer.push(0x8b);
    if ptr.lowest_bits(3) == 5 {
      self.buffer.push(0x45 | (dest.lowest_bits(3) << 3) as u8);
      self.buffer.push(0x00);
    } else {
      self.buffer.push((dest.lowest_bits(3) << 3) as u8 | ptr.lowest_bits(3) as u8);
      if ptr.lowest_bits(3) == 4 {
        self.buffer.push(0x24);
      };
    }
  }
  pub fn emit_movl_rm(&mut self, src: u32, ptr: u32) {
    self.emit_conditional_rexrb(src, ptr);
    self.buffer.push(0x89);
    if ptr.lowest_bits(3) == 5 {
      self.buffer.push(0x45 | (src.lowest_bits(3) << 3) as u8);
      self.buffer.push(0x00);
    } else {
      self.buffer.push((src.lowest_bits(3) << 3) as u8 | ptr.lowest_bits(3) as u8);
      if ptr.lowest_bits(3) == 4 {
        self.buffer.push(0x24);
      };
    }
  }
  //FIXME: doesn't seem like this'll work with extended registers i.e. r8-r15
  //fn specify_rm(src: u32, ptr_dest: u32) -> u8 {
  //  (src.lowest_bits(3) << 3) as u8 | (ptr_dest.lowest_bits(3) as u8)
  //}
  //fn specify_mr(src: u32, ptr_dest: u32) -> u8 {
  //  MacroAssembler::specify_rm(src, ptr_dest)
  //}
  //fn specify_sib(base: u32, index: u32, scale: u32) -> u8 {
  //  let scale = match scale {
  //    1 => 0,
  //    2 => 1,
  //    4 => 2,
  //    8 => 3,
  //    _ => unreachable!("invalid SIB scale"),
  //  } << 6;
  //  base.lowest_bits(3) as u8 |  ((index.lowest_bits(3) as u8) << 3) | scale
  //}
  //pub fn emit_movl_rm(&mut self, src: u32, ptr_dest: u32) {
  //  if src.nth_bit_bool(3) {
  //    self.buffer.push(MacroAssembler::REXRB);
  //  } else {
  //    self.buffer.push(MacroAssembler::REXB);
  //  }
  //  let specify_regs = MacroAssembler::specify_rm(src, ptr_dest); 
  //  self.buffer.push(0x89);
  //  self.buffer.push(specify_regs);
  //}
  //pub fn emit_movl_rm_sib_offset(&mut self, src: u32, ptr_dest: u32, idx: u32, scale: u32, offset: u32) {
  //  let specify_regs = MacroAssembler::specify_rm(src, ptr_dest);
  //  let sib = MacroAssembler::specify_sib(ptr_dest, idx, scale);
  //  self.buffer.push(0x89);
  //  self.buffer.push(sib);
  //  self.buffer.push(specify_regs);
  //  self.emit_imm32(offset);
  //}
  //pub fn emit_movl_rm_offset(&mut self, src: u32, ptr_dest: u32, offset: u32) {
  //  let specify_regs = MacroAssembler::specify_rm(src, ptr_dest); 
  //  //let sib = MacroAssembler::specify_sib(ptr_dest, ptr_dest, 1);
  //  self.buffer.push(0x89);
  //  //self.buffer.push(sib);
  //  self.buffer.push(specify_regs);
  //  self.emit_imm32(offset);
  //}
  //pub fn emit_movl_mr(&mut self, ptr_src: u32, dest: u32) {
  //  if ptr_src.nth_bit_bool(3) {
  //    self.buffer.push(MacroAssembler::REXRB);
  //  } else {
  //    self.buffer.push(MacroAssembler::REXB);
  //  }
  //  let specify_regs = MacroAssembler::specify_mr(ptr_src, dest);
  //  self.buffer.push(0x8b);
  //  self.buffer.push(specify_regs);
  //}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn movl_rr() {
    for src in MacroAssembler::test_regs() {
      for dest in MacroAssembler::test_regs() {
        let mut masm = MacroAssembler::new();
        masm.emit_push_imm32(0xbfc0_0101);
        masm.emit_pop_r32(src);
        masm.emit_movl_rr(src, dest);
        masm.emit_push_r32(dest);
        masm.emit_pop_r32(0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(out, 0xbfc0_0101);
      }
    }
  }

  #[test]
  fn movl_ir() {
    for reg in MacroAssembler::test_regs() {
      let mut masm = MacroAssembler::new();
      masm.emit_movl_ir(0xadcb_1324, reg);
      masm.emit_push_r32(reg);
      masm.emit_pop_r32(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u32;
      unsafe {
        asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, 0xadcb_1324);
    }
  }

  #[test]
  fn movq() {
    for reg in MacroAssembler::test_regs() {
      let mut masm = MacroAssembler::new();
      masm.emit_movq_ir(0xadcb_1324_ff00_dcda, reg);
      masm.emit_movq_rr(reg, 0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u64;
      unsafe {
        asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, 0xadcb_1324_ff00_dcda);
    }
  }

  #[test]
  fn movl_mr() {
    //using all_regs() in outer loop to test movl (%rsp), *
    for ptr in MacroAssembler::all_regs() {
      for dest in MacroAssembler::test_regs() {
        let mut masm = MacroAssembler::new();
        masm.emit_push_imm32(0xabcd_1235);
        masm.emit_movq_rr(X64RegNum::RSP as u32, ptr);
        masm.emit_movl_mr(ptr, dest);
        masm.emit_pop_r32(1);
        masm.emit_movl_rr(dest, 0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(out, 0xabcd_1235);
      }
    }
  }

  #[test]
  fn movl_rm() {
    for ptr in MacroAssembler::all_regs() {
      for src in MacroAssembler::test_regs() {
        let memory: u32 = 0xfeaf_fb24;
        let memory_location = &memory as *const u32 as u64;
        let mut masm = MacroAssembler::new();
        masm.emit_movl_ir(0x5324_bcda, src);
        masm.emit_push_r32(1);
        masm.emit_movq_rr(X64RegNum::RSP as u32, ptr);
        masm.emit_movl_rm(src, ptr);
        masm.emit_pop_r32(0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        //checking movl (%r), r is tricky since we need to get the value of %rsp out
        //but we might as well run them anyway to make sure they don't segfault
        if ptr != src {
          assert_eq!(out, 0x5324_bcda);
        }
      }
    }
  }
}
