use std::io;
use std::collections::HashSet;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnsRegisters;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::macro_assembler::MacroAssembler;
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
    masm.emit_call(Block::load_registers as u64, &console.r3000 as *const R3000 as u64);
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
  fn load_registers(r3000: &R3000) {
    let registers = (0..31).map(|n| r3000.nth_reg(n)).collect::<Vec<u32>>();
    unsafe {
      asm!("movq (%r15), %rax
            movq -8(%r15), %rbx
            movq -16(%r15), %rcx
            movq -24(%r15), %rdx
            movq -32(%r15), %rsi
            movq -40(%r15), %rdi
            movq -48(%r15), %rbp
            movq -56(%r15), %r8
            movq -64(%r15), %r9
            movq -72(%r15), %r10
            movq -80(%r15), %r11
            movq -88(%r15), %r12
            movq -96(%r15), %r13
            movq -104(%r15), %r14
            pushq -112(%r15)
            popq %r15
            "
            ::"{r15}"(&registers[0]));
    }
  }
  pub fn final_pc(&self) -> u32 {
    self.final_pc
  }
  pub fn nominal_len(&self) -> u32 {
    self.nominal_len
  }
}

