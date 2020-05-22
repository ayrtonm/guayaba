use std::io;
use std::collections::HashSet;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnsRegisters;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::macro_assembler::MacroAssembler;
use crate::cd::CD;
use crate::console::Console;

pub struct Block {
  //a vec of closures to be executed in order
  function: JIT_Fn,
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
    let function = Block::create_function(tagged_opcodes, console, logging)?;
    Ok(Block {
      function,
      final_pc,
      nominal_len,
    })
  }
  pub fn new_optimized(tagged_opcodes: &Vec<Insn>, console: &Console, final_pc: u32,
                       nominal_len: u32, logging: bool) -> io::Result<Self> {
    let function = Block::create_optimized_function(tagged_opcodes, console, logging)?;
    Ok(Block {
      function,
      final_pc,
      nominal_len,
    })
  }
  fn create_function(tagged_opcodes: &Vec<Insn>, console: &Console, logging: bool) -> io::Result<JIT_Fn> {
    let mut masm = MacroAssembler::new();
    let inputs = tagged_opcodes.unique_inputs();
    let outputs = tagged_opcodes.unique_outputs();
    let registers_used: HashSet<_> = inputs.union(&outputs).filter(|&&r| r != 0).collect();
    //todo!("make a register map for {:?}", registers_used);
    //TODO: create a register map which to be used when emitting macros
    //TODO: populate the register map
    for insn in tagged_opcodes {
      //TODO: pass the rgister map to Stub::new
      masm.emit_insn(&insn, logging);
      masm.emit_call(CD::exec_command as u64, &console.cd as *const CD as u64);
    };
    Ok(masm.compile_buffer()?)
  }
  pub fn execute(&self) {
    self.function.execute()
  }
  pub fn final_pc(&self) -> u32 {
    self.final_pc
  }
  pub fn nominal_len(&self) -> u32 {
    self.nominal_len
  }
}

