use std::io;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisterFrequency;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::RegisterMap;
use crate::jit::x64_jit::register_allocator::*;
use crate::cd::CD;
use crate::r3000::R3000;
use crate::console::Console;

pub struct Block {
  //a vec of closures to be executed in order
  pub function: JIT_Fn,
  initial_pc: u32,
  //the physical address of the last instruction
  //this will be either the branch delay slot or a syscall
  final_phys_pc: u32,
  //the number of MIPS opcodes represented by this Block
  //may be more than the number of macros in the function
  nominal_len: u32,
}

impl Block {
  pub fn new(tagged_opcodes: &Vec<Insn>, console: &Console,
             initial_pc: u32, final_phys_pc: u32,
             nominal_len: u32, logging: bool) -> io::Result<Self> {
    let function = Block::create_function(tagged_opcodes, &console, initial_pc, logging)?;
    Ok(Block {
      function,
      initial_pc,
      final_phys_pc,
      nominal_len,
    })
  }
  pub fn new_optimized(tagged_opcodes: &Vec<Insn>, console: &Console,
                       initial_pc: u32, final_phys_pc: u32,
                       nominal_len: u32, logging: bool) -> io::Result<Self> {
    let function = Block::create_optimized_function(tagged_opcodes, &console, initial_pc, logging)?;
    Ok(Block {
      function,
      initial_pc,
      final_phys_pc,
      nominal_len,
    })
  }
  fn create_function(tagged_opcodes: &Vec<Insn>, console: &Console,
                     initial_pc: u32, logging: bool) -> io::Result<JIT_Fn> {
    let mut masm = MacroAssembler::new();
    let mut register_map = RegisterMap::new(&tagged_opcodes);
    let cop0_reg_addr = console.cop0.reg_ptr() as u64;
    let end = masm.create_undefined_label();
    masm.emit_push_imm64(Console::read_byte_sign_extended as u64);
    masm.emit_push_imm64(Console::read_half_sign_extended as u64);
    masm.emit_push_imm64(Console::read_byte as u64);
    masm.emit_push_imm64(Console::read_half as u64);
    masm.emit_push_imm64(Console::read_word as u64);
    masm.emit_push_imm64(Console::write_byte as u64);
    masm.emit_push_imm64(Console::write_half as u64);
    masm.emit_push_imm64(Console::write_word as u64);
    masm.emit_push_imm64(console as *const Console as u64);
    masm.emit_push_imm64(cop0_reg_addr);
    masm.emit_push_imm64(console.r3000.reg_ptr() as u64);
    masm.load_registers(&register_map);
    let mut jmp_label = None;
    for insn in tagged_opcodes {
      let prev_label = jmp_label;
      //FIXME: there are some cases where this method won't be able to bind all MIPS register
      //this will cause segfaults
      let init_x64 = X64_R13;
      let mut i = 0;
      let mut bound_gprs = vec![];
      for dep in insn.dependencies() {
        if dep != 0 {
          if !register_map.mips_is_bound(dep) {
            while bound_gprs.iter().any(|&x| x == init_x64 - i) {
              i += 1;
              if i == 9 {
                i += 1;
              }
            }
            masm.emit_swap_mips_registers(dep, init_x64 - i, &mut register_map, 0);
            bound_gprs.push(init_x64 - i);
            i += 1;
            if i == 9 {
              i += 1;
            }
          }
        }
      }
      match prev_label {
        Some(some_jump) => {
          jmp_label = masm.emit_insn(&insn, &mut register_map, initial_pc, end, 4, logging);
          masm.emit_call_label(some_jump);
        },
        None => {
          jmp_label = masm.emit_insn(&insn, &mut register_map, initial_pc, end, 0, logging);
        },
      }
      if insn.op() == 0x214a_0080 {
        break
      }
    };
    masm.define_label(end);
    masm.save_registers(&register_map);
    masm.emit_addq_ir(88, X64_RSP);
    println!("compiled {} bytes", masm.len());
    let jit_fn = masm.compile_buffer()?;
    Ok(jit_fn)
  }
  pub fn final_phys_pc(&self) -> u32 {
    self.final_phys_pc
  }
  pub fn nominal_len(&self) -> u32 {
    self.nominal_len
  }
}

