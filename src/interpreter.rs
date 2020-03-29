use std::io;
use std::ops::Shl;
use std::ops::Shr;
use crate::common::*;
use crate::register::Register;
use crate::register::Parts;
use crate::r3000::R3000;
use crate::r3000::DelayedWrite;
use crate::r3000::Name;
use crate::cop0::Cop0;
use crate::memory::Memory;
use crate::cd::CD;
use crate::gte::GTE;

pub struct Interpreter {
  r3000: R3000,
  cop0: Cop0,
  memory: Memory,
  gte: GTE,
  cd: Option<CD>,
  next_pc: Option<Register>,
  //these are register writes due to memory loads which happen after one cycle
  delayed_writes: Vec<DelayedWrite>,
}

impl Interpreter {
  pub fn new(bios_filename: &String, infile: Option<&String>) -> io::Result<Self> {
    let r3000 = R3000::new();
    let cop0 = Default::default();
    let memory = Memory::new(bios_filename)?;
    let gte = Default::default();
    let cd = infile.and_then(|f| CD::new(f).ok());
    let delayed_writes = Vec::new();
    Ok(Interpreter {
      r3000,
      cop0,
      memory,
      gte,
      cd,
      next_pc: None,
      delayed_writes,
    })
  }
  pub fn run(&mut self, n: Option<u32>) {
    n.map(
      |n| {
        println!("started in test mode");
        for i in 1..=n {
          println!("{} ----------------------", i);
          self.step();
        }
    }).or_else(
      || {
        println!("started in free-running mode");
        let mut i = 1;
        loop {
          println!("{} ----------------------", i);
          self.step();
          i += 1;
        }
      });
    self.cd.as_ref().map(|cd| cd.preview(10));
  }
  fn step(&mut self) {
    //get opcode from memory at program counter
    let op = self.memory.read_word(self.r3000.pc());
    println!("read opcode {:#x} from [{:#x}]", op, self.r3000.pc());
    print!("  ");
    //the instruction following each jump is always executed before updating the pc
    //increment the program counter
    *self.r3000.pc_mut() = self.next_pc
                           .take()
                           .map_or_else(|| self.r3000.pc() + 4, |next_pc| next_pc);
    self.next_pc = self.execute_opcode(op);
  }
  //if program counter should incremented normally, return None
  //otherwise return Some(new program counter)
  fn execute_opcode(&mut self, op: u32) -> Option<Register> {
    let logging = true;
    macro_rules! log {
      ($e:tt) => {
        {
          print!("reached unimplemented opcode: ");
          println!($e);
          if logging {
            unreachable!("reached unimplemented opcode {:#x?}", self.r3000);
          } else {
            None
          }
        }
      };
    }
    //loading a value from memory is a delayed operation (i.e. the updated register
    //is not visible to the next opcode). Note that the rs + imm16 in parentheses is
    //symbolic and only used to improve readability. This macro should be able to
    //handle all loads in the MIPS instructions set so there's no point to generalizing it
    macro_rules! mov {
      //delayed aligned reads
      (rt = [rs + imm16] $method:ident) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          //FIXME: should this be sign extended?
          let imm16 = get_imm16(op);
          let rt = get_rt(op);
          let result = self.memory.$method(rs + imm16);
          self.delayed_writes.push(DelayedWrite::new(Name::Rn(rt), result, 1));
          println!("R{} = [{:#x} + {:#x}] \n  = [{:#x}] \n  = {:#x}", rt, rs, imm16, rs + imm16, result);
          None
        }
      };
      //aligned writes
      ([rs + imm16] = rt $method:ident) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let rt = self.r3000.nth_reg(get_rt(op));
          let imm16 = get_imm16(op).half_sign_extended();
          self.memory.$method(rs + imm16, rt);
          println!("[{:#x} + {:#x}] = [{:#x}] \n  = R{} \n  = {:#x}", rs, imm16, rs + imm16, get_rt(op), rt);
          None
        }
      };
      (lo = rs) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let lo = self.r3000.lo_mut();
          *lo = rs;
          println!("op1");
          None
        }
      };
      (hi = rs) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let hi = self.r3000.hi_mut();
          *hi = rs;
          println!("op2");
          None
        }
      };
      (rd = lo) => {
        {
          let lo = self.r3000.lo();
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = lo);
          println!("op3");
          None
        }
      };
      (rd = hi) => {
        {
          let hi = self.r3000.hi();
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = hi);
          println!("op4");
          None
        }
      };
    }
    //since self.r3000 is borrowed mutably on the lhs, the rhs must be
    //computed from the immutable references before assigning its value
    //to the lhs
    macro_rules! compute {
      //ALU instructions with two general purpose registers
      (rd = rs $method:ident rt) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let rt = self.r3000.nth_reg(get_rt(op));
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = rs.$method(rt));
          println!("R{} = R{} {} {:#x} \n  = {:#x} {} {:#x} \n  = {:#x}",
                    get_rd(op), get_rs(op), stringify!($method), get_rt(op),
                    rs, stringify!($method), rt, self.r3000.nth_reg(get_rd(op)));
          None
        }
      };
      //ALU instructions with two general purpose registers that trap overflow
      //FIXME: this doesn't actually do anything in case of an overflow yet
      (rd = rs $method:ident rt trap) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let rt = self.r3000.nth_reg(get_rt(op));
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = rs.$method(rt));
          println!("op6");
          None
        }
      };
      //ALU instructions with a register and immediate 16-bit data that trap overflow
      (rt = rs $method:ident imm16 trap) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let imm16 = get_imm16(op).half_sign_extended();
          let rt = self.r3000.nth_reg_mut(get_rt(op));
          rt.map(|rt| *rt = rs.$method(imm16));
          println!("op7");
          None
        }
      };
      //ALU instructions with a register and immediate 16-bit data
      (rt = rs $method:tt imm16) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let imm16 = get_imm16(op).half_sign_extended();
          let rt = self.r3000.nth_reg_mut(get_rt(op));
          rt.map(|rt| *rt = rs.$method(imm16));
          println!("R{} = R{} {} {:#x} \n  = {:#x} {} {:#x} \n  = {:#x}",
                    get_rt(op), get_rs(op), stringify!($method), imm16,
                    rs, stringify!($method), imm16, self.r3000.nth_reg(get_rt(op)));
          None
        }
      };
      //shifts a register based on immediate 5 bits
      (rd = rt $method:tt imm5) => {
        {
          let rt = self.r3000.nth_reg(get_rt(op));
          let imm5 = get_imm5(op);
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = rt.$method(imm5));
          println!("R{} = R{} {} {:#x} \n  = {:#x} {} {:#x} \n  = {:#x}",
                    get_rd(op), get_rt(op), stringify!($method), imm5,
                    rt, stringify!($method), imm5, self.r3000.nth_reg(get_rd(op)));
          None
        }
      };
      //shifts a register based on the lowest 5 bits of another register
      (rd = rt $method:tt (rs and 0x1F)) => {
        {
          let rt = self.r3000.nth_reg(get_rt(op));
          let rs = self.r3000.nth_reg(get_rs(op));
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = rt.$method(rs & 0x1F));
          println!("op9");
          None
        }
      };
      (rt = imm16 shl 16) => {
        {
          let rt = self.r3000.nth_reg_mut(get_rt(op));
          let imm16 = get_imm16(op);
          rt.map(|rt| *rt = imm16 << 16);
          println!("R{} = {:#x} << 16 \n  = {:#x}", get_rt(op), imm16, self.r3000.nth_reg(get_rt(op)));
          None
        }
      };
      (hi:lo = rs * rt) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let rt = self.r3000.nth_reg(get_rt(op));
          let result = (rs as u64) * (rt as u64);
          let hi_res = (result >> 32) as u32;
          let lo_res = (result & 0x0000_0000_ffff_ffff) as u32;
          let delay = match rs {
            0x0000_0000..=0x0000_07ff => {
              6
            },
            0x0000_0800..=0x000f_ffff => {
              9
            },
            0x0010_0000..=0xffff_ffff => {
              13
            },
          };
          self.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, delay));
          self.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, delay));
          println!("op11");
          None
        }
      };
      (hi:lo = rs * rt signed) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op)) as i32;
          let rt = self.r3000.nth_reg(get_rt(op)) as i32;
          let result = (rs as i64) * (rt as i64);
          let hi_res = (result >> 32) as u32;
          let lo_res = (result & 0x0000_0000_ffff_ffff) as u32;
          let delay = match rs as u32 {
            0x0000_0000..=0x0000_07ff | 0xffff_f800..=0xffff_ffff => {
              6
            },
            0x0000_0800..=0x000f_ffff | 0xfff0_0000..=0xffff_f801 => {
              9
            },
            0x0010_0000..=0x7fff_ffff | 0x8000_0000..=0xfff0_0001 => {
              13
            },
          };
          self.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, delay));
          self.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, delay));
          println!("op11");
          None
        }
      };
      (hi:lo = rs / rt) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let rt = self.r3000.nth_reg(get_rt(op));
          let lo_res = match rt {
            0 => {
              0xffff_ffff
            },
            _ => {
              rs / rt
            },
          };
          let hi_res = rs % rt;
          self.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, 36));
          self.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, 36));
          println!("op12");
          None
        }
      };
      (hi:lo = rs / rt signed) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op)) as i32;
          let rt = self.r3000.nth_reg(get_rt(op)) as i32;
          let lo_res = match rt {
            0 => {
              match rs {
                0x0000_0000..=0x7fff_ffff => {
                  -1
                },
                -0x8000_0000..=-1 => {
                  1
                },
              }
            },
            -1 => {
              match rs {
                -0x8000_0000..=-1 => {
                  1
                },
                _ => {
                  rs / rt
                },
              }
            }
            _ => {
              rs / rt
            },
          } as u32;
          let hi_res = (rs % rt) as u32;
          self.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, 36));
          self.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, 36));
          println!("op12");
          None
        }
      };
    }
    macro_rules! jump {
      (imm26) => {
        {
          let imm = get_imm26(op);
          let dest = (self.r3000.pc() & 0xf000_0000) + (imm * 4);
          println!("PC = {:#x}", dest);
          Some(dest)
        }
      };
      (rs) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          println!("op14");
          Some(rs)
        }
      };
      (rs $cmp:tt rt) => {
        {
          let rt = self.r3000.nth_reg(get_rt(op));
          let rs = self.r3000.nth_reg(get_rs(op));
          println!("op15");
          if rs $cmp rt {
            let imm16 = get_imm16(op);
            let pc = self.r3000.pc();
            let dest = pc + (imm16 * 4);
            Some(dest)
          } else {
            None
          }
        }
      };
      (rs $cmp:tt 0) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          println!("op16");
          if (rs as i32) $cmp 0 {
            let imm16 = get_imm16(op);
            let pc = self.r3000.pc();
            let dest = pc + (imm16 * 4);
            Some(dest)
          } else {
            None
          }
        }
      };
    }
    macro_rules! call {
      (imm26) => {
        {
          *self.r3000.ra_mut() = self.r3000.pc() + 4;
          println!("op17");
          jump!(imm26)
        }
      };
      (rs) => {
        {
          let result = self.r3000.pc() + 4;
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = result);
          println!("op18");
          jump!(rs)
        }
      };
      (rs $cmp:tt rt) => {
        {
          let rt = self.r3000.nth_reg(get_rt(op));
          let rs = self.r3000.nth_reg(get_rs(op));
          println!("op19");
          if *rs $cmp *rt {
            *self.r3000.ra_mut() = self.r3000.pc() + 4;
            let imm16 = get_imm16(op);
            let pc = self.r3000.pc();
            let dest = pc + (imm16 * 4);
            Some(dest)
          } else {
            None
          }
        }
      };
      (rs $cmp:tt 0) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          println!("op20");
          if (rs as i32) $cmp 0 {
            *self.r3000.ra_mut() = self.r3000.pc() + 4;
            let imm16 = get_imm16(op);
            let pc = self.r3000.pc();
            let dest = pc + (imm16 * 4);
            Some(dest)
          } else {
            None
          }
        }
      };
    }
    macro_rules! cop {
      ($copn:ident) => {
        {
          match get_rs(op) {
            0x00 => {
              //MFCn
              let rt = get_rt(op);
              let rd_data = self.$copn.nth_data_reg(get_rd(op));
              self.delayed_writes.push(DelayedWrite::new(Name::Rn(rt), rd_data, 1));
              None
            },
            0x02 => {
              //CFCn
              let rt = get_rt(op);
              let rd_ctrl = self.$copn.nth_ctrl_reg(get_rd(op));
              self.delayed_writes.push(DelayedWrite::new(Name::Rn(rt), rd_ctrl, 1));
              None
            },
            0x04 => {
              //MTCn
              let rt = self.r3000.nth_reg(get_rt(op));
              let rd = self.$copn.nth_data_reg_mut(get_rd(op));
              rd.map(|rd| *rd = rt);
              None
            },
            0x06 => {
              //CTCn
              let rt = self.r3000.nth_reg(get_rt(op));
              let rd = self.$copn.nth_ctrl_reg_mut(get_rd(op));
              rd.map(|rd| *rd = rt);
              None
            },
            0x08 => {
              match get_rt(op) {
                0x00 => {
                  //BCnF
                  self.$copn.bcnf(get_imm16(op))
                },
                0x01 => {
                  //BCnT
                  //technically we're implementing one illegal instruction here
                  //since BCnT is not implemented for COP0
                  //however, GTE (i.e. COP2) does implement it
                  None
                },
                _ => {
                  unreachable!("ran into invalid opcode")
                },
              }
            },
            0x10..=0x1F => {
              //COPn imm25
              self.$copn.execute_command(get_imm25(op))
            },
            _ => {
              unreachable!("ran into invalid opcode")
            },
          }
        }
      }
    }
    //after executing an opcode, complete the loads from the previous opcode
    self.r3000.flush_write_cache(&mut self.delayed_writes);
    //this match statement optionally returns the next program counter
    //if the return value is None, then we increment pc as normal
    match get_primary_field(op) {
      0x00 => {
        //SPECIAL
        match get_secondary_field(op) {
          0x00 => {
            //SLL
            compute!(rd = rt shl imm5)
          },
          0x02 => {
            //SRL
            compute!(rd = rt shr imm5)
          },
          0x03 => {
            //SRA
            compute!(rd = rt sra imm5)
          },
          0x04 => {
            //SLLV
            compute!(rd = rt shl (rs and 0x1F))
          },
          0x06 => {
            //SRLV
            compute!(rd = rt shr (rs and 0x1F))
          },
          0x07 => {
            //SRAV
            compute!(rd = rt sra (rs and 0x1F))
          },
          0x08 => {
            //JR
            jump!(rs)
          },
          0x09 => {
            //JALR
            call!(rs)
          },
          0x0C => {
            //SYSCALL
            log!("syscall")
          },
          0x0D => {
            //BREAK
            log!("break")
          },
          0x10 => {
            //MFHI
            mov!(rd = hi)
          },
          0x11 => {
            //MTHI
            mov!(hi = rs)
          },
          0x12 => {
            //MFLO
            mov!(rd = lo)
          },
          0x13 => {
            //MTLO
            mov!(lo = rs)
          },
          0x18 => {
            //MULT
            compute!(hi:lo = rs * rt signed)
          },
          0x19 => {
            //MULTU
            compute!(hi:lo = rs * rt)
          },
          0x1A => {
            //DIV
            compute!(hi:lo = rs / rt signed)
          },
          0x1B => {
            //DIVU
            compute!(hi:lo = rs / rt)
          },
          0x20 => {
            //ADD
            compute!(rd = rs wrapping_add rt trap)
          },
          0x21 => {
            //ADDU
            compute!(rd = rs wrapping_add rt)
          },
          0x22 => {
            //SUB
            compute!(rd = rs wrapping_sub rt trap)
          },
          0x23 => {
            //SUBU
            compute!(rd = rs wrapping_sub rt)
          },
          0x24 => {
            //AND
            compute!(rd = rs and rt)
          },
          0x25 => {
            //OR
            compute!(rd = rs or rt)
          },
          0x26 => {
            //XOR
            compute!(rd = rs xor rt)
          },
          0x27 => {
            //NOR
            compute!(rd = rs nor rt)
          },
          0x2A => {
            //SLT
            compute!(rd = rs signed_compare rt)
          },
          0x2B => {
            //SLTU
            compute!(rd = rs compare rt)
          },
          _ => {
            //invalid opcode
            unreachable!("ran into invalid opcode")
          }
        }
      },
      0x01 => {
        //BcondZ
        match get_rt(op) {
          0x00 => {
            //BLTZ
            jump!(rs < 0)
          },
          0x01 => {
            //BGEZ
            jump!(rs >= 0)
          },
          0x80 => {
            //BLTZAL
            call!(rs < 0)
          },
          0x81 => {
            //BGEZAL
            call!(rs >= 0)
          },
          _ => {
            //invalid opcode
            unreachable!("ran into invalid opcode")
          },
        }
      },
      0x02 => {
        //J
        jump!(imm26)
      },
      0x03 => {
        //JAL
        call!(imm26)
      },
      0x04 => {
        //BEQ
        jump!(rs == rt)
      },
      0x05 => {
        //BNE
        jump!(rs != rt)
      },
      0x06 => {
        //BLEZ
        jump!(rs <= 0)
      },
      0x07 => {
        //BGTZ
        jump!(rs > 0)
      },
      0x08 => {
        //ADDI
        compute!(rt = rs wrapping_add imm16 trap)
      },
      0x09 => {
        //ADDIU
        compute!(rt = rs wrapping_add imm16)
      },
      0x0A => {
        //SLTI
        compute!(rt = rs signed_compare imm16)
      },
      0x0B => {
        //SLTIU
        compute!(rt = rs compare imm16)
      },
      0x0C => {
        //ANDI
        compute!(rt = rs and imm16)
      },
      0x0D => {
        //ORI
        compute!(rt = rs or imm16)
      },
      0x0E => {
        //XORI
        compute!(rt = rs xor imm16)
      },
      0x0F => {
        //LUI
        compute!(rt = imm16 shl 16)
      },
      0x10 => {
        //COP0
        cop!(cop0)
      },
      0x11 => {
        //COP1
        unreachable!("COP1 is not implemented on the PSX")
      },
      0x12 => {
        //COP2
        cop!(gte)
      },
      0x13 => {
        //COP3
        unreachable!("COP3 is not implemented on the PSX")
      },
      0x20 => {
        //LB
        mov!(rt = [rs + imm16] read_byte_sign_extended)
      },
      0x21 => {
        //LH
        mov!(rt = [rs + imm16] read_half_sign_extended)
      },
      0x22 => {
        //LWL
        log!("lwl")
      },
      0x23 => {
        //LW
        mov!(rt = [rs + imm16] read_word)
      },
      0x24 => {
        //LBU
        mov!(rt = [rs + imm16] read_byte)
      },
      0x25 => {
        //LHU
        mov!(rt = [rs + imm16] read_half)
      },
      0x26 => {
        //LWR
        log!("lwr")
      },
      0x28 => {
        //SB
        mov!([rs + imm16] = rt write_byte)
      },
      0x29 => {
        //SH
        mov!([rs + imm16] = rt write_half)
      },
      0x2A => {
        //SWL
        log!("swl")
      },
      0x2B => {
        //SW
        mov!([rs + imm16] = rt write_word)
      },
      0x2E => {
        //SWR
        log!("swr")
      },
      0x30 => {
        //LWC0
        unreachable!("LWC0 is not implemented on the PSX")
      },
      0x31 => {
        //LWC1
        unreachable!("LWC1 is not implemented on the PSX")
      },
      0x32 => {
        //LWC2
        log!("lwc2")
      },
      0x33 => {
        //LWC3
        unreachable!("LWC3 is not implemented on the PSX")
      },
      0x38 => {
        //SWC0
        unreachable!("SWC0 is not implemented on the PSX")
      },
      0x39 => {
        //SWC1
        unreachable!("SWC1 is not implemented on the PSX")
      },
      0x3A => {
        //SWC2
        log!("swc2")
      },
      0x3B => {
        //SWC3
        unreachable!("SWC3 is not implemented on the PSX")
      },
      _ => {
        //invalid opcode
        unreachable!("ran into invalid opcode")
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy_bios() {
    //this is the entry point in case we want to test some dummy instructions
    const BIOS: Register = 0x1fc0_0000;
    let mut vm = Interpreter::new(&"/home/ayrton/dev/rspsx/scph1001.bin".to_string(),
                                  None).unwrap();
    vm.memory.write_word(BIOS, 0x0000_0002);
    let dest = 0x0bf0_0000;
    let instr = (2 << 26) | (dest & 0x03ff_ffff);
    vm.memory.write_word(BIOS + 4, 0x0000_0003);
    vm.memory.write_word(BIOS + 8, 0x0000_0004);
    vm.memory.write_word(BIOS + 12, instr);
    vm.memory.write_word(BIOS + 16, 0x0000_0006);
    vm.run(None);
  }
}
