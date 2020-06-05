use std::io;
use crate::jit::insn::Insn;
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
    let mut masm = MacroAssembler::new();
    let mut register_map = RegisterMap::new(&tagged_opcodes);
    let cop0_reg_addr = console.cop0.reg_ptr() as u64;
    //TODO: these should be conditionally pushed and popped onto/from the stack
    //based on the tagged opcodes. I might need to tag them more thoroughly make
    //good use of the stack space. This also means noting the position of each
    //thing on the stack and passing the positions to emit_insn()
    masm.emit_movq_ir(console as *const Console as u64, 0);
    masm.emit_push_r64(0);
    masm.emit_movq_ir(Console::write_word as u64, 0);
    masm.emit_push_r64(0);
    masm.emit_movq_ir(cop0_reg_addr, 0);
    masm.emit_push_r64(0);
    masm.load_registers(&register_map, &console);
    for insn in &tagged_opcodes[0..4] {
      //TODO: make sure all inputs are to this insn are in registers here
      masm.emit_insn(&insn, &register_map, logging);
    };
    masm.save_registers(&register_map, &console);
    masm.emit_addq_ir(24, X64_RSP);
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

