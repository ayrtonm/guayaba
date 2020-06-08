use rand::thread_rng;
use rand::seq::SliceRandom;
use crate::console::Console;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisterFrequency;
use crate::jit::insn::MIPSRegister;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::register::BitTwiddle;

pub type X64GPRNum = u32;
pub const X64_RAX: X64GPRNum = 0;
pub const X64_RCX: X64GPRNum = 1;
pub const X64_RDX: X64GPRNum = 2;
pub const X64_RBX: X64GPRNum = 3;
pub const X64_RSP: X64GPRNum = 4;
pub const X64_RBP: X64GPRNum = 5;
pub const X64_RSI: X64GPRNum = 6;
pub const X64_RDI: X64GPRNum = 7;
pub const X64_R8: X64GPRNum = 8;
pub const X64_R9: X64GPRNum = 9;
pub const X64_R10: X64GPRNum = 10;
pub const X64_R11: X64GPRNum = 11;
pub const X64_R12: X64GPRNum = 12;
pub const X64_R13: X64GPRNum = 13;
pub const X64_R14: X64GPRNum = 14;
pub const X64_R15: X64GPRNum = 15;

//MIPS registers are either bound to an x64 register or spilled onto the stack
#[derive(Debug,Copy,Clone)]
enum X64Register {
  GPR(X64GPRNum),
  Stack(i32),
}

#[derive(Debug)]
pub struct Mapping {
  x64_reg: X64Register,
  mips_reg: Option<MIPSRegister>,
}

impl Mapping {
  fn new(tuple: (X64Register, Option<MIPSRegister>)) -> Mapping {
    Mapping {
      x64_reg: tuple.0,
      mips_reg: tuple.1,
    }
  }
  fn on_stack(&self) -> bool {
    match self.x64_reg {
      X64Register::Stack(_) => true,
      _ => false,
    }
  }
  fn stack_offset(&self) -> i32 {
    match self.x64_reg {
      X64Register::Stack(offset) => offset,
      _ => unreachable!("This shouldn't be called on a GPR mapping"),
    }
  }
  fn spill(&mut self, offset: i32) {
    self.x64_reg = X64Register::Stack(offset);
  }
  fn is_bound(&self) -> bool {
    match self.x64_reg {
      X64Register::GPR(_) => true,
      _ => false,
    }
  }
  pub fn bound_gpr(&self) -> X64GPRNum {
    match self.x64_reg {
      X64Register::GPR(x) => x,
      _ => unreachable!("This shouldn't be called on a stack mapping"),
    }
  }
  fn bind(&mut self, x64_reg: X64GPRNum) {
    self.x64_reg = X64Register::GPR(x64_reg);
  }
  fn x64_reg(&self) -> X64Register {
    self.x64_reg
  }
  fn in_use(&self) -> bool {
    self.mips_reg.is_some()
  }
  fn mips_reg(&self) -> MIPSRegister {
    self.mips_reg.expect("This shouldn't be called on unused mappings")
  }
}

#[derive(Debug)]
pub struct RegisterMap {
  mappings: Vec<Mapping>,
}

