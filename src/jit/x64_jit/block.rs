use std::io;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnsRegisters;
use crate::jit::x64_jit::stub::Stub;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::macro_assembler::MacroAssembler;

pub struct Block {
  //a vec of closures to be executed in order
  stubs: JIT_Fn,
  //the physical address of the last instruction
  //this will be either the branch delay slot or a syscall
  final_pc: u32,
  //the number of MIPS opcodes represented by this Block
  //may be more than the length of stubs
  nominal_len: u32,
}

impl Block {
  pub fn new(tagged_opcodes: &Vec<Insn>, final_pc: u32,
             nominal_len: u32, logging: bool) -> io::Result<Self> {
    let stubs = Block::create_stubs(tagged_opcodes, logging)?;
    Ok(Block {
      stubs,
      final_pc,
      nominal_len,
    })
  }
  pub fn new_optimized(tagged_opcodes: &Vec<Insn>, final_pc: u32,
                       nominal_len: u32, logging: bool) -> io::Result<Self> {
    let stubs = Block::create_optimized_stubs(tagged_opcodes, logging)?;
    Ok(Block {
      stubs,
      final_pc,
      nominal_len,
    })
  }
  fn create_stubs(tagged_opcodes: &Vec<Insn>, logging: bool) -> io::Result<JIT_Fn> {
    let mut masm = MacroAssembler::new();
    let inputs = tagged_opcodes.unique_inputs();
    let outputs = tagged_opcodes.unique_outputs();
    let registers_used = inputs.union(&outputs);
    //for insn in tagged_opcodes {
    //  ret.push(Stub::new(&insn, logging));
    //};
    Ok(masm.compile_buffer()?)
  }
  pub fn execute(&self) {
    self.stubs.execute()
  }
  pub fn final_pc(&self) -> u32 {
    self.final_pc
  }
  pub fn nominal_len(&self) -> u32 {
    self.nominal_len
  }
}

