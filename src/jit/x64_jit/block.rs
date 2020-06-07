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
  //the physical address of the last instruction
  //this will be either the branch delay slot or a syscall
  final_pc: u32,
  //the number of MIPS opcodes represented by this Block
  //may be more than the number of macros in the function
  nominal_len: u32,
}

impl Block {
  pub fn new(tagged_opcodes: &Vec<Insn>, console: &Console, final_pc: u32,
             nominal_len: u32, logging: bool) -> io::Result<Self> {
    let function = Block::create_function(tagged_opcodes, &console, logging)?;
    Ok(Block {
      function,
      final_pc,
      nominal_len,
    })
  }
  pub fn new_optimized(tagged_opcodes: &Vec<Insn>, console: &Console, final_pc: u32,
                       nominal_len: u32, logging: bool) -> io::Result<Self> {
    let function = Block::create_optimized_function(tagged_opcodes, &console, logging)?;
    Ok(Block {
      function,
      final_pc,
      nominal_len,
    })
  }
  fn create_function(tagged_opcodes: &Vec<Insn>, console: &Console,
                     logging: bool) -> io::Result<JIT_Fn> {
    println!("with {} MIPS registers", tagged_opcodes.registers_by_frequency().len());
    let mut masm = MacroAssembler::new();
    let mut register_map = RegisterMap::new(&tagged_opcodes);
    let cop0_reg_addr = console.cop0.reg_ptr() as u64;
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
    masm.load_registers(&register_map, &console);
    for insn in &tagged_opcodes[0..4] {
      //this is for debugging
      if insn.op() == 0x825 {
        break
      }
      //TODO: make sure all inputs are to this insn are in registers here
      masm.emit_insn(&insn, &mut register_map, logging);
    };
    masm.save_registers(&register_map, &console);
    masm.emit_addq_ir(88, X64_RSP);
    println!("compiled {} bytes", masm.len());
    let jit_fn = masm.compile_buffer()?;
    Ok(jit_fn)
  }
  pub fn final_pc(&self) -> u32 {
    self.final_pc
  }
  pub fn nominal_len(&self) -> u32 {
    self.nominal_len
  }
}