impl RegisterMap {
  pub fn new(tagged_opcodes: &Vec<Insn>) -> Self {
    let stack_offsets = (0..=15).map(|offset| X64Register::Stack(offset * 4));
    let x64_registers: Vec<_> = MacroAssembler::free_regs().iter()
                                                           .map(|&x| X64Register::GPR(x))
                                                           .chain(stack_offsets)
                                                           .collect();
    //let reg_by_freq = tagged_opcodes.registers_by_frequency();
    //for debugging
    let mut reg_by_freq: Vec<_> = (1..=31).collect();
    reg_by_freq.shuffle(&mut thread_rng());
    let num_regs_on_stack = x64_registers.len() - reg_by_freq.len();
    let nones = [None].iter().map(|&m| m).cycle().take(num_regs_on_stack);
    let mips_registers: Vec<_> = reg_by_freq.iter()
                                            .map(|&m| Some(m))
                                            .chain(nones)
                                            .collect();
    let mappings = x64_registers.iter()
                                .zip(mips_registers)
                                .map(|(&x,m)| Mapping::new((x,m)))
                                .collect::<Vec<Mapping>>();
    RegisterMap { mappings }
  }
  pub fn count_spilled(&self) -> i32 {
    4 * self.spilled_mappings().iter().count() as i32
  }
  fn spilled_mappings(&self) -> Vec<&Mapping> {
    self.mappings.iter()
                 .filter(|&map| map.in_use())
                 .filter(|&map| map.on_stack())
                 .collect()
  }
  fn bound_mappings(&self) -> Vec<&Mapping> {
    self.mappings.iter()
                 .filter(|&map| map.in_use())
                 .filter(|&map| map.is_bound())
                 .collect()
  }
  pub fn mips_is_bound(&self, mips_reg: u32) -> bool {
    self.bound_mappings()
        .iter()
        .any(|&map| map.mips_reg() == mips_reg as i32)
  }
  pub fn gpr_is_bound(&self, x64_reg: X64GPRNum) -> bool {
    self.bound_mappings()
        .iter()
        .any(|&map| map.bound_gpr() == x64_reg)
  }
  pub fn mips_to_x64(&self, mips_reg: u32) -> Option<&Mapping> {
    self.mappings.iter()
                 .find(|&map| map.mips_reg() == mips_reg as i32)
  }
  fn gpr_to_mips(&self, x64_reg: X64GPRNum) -> Option<u32> {
    self.bound_mappings()
        .iter()
        .find(|&m| m.bound_gpr() == x64_reg)
        .map(|&m| m.mips_reg() as u32)
  }
  fn offset_to_mips(&self, offset: i32) -> Option<u32> {
    self.spilled_mappings()
        .iter()
        .find(|&m| m.stack_offset() == offset)
        .map(|&m| m.mips_reg() as u32)
  }
  fn bind_mips_to_gpr(&mut self, mips_reg: u32, x64_reg: X64GPRNum) {
    for map in &mut self.mappings {
      if map.in_use() {
        if map.mips_reg() == mips_reg as i32 {
          map.bind(x64_reg);
        }
      }
    }
  }
  fn spill_mips_to_offset(&mut self, mips_reg: u32, offset: i32) {
    for map in &mut self.mappings {
      if map.in_use() {
        if map.mips_reg() == mips_reg as i32 {
          map.spill(offset);
        }
      }
    }
  }
}

