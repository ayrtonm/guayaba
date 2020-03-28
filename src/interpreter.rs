use std::io;
use std::ops::Shl;
use std::ops::Shr;
use crate::common::*;
use crate::register::Register;
use crate::register::Parts;
use crate::r3000::R3000;
use crate::r3000::DelayedWrite;
use crate::r3000::Name;
use crate::memory::Memory;
use crate::cd::CD;

pub struct Interpreter {
  r3000: R3000,
  memory: Memory,
  cd: Option<CD>,
  next_pc: Option<Register>,
  //these are register writes due to memory loads which happen after one cycle
  delayed_writes: Vec<DelayedWrite>,
}

impl Interpreter {
  pub fn new(bios_filename: &String, infile: Option<&String>) -> io::Result<Self> {
    let r3000 = R3000::new();
    let memory = Memory::new(bios_filename)?;
    let cd = infile.and_then(|f| CD::new(f).ok());
    let delayed_writes = Vec::new();
    Ok(Interpreter {
      r3000,
      memory,
      cd,
      next_pc: None,
      delayed_writes,
    })
  }
  pub fn run(&mut self, n: Option<u32>) {
    println!("{:#x?}", self.r3000);
    n.map(|n| {
      for _ in 0..n {
        self.step();
      }
    });
    println!("{:#x?}", self.r3000);
    self.cd.as_ref().map(|cd| cd.preview(10));
  }
  fn step(&mut self) {
    //get opcode from memory at program counter
    let op = self.memory.read_word(*self.r3000.pc());
    //the instruction following each jump is always executed before updating the pc
    //increment the program counter
    *self.r3000.pc_mut() = self.next_pc
                           .take()
                           .map_or_else(|| *self.r3000.pc() + 4, |next_pc| next_pc);
    self.next_pc = self.execute_opcode(op);
  }
  //if program counter should incremented normally, return None
  //otherwise return Some(new program counter)
  fn execute_opcode(&mut self, op: u32) -> Option<Register> {
    //loading a value from memory is a delayed operation (i.e. the updated register
    //is not visible to the next opcode). Note that the rs + imm16 in parentheses is
    //symbolic and only used to improve readability. This macro should be able to
    //handle all loads in the MIPS instructions set so there's no point to generalizing it
    macro_rules! mov {
      //delayed aligned reads
      (rt = [rs + imm16] $method:ident) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let imm16 = get_imm16(op);
          let rt = get_rt(op);
          self.delayed_writes.push(DelayedWrite::new(Name::Rn(rt),
                                   self.memory.$method(rs + imm16), 1));
          None
        }
      };
      //aligned writes
      ([rs + imm16] = rt $method:ident) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let rt = self.r3000.nth_reg(get_rt(op));
          let imm16 = get_imm16(op);
          println!("moving {:#x} to {:#x} with pc at {:#x}", *rt, rs + imm16, self.r3000.pc());
          self.memory.$method(rs + imm16, *rt);
          None
        }
      };
      (lo = rs) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let result = *rs;
          let lo = self.r3000.lo_mut();
          *lo = result;
          None
        }
      };
      (hi = rs) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let result = *rs;
          let hi = self.r3000.hi_mut();
          *hi = result;
          None
        }
      };
      (rd = lo) => {
        {
          let lo = self.r3000.lo();
          let result = *lo;
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = result);
          None
        }
      };
      (rd = hi) => {
        {
          let hi = self.r3000.hi();
          let result = *hi;
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = result);
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
          let result = rs.$method(*rt);
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = result);
          None
        }
      };
      //ALU instructions with two general purpose registers that trap overflow
      (rd = rs $method:ident rt trap) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let rt = self.r3000.nth_reg(get_rt(op));
          let result = rs.$method(*rt);
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = result);
          None
        }
      };
      //ALU instructions with a register and immediate 16-bit data
      (rt = rs $method:tt imm16) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          let imm16 = get_imm16(op);
          let result = rs.$method(imm16);
          let rt = self.r3000.nth_reg_mut(get_rt(op));
          rt.map(|rt| *rt = result);
          None
        }
      };
      //shifts a register based on immediate 5 bits
      (rd = rt $method:tt imm5) => {
        {
          let rt = self.r3000.nth_reg(get_rt(op));
          let imm5 = get_imm5(op);
          let result = rt.$method(imm5);
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = result);
          None
        }
      };
      //shifts a register based on the lowest 5 bits of another register
      (rd = rt $method:tt (rs and 0x1F)) => {
        {
          let rt = self.r3000.nth_reg(get_rt(op));
          let rs = self.r3000.nth_reg(get_rs(op));
          let result = rt.$method(rs & 0x1F);
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = result);
          None
        }
      };
      (rt = imm16 shl 16) => {
        {
          let rt = self.r3000.nth_reg_mut(get_rt(op));
          let imm16 = get_imm16(op);
          rt.map(|rt| *rt = imm16 << 16);
          None
        }
      };
      (hi:lo = rs * rt) => {
        {
          let rs = *self.r3000.nth_reg(get_rs(op));
          let rt = *self.r3000.nth_reg(get_rt(op));
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
          None
        }
      };
      (hi:lo = rs / rt) => {
        {
          let rs = *self.r3000.nth_reg(get_rs(op));
          let rt = *self.r3000.nth_reg(get_rt(op));
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
          None
        }
      };
    }
    macro_rules! jump {
      (imm26) => {
        {
          let imm = get_imm26(op);
          let dest = (*self.r3000.pc() & 0xf000_0000) + (imm * 4);
          Some(dest)
        }
      };
      (rs) => {
        {
          let rs = self.r3000.nth_reg(get_rs(op));
          Some(*rs)
        }
      };
      (rs $cmp:tt rt) => {
        {
          let rt = self.r3000.nth_reg(get_rt(op));
          let rs = self.r3000.nth_reg(get_rs(op));
          if *rs $cmp *rt {
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
          if (*rs as i32) $cmp 0 {
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
          jump!(imm26)
        }
      };
      (rs) => {
        {
          let result = self.r3000.pc() + 4;
          let rd = self.r3000.nth_reg_mut(get_rd(op));
          rd.map(|rd| *rd = result);
          jump!(rs)
        }
      };
      (rs $cmp:tt rt) => {
        {
          let rt = self.r3000.nth_reg(get_rt(op));
          let rs = self.r3000.nth_reg(get_rs(op));
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
          if (*rs as i32) $cmp 0 {
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
    //after executing an opcode, complete the loads from the previous opcode
    self.r3000.flush_write_cache(&mut self.delayed_writes);
    let a = get_primary_field(op);
    println!("primary field is {:#x?}", a);
    //this match statement optionally returns the next program counter
    //if the return value is None, then we increment pc as normal
    match a {
      0x00 => {
        //SPECIAL
        let b = get_secondary_field(op);
        println!("secondary field is {:#x?}", b);
        match b {
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
            None
          },
          0x0D => {
            //BREAK
            None
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
            None
          },
          0x19 => {
            //MULTU
            compute!(hi:lo = rs * rt)
          },
          0x1A => {
            //DIV
            None
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
            None
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
            panic!("ran into invalid opcode")
          }
        }
      },
      0x01 => {
        //BcondZ
        let c = get_rt(op);
        match c {
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
            panic!("ran into invalid opcode")
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
        None
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
        None
      },
      0x11 => {
        //COP1
        None
      },
      0x12 => {
        //COP2
        None
      },
      0x13 => {
        //COP3
        None
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
        None
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
        None
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
        None
      },
      0x2B => {
        //SW
        mov!([rs + imm16] = rt write_word)
      },
      0x2E => {
        //SWR
        None
      },
      0x30 => {
        //LWC0
        None
      },
      0x31 => {
        //LWC1
        None
      },
      0x32 => {
        //LWC2
        None
      },
      0x33 => {
        //LWC3
        None
      },
      0x38 => {
        //SWC0
        None
      },
      0x39 => {
        //SWC1
        None
      },
      0x3A => {
        //SWC2
        None
      },
      0x3B => {
        //SWC3
        None
      },
      _ => {
        //invalid opcode
        panic!("ran into invalid opcode")
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