impl MacroAssembler {
  pub const MIPS_REG_POSITION: i32     = 0;
  pub const COP0_POSITION: i32         = 8;
  pub const CONSOLE_POSITION: i32      = 16;
  pub const WRITE_WORD_POSITION: i32   = 24;
  pub const WRITE_HALF_POSITION: i32   = 32;
  pub const WRITE_BYTE_POSITION: i32   = 40;
  pub const READ_WORD_POSITION: i32    = 48;
  pub const READ_HALF_POSITION: i32    = 56;
  pub const READ_BYTE_POSITION: i32    = 64;
  pub const READ_HALF_SE_POSITION: i32 = 72;
  pub const READ_BYTE_SE_POSITION: i32 = 80;
  //emit an instruction to load a MIPS register into the specified x64 register
  //also loads the value in the specified x64 register into the x64 register which contained the MIPS register
  //then updates the register map accordingly so we avoid having to swap them back
  pub fn emit_swap_mips_registers(&mut self, register_map: &mut RegisterMap, mips_reg: u32, x64_reg: u32) {
    let other_x64_reg = register_map.mips_to_x64(mips_reg).expect("");
    if register_map.gpr_is_bound(x64_reg) {
      match other_x64_reg.x64_reg() {
        X64Register::GPR(other_x64_reg) => {
          if x64_reg != other_x64_reg {
            let other_mips_reg = register_map.gpr_to_mips(x64_reg).expect("");
            register_map.bind_mips_to_gpr(mips_reg, x64_reg);
            register_map.bind_mips_to_gpr(other_mips_reg, other_x64_reg);
            self.emit_xchgq_rr(other_x64_reg, x64_reg);
          }
        },
        X64Register::Stack(offset) => {
          let other_mips_reg = register_map.gpr_to_mips(x64_reg).expect("");
          register_map.bind_mips_to_gpr(mips_reg, x64_reg);
          register_map.spill_mips_to_offset(other_mips_reg, offset);
          self.emit_xchgl_rm_offset(x64_reg, X64_RSP, offset);
        },
      }
    } else {
      match other_x64_reg.x64_reg() {
        X64Register::GPR(other_x64_reg) => {
          register_map.bind_mips_to_gpr(mips_reg, x64_reg);
          self.emit_movl_rr(other_x64_reg, x64_reg);
        },
        X64Register::Stack(offset) => {
          register_map.bind_mips_to_gpr(mips_reg, x64_reg);
          self.emit_movl_mr_offset(X64_RSP, x64_reg, offset);
        },
      }
    }
  }
  pub fn load_registers(&mut self, register_map: &RegisterMap) {
    self.emit_movq_mr(X64_RSP, X64_R15);
    self.emit_addq_ir(-register_map.count_spilled(), X64_RSP);
    for map in register_map.spilled_mappings() {
      let offset = map.stack_offset();
      let mips_idx = 4 * (map.mips_reg() - 1);
      self.emit_movl_mr_offset(X64_R15, X64_R14, mips_idx);
      self.emit_movl_rm_offset(X64_R14, X64_RSP, offset);
    }
    for map in register_map.bound_mappings() {
      let x64_reg = map.bound_gpr();
      if x64_reg != X64_R15 {
        let mips_idx = 4 * (map.mips_reg() - 1);
        self.emit_movl_mr_offset(X64_R15, x64_reg, mips_idx);
      }
    }
    match register_map.bound_mappings().iter().find(|&m| m.bound_gpr() == X64_R15) {
      Some(map) => {
        let mips_idx = 4 * (map.mips_reg() - 1);
        self.emit_movl_mr_offset(X64_R15, X64_R15, mips_idx);
      },
      _ => (),
    }
  }
  pub fn save_registers(&mut self, register_map: &RegisterMap) {
    let frame_pointer = register_map.count_spilled();
    let mut stack_pointer = frame_pointer;
    let r15_in_use = register_map.bound_mappings()
                                 .iter()
                                 .any(|&m| m.bound_gpr() == X64_R15);
    if r15_in_use {
      self.emit_push_r64(X64_R15);
      stack_pointer += 8;
    }
    let mips_reg_ptr = stack_pointer + MacroAssembler::MIPS_REG_POSITION;
    self.emit_movq_mr_offset(X64_RSP, X64_R15, mips_reg_ptr);
    for map in register_map.bound_mappings() {
      let x64_reg = map.bound_gpr();
      if x64_reg != X64_R15 {
        let mips_idx = 4 * (map.mips_reg() - 1);
        self.emit_movl_rm_offset(x64_reg, X64_R15, mips_idx);
      }
    }
    for map in register_map.spilled_mappings() {
      let offset = map.stack_offset() + (stack_pointer - frame_pointer);
      let mips_idx = 4 * (map.mips_reg() - 1);
      self.emit_movl_mr_offset(X64_RSP, X64_R14, offset);
      self.emit_movl_rm_offset(X64_R14, X64_R15, mips_idx);
    }
    match register_map.bound_mappings().iter().find(|&m| m.bound_gpr() == X64_R15) {
      Some(map) => {
        let mips_idx = 4 * (map.mips_reg() - 1);
        self.emit_pop_r64(X64_R14);
        self.emit_movl_rm_offset(X64_R14, X64_R15, mips_idx);
        stack_pointer -= 8;
      },
      _ => (),
    }
    assert_eq!(stack_pointer, frame_pointer);
    self.emit_addq_ir(register_map.count_spilled(), X64_RSP);
  }
}
