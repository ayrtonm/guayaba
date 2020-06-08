use std::ops::{Add, Shl, Shr, Sub};
use crate::r3000::{MaybeSet, DelayedWrite, Name};
use crate::cop0::Cop0Exception;
use crate::console::Console;
use crate::common::*;
use crate::register::BitTwiddle;
use crate::jit::insn::Insn;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::RegisterMap;
use crate::jit::x64_jit::register_allocator::*;

impl MacroAssembler {
  pub fn emit_insn(&mut self, insn: &Insn, register_map: &mut RegisterMap, logging: bool) {
    let op = insn.op();
    let offset = insn.offset();
    let frame_pointer = register_map.count_spilled();
    let mut stack_pointer = frame_pointer;
    macro_rules! log {
      () => ($crate::print!("\n"));
      ($($arg:tt)*) => ({
        if logging {
          println!($($arg)*);
        };
      })
    }
    ////loading a value from memory is a delayed operation (i.e. the updated register
    ////is not visible to the next opcode). Note that the rs + imm16 in parentheses is
    ////symbolic and only used to improve readability. This macro should be able to
    ////handle all loads in the MIPS instructions set so there's no point to generalizing it
    macro_rules! mov {
    //  (rt = [rs + imm16] left) => {
    //    {
    //      mov!(rt = [rs + imm16] 24_u32, sub lowest_bits shl)
    //    }
    //  };
    //  (rt = [rs + imm16] right) => {
    //    {
    //      mov!(rt = [rs + imm16] 0_u32, add upper_bits_in_place shr)
    //    }
    //  };
    //  (rt = [rs + imm16] $offset:expr, $operator:ident $mask:ident $shift:ident) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      let imm16 = get_imm16(op).half_sign_extended();
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        let rt = vm.delayed_writes.iter()
    //                                    .rev()
    //                                    .find(|write| *write.name() == Name::Rn(t))
    //                                    .map_or(vm.r3000.nth_reg(t),|write| write.value());
    //        let address = rs.wrapping_add(imm16);
    //        let aligned_address = *address.clone().clear_mask(3);
    //        let aligned_word = vm.read_word(aligned_address);
    //        let num_bits = $offset.$operator(8*address.lowest_bits(2));
    //        let result = rt.$mask(num_bits) | aligned_word.$shift(num_bits);
    //        vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(t), result));
    //        None
    //      })
    //    }
    //  };
    //  //delayed aligned reads
    //  (rt = [rs + imm16] $method:ident) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      let imm16 = get_imm16(op).half_sign_extended();
    //      if imm16 == 0 {
    //        Box::new(move |vm| {
    //          let rs = vm.r3000.nth_reg(s);
    //          let result = vm.$method(rs);
    //          vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(t), result));
    //          log!("R{} = [{:#x} + {:#x}] \n  = [{:#x}] \n  = {:#x} {}",
    //                    t, rs, 0, rs, result, stringify!($method));
    //          None
    //        })
    //      } else {
    //        Box::new(move |vm| {
    //          let rs = vm.r3000.nth_reg(s);
    //          let result = vm.$method(rs.wrapping_add(imm16));
    //          vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(t), result));
    //          log!("R{} = [{:#x} + {:#x}] \n  = [{:#x}] \n  = {:#x} {}",
    //                    t, rs, imm16, rs.wrapping_add(imm16), result, stringify!($method));
    //          None
    //        })
    //      }
    //    }
    //  };
    //  ([rs + imm16] = rt left) => {
    //    {
    //      mov!([rs + imm16] = rt 24_u32, sub upper_bits_in_place shr)
    //    }
    //  };
    //  ([rs + imm16] = rt right) => {
    //    {
    //      mov!([rs + imm16] = rt 0_u32, add lowest_bits shl)
    //    }
    //  };
    //  ([rs + imm16] = rt $offset:expr, $operator:ident $mask:ident $shift:ident) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      let imm16 = get_imm16(op).half_sign_extended();
    //      if imm16 == 0 {
    //        Box::new(move |vm| {
    //          let rs = vm.r3000.nth_reg(s);
    //          let rt = vm.r3000.nth_reg(t);
    //          if !vm.cop0.cache_isolated() {
    //            let address = rs;
    //            let aligned_address = *address.clone().clear_mask(3);
    //            let aligned_word = vm.read_word(aligned_address);
    //            let num_bits = $offset.$operator(8*address.lowest_bits(2));
    //            let result = rt.$shift(num_bits) | aligned_word.$mask(num_bits);
    //            vm.write_word(aligned_address, result);
    //          } else {
    //            log!("ignoring write while cache is isolated");
    //          };
    //          None
    //        })
    //      } else {
    //        Box::new(move |vm| {
    //          let rs = vm.r3000.nth_reg(s);
    //          let rt = vm.r3000.nth_reg(t);
    //          if !vm.cop0.cache_isolated() {
    //            let address = rs.wrapping_add(imm16);
    //            let aligned_address = *address.clone().clear_mask(3);
    //            let aligned_word = vm.read_word(aligned_address);
    //            let num_bits = $offset.$operator(8*address.lowest_bits(2));
    //            let result = rt.$shift(num_bits) | aligned_word.$mask(num_bits);
    //            vm.write_word(aligned_address, result);
    //          } else {
    //            log!("ignoring write while cache is isolated");
    //          };
    //          None
    //        })
    //      }
    //    }
    //  };
    //  //aligned writes
      ([rs + imm16] = rt $method:ident) => {
        {
          let s = get_rs(op);
          let t = get_rt(op);
          let imm16 = get_imm16(op).half_sign_extended();
          stack_pointer += self.emit_conditional_push_r64(register_map, X64_RDI);
          let cop0_ptr = MacroAssembler::COP0_POSITION + stack_pointer;
          self.emit_movq_mr_offset(X64_RSP, X64_RDI, cop0_ptr);
          self.emit_movl_mr(X64_RDI, X64_RDI);
          self.emit_btl_ir(16, X64_RDI);
          let skip_write = self.create_undefined_label();
          self.emit_jb_label(skip_write);
          let console_ptr = MacroAssembler::CONSOLE_POSITION + stack_pointer;
          self.emit_movq_mr_offset(X64_RSP, X64_RDI, console_ptr);
          self.emit_swap_mips_registers(register_map, s, X64_RSI);
          self.emit_push_r64(X64_RSI);
          stack_pointer += 8;
          self.emit_addl_ir(imm16 as i32, X64_RSI);
          self.emit_swap_mips_registers(register_map, t, X64_RDX);
          self.emit_function_call(MacroAssembler::WRITE_WORD_POSITION, register_map, stack_pointer);
          self.emit_pop_r64(X64_RSI);
          stack_pointer -= 8;
          stack_pointer += self.emit_conditional_pop_r64(register_map, X64_RDI);
          self.define_label(skip_write);
        }
      };
    //  (lo = rs) => {
    //    {
    //      let s = get_rs(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        let lo = vm.r3000.lo_mut();
    //        *lo = rs;
    //        log!("op1");
    //        None
    //      })
    //    }
    //  };
    //  (hi = rs) => {
    //    {
    //      let s = get_rs(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        let hi = vm.r3000.hi_mut();
    //        *hi = rs;
    //        log!("op2");
    //        None
    //      })
    //    }
    //  };
    //  (rd = lo) => {
    //    {
    //      let d = get_rd(op);
    //      Box::new(move |vm| {
    //        let lo = vm.r3000.lo();
    //        let rd = vm.r3000.nth_reg_mut(d);
    //        vm.modified_register = rd.maybe_set(lo);
    //        log!("op3");
    //        None
    //      })
    //    }
    //  };
    //  (rd = hi) => {
    //    {
    //      let d = get_rd(op);
    //      Box::new(move |vm| {
    //        let hi = vm.r3000.hi();
    //        let rd = vm.r3000.nth_reg_mut(d);
    //        vm.modified_register = rd.maybe_set(hi);
    //        log!("op4");
    //        None
    //      })
    //    }
    //  };
    }
    ////since vm.r3000 is borrowed mutably on the lhs, the rhs must be
    ////computed from the immutable references before assigning its value
    ////to the lhs
    macro_rules! compute {
    //  //ALU instructions with two general purpose registers
    //  (rd = rs $method:ident rt) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      let d = get_rd(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        let rt = vm.r3000.nth_reg(t);
    //        let rd = vm.r3000.nth_reg_mut(d);
    //        vm.modified_register = rd.maybe_set(rs.$method(rt));
    //        log!("R{} = R{} {} R{}\n  = {:#x} {} {:#x}\n  = {:#x}",
    //                  d, s, stringify!($method), t,
    //                  rs, stringify!($method), rt, vm.r3000.nth_reg(d));
    //        None
    //      })
    //    }
    //  };
    //  //ALU instructions with two general purpose registers that trap overflow
    //  (rd = rs $method:ident rt trap) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      let d = get_rd(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s) as u64;
    //        let rt = vm.r3000.nth_reg(t) as u64;
    //        let rd = vm.r3000.nth_reg_mut(d);
    //        let result = rs.$method(rt);
    //        let ret = match result {
    //          Some(result) => {
    //            vm.modified_register = rd.maybe_set(result as u32);
    //            None
    //          },
    //          None => {
    //            let pc = vm.r3000.pc();
    //            Some(vm.cop0.generate_exception(Cop0Exception::Overflow, pc))
    //          },
    //        };
    //        log!("R{} = R{} {} R{} trap overflow\n  = {:#x} {} {:#x}\n  = {:#x}",
    //                  d, s, stringify!($method), t,
    //                  rs, stringify!($method), rt, vm.r3000.nth_reg(d));
    //        ret
    //      })
    //    }
    //  };
    //  //ALU instructions with a register and immediate 16-bit data that trap overflow
    //  (rt = rs $method:ident signed imm16 trap) => {
    //    {
    //      let s = get_rs(op);
    //      let imm16 = get_imm16(op).half_sign_extended() as i32;
    //      let t = get_rt(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s) as i32;
    //        let rt = vm.r3000.nth_reg_mut(t);
    //        let result = rs.$method(imm16);
    //        let ret = match result {
    //          Some(result) => {
    //            vm.modified_register = rt.maybe_set(result as u32);
    //            None
    //          },
    //          None => {
    //            let pc = vm.r3000.pc();
    //            Some(vm.cop0.generate_exception(Cop0Exception::Overflow, pc))
    //          },
    //        };
    //        log!("R{} = R{} {} {:#x} trap overflow\n  = {:#x} {} {:#x}\n  = {:#x}",
    //                  t, s, stringify!($method), imm16,
    //                  rs, stringify!($method), imm16, vm.r3000.nth_reg(t));
    //        ret
    //      })
    //    }
    //  };
      //ALU instructions with a register and immediate 16-bit data
      (rt = rs or imm16) => {
        {
          let s = get_rs(op);
          let imm16 = get_imm16(op) as u16;
          let t = get_rt(op);
          if t != 0 {
            let dest = register_map.mips_to_x64(t).unwrap().bound_gpr();
            if s != 0 {
              let src = register_map.mips_to_x64(s).unwrap().bound_gpr();
              self.emit_movl_rr(src, dest);
              self.emit_orw_ir(imm16, dest);
            } else {
              self.emit_movl_ir(imm16 as u32, dest);
            }
          };
        }
      };
      //ALU instructions with a register and immediate 16-bit data
      (rt = rs wrapping_add signed imm16) => {
        {
          let s = get_rs(op);
          let imm16 = get_imm16(op).half_sign_extended();
          let t = get_rt(op);
          if t != 0 {
            let dest = register_map.mips_to_x64(t).unwrap().bound_gpr();
            if s != 0 {
              let src = register_map.mips_to_x64(s).unwrap().bound_gpr();
              self.emit_movl_rr(src, dest);
              self.emit_addl_ir(imm16 as i32, dest);
            } else {
              self.emit_movl_ir(imm16, dest);
            }
          };
        }
      };
    ////ALU instructions with a register and immediate 16-bit data
    //(rt = rs $method:tt signed imm16) => {
    //  {
    //    let s = get_rs(op);
    //    let imm16 = get_imm16(op).half_sign_extended();
    //    let t = get_rt(op);
    //    if t != 0 {
    //      let dest = register_map.mips_to_x64(t);
    //      if s != 0 {
    //        let src = register_map.mips_to_x64(s);
    //        self.emit_movl_rr(src, dest);
    //        self.emit_addl_ir(imm16, dest);
    //      } else {
    //        self.emit_movl_rr(imm16, dest);
    //      }
    //    };
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        let rt = vm.r3000.nth_reg_mut(t);
    //        vm.modified_register = rt.maybe_set(rs.$method(imm16));
    //        log!("R{} = R{} {} {:#x}\n  = {:#x} {} {:#x}\n  = {:#x}",
    //                  t, s, stringify!($method), imm16,
    //                  rs, stringify!($method), imm16, vm.r3000.nth_reg(t));
    //        None
    //      })
    //  }
    //};
      //shifts a register based on immediate 5 bits
      (rd = rt $method:tt imm5) => {
        {
          let t = get_rt(op);
          let imm5 = get_imm5(op);
          let d = get_rd(op);
          if d != 0 {
            todo!("implement SLL in x64 assembly");
          }
          //Box::new(move |vm| {
          //  let rt = vm.r3000.nth_reg(t);
          //  let rd = vm.r3000.nth_reg_mut(d);
          //  vm.modified_register = rd.maybe_set(rt.$method(imm5));
          //  log!("R{} = R{} {} {:#x}\n  = {:#x} {} {:#x}\n  = {:#x}",
          //            d, t, stringify!($method), imm5,
          //            rt, stringify!($method), imm5, vm.r3000.nth_reg(d));
          //  None
          //})
        }
      };
    //  //shifts a register based on the lowest 5 bits of another register
    //  (rd = rt $method:tt (rs and 0x1F)) => {
    //    {
    //      let t = get_rt(op);
    //      let s = get_rs(op);
    //      let d = get_rd(op);
    //      Box::new(move |vm| {
    //        let rt = vm.r3000.nth_reg(t);
    //        let rs = vm.r3000.nth_reg(s);
    //        let rd = vm.r3000.nth_reg_mut(d);
    //        vm.modified_register = rd.maybe_set(rt.$method(rs & 0x1F));
    //        log!("op9");
    //        None
    //      })
    //    }
    //  };
      (rt = imm16 shl 16) => {
        {
          let t = get_rt(op);
          let imm16 = get_imm16(op);
          let result = imm16 << 16;
          if t != 0 {
            self.emit_movl_ir(result, register_map.mips_to_x64(t).unwrap().bound_gpr());
          };
        }
      };
    //  (hi:lo = rs * rt) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        let rt = vm.r3000.nth_reg(t);
    //        let result = (rs as u64) * (rt as u64);
    //        let hi_res = (result >> 32) as u32;
    //        let lo_res = (result & 0x0000_0000_ffff_ffff) as u32;
    //        let delay = match rs {
    //          0x0000_0000..=0x0000_07ff => {
    //            6
    //          },
    //          0x0000_0800..=0x000f_ffff => {
    //            9
    //          },
    //          0x0010_0000..=0xffff_ffff => {
    //            13
    //          },
    //        };
    //        //TODO: add delay back in
    //        //vm.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, delay));
    //        //vm.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, delay));
    //        *vm.r3000.hi_mut() = hi_res;
    //        *vm.r3000.lo_mut() = lo_res;
    //        log!("op11");
    //        None
    //      })
    //    }
    //  };
    //  (hi:lo = rs * rt signed) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s) as i32;
    //        let rt = vm.r3000.nth_reg(t) as i32;
    //        let result = (rs as i64) * (rt as i64);
    //        let hi_res = (result >> 32) as u32;
    //        let lo_res = (result & 0x0000_0000_ffff_ffff) as u32;
    //        let delay = match rs as u32 {
    //          0x0000_0000..=0x0000_07ff | 0xffff_f800..=0xffff_ffff => {
    //            6
    //          },
    //          0x0000_0800..=0x000f_ffff | 0xfff0_0000..=0xffff_f801 => {
    //            9
    //          },
    //          0x0010_0000..=0x7fff_ffff | 0x8000_0000..=0xfff0_0001 => {
    //            13
    //          },
    //        };
    //        //TODO: add delay back in
    //        //vm.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, delay));
    //        //vm.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, delay));
    //        *vm.r3000.hi_mut() = hi_res;
    //        *vm.r3000.lo_mut() = lo_res;
    //        log!("op11");
    //        None
    //      })
    //    }
    //  };
    //  (hi:lo = rs / rt) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        let rt = vm.r3000.nth_reg(t);
    //        let lo_res = match rt {
    //          0 => {
    //            0xffff_ffff
    //          },
    //          _ => {
    //            rs / rt
    //          },
    //        };
    //        let hi_res = rs % rt;
    //        //TODO: add delay back in
    //        //vm.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, 36));
    //        //vm.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, 36));
    //        *vm.r3000.hi_mut() = hi_res;
    //        *vm.r3000.lo_mut() = lo_res;
    //        log!("op12");
    //        None
    //      })
    //    }
    //  };
    //  (hi:lo = rs / rt signed) => {
    //    {
    //      let s = get_rs(op);
    //      let t = get_rt(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s) as i32;
    //        let rt = vm.r3000.nth_reg(t) as i32;
    //        let lo_res = match rt {
    //          0 => {
    //            match rs {
    //              0x0000_0000..=0x7fff_ffff => -1,
    //              -0x8000_0000..=-1 => 1,
    //            }
    //          },
    //          -1 => {
    //            match rs {
    //              -0x8000_0000..=-1 => 1,
    //              _ => rs / rt,
    //            }
    //          }
    //          _ => rs / rt,
    //        } as u32;
    //        let hi_res = (rs % rt) as u32;
    //        //TODO: add delay back in
    //        //vm.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, 36));
    //        //vm.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, 36));
    //        *vm.r3000.hi_mut() = hi_res;
    //        *vm.r3000.lo_mut() = lo_res;
    //        log!("op12");
    //        None
    //      })
    //    }
    //  };
    }
    //macro_rules! cop {
    //  ($copn:ident) => {
    //    {
    //      match get_rs(op) {
    //        0x00 => {
    //          //MFCn
    //          let t = get_rt(op);
    //          let d = get_rd(op);
    //          Box::new(move |vm| {
    //            let rd_data = vm.$copn.nth_data_reg(d);
    //            vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(t), rd_data));
    //            log!("R{} = {}R{}\n  = {:#x} after the delay slot",
    //                      t, stringify!($copn), d, rd_data);
    //            None
    //          })
    //        },
    //        0x02 => {
    //          //CFCn
    //          let t = get_rt(op);
    //          let d = get_rd(op);
    //          Box::new(move |vm| {
    //            let rd_ctrl = vm.$copn.nth_ctrl_reg(d);
    //            vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(t), rd_ctrl));
    //            None
    //          })
    //        },
    //        0x04 => {
    //          //MTCn
    //          let t = get_rt(op);
    //          let d = get_rd(op);
    //          Box::new(move |vm| {
    //            let rt = vm.r3000.nth_reg(t);
    //            let rd = vm.$copn.nth_data_reg_mut(d);
    //            vm.modified_register = rd.maybe_set(rt);
    //            log!("{}R{} = R{}\n  = {:#x}",
    //                      stringify!($copn), d, t,
    //                      vm.$copn.nth_data_reg(d));
    //            None
    //          })
    //        },
    //        0x06 => {
    //          //CTCn
    //          let t = get_rt(op);
    //          let d = get_rd(op);
    //          Box::new(move |vm| {
    //            let rt = vm.r3000.nth_reg(t);
    //            let rd = vm.$copn.nth_ctrl_reg_mut(d);
    //            vm.modified_register = rd.maybe_set(rt);
    //            None
    //          })
    //        },
    //        0x08 => {
    //          match get_rt(op) {
    //            0x00 => {
    //              //BCnF
    //              let imm16 = get_imm16(op);
    //              Box::new(move |vm| {
    //                vm.$copn.bcnf(imm16);
    //                None
    //              })
    //            },
    //            0x01 => {
    //              //BCnT
    //              //technically we're implementing one illegal instruction here
    //              //since BCnT is not implemented for COP0
    //              //however, GTE (i.e. COP2) does implement it
    //              Box::new(move |vm| {
    //                None
    //              })
    //            },
    //            _ => {
    //              unreachable!("ran into invalid opcode")
    //            },
    //          }
    //        },
    //        0x10..=0x1F => {
    //          //COPn imm25
    //          let imm25 = get_imm25(op);
    //          Box::new(move |vm| {
    //            vm.$copn.execute_command(imm25);
    //            None
    //          })
    //        },
    //        _ => {
    //          unreachable!("ran into invalid opcode")
    //        },
    //      }
    //    }
    //  }
    //}
    macro_rules! jump {
      (imm26) => {
        {
          let imm26 = get_imm26(op);
          let shifted_imm26 = imm26 * 4;
          stack_pointer += self.emit_conditional_push_r64(register_map, X64_R14);
          stack_pointer += self.emit_conditional_push_r64(register_map, X64_R15);
          self.emit_movq_mr_offset(X64_RSP, X64_R14, stack_pointer);
          let pc_idx = 31;
          self.emit_movl_mr_offset(X64_R14, X64_R15, 4 * pc_idx);
          self.emit_addl_ir(offset as i32, X64_R15);
          self.emit_andl_ir(0xf000_0000, X64_R15);
          self.emit_addl_ir(shifted_imm26 as i32, X64_R15);
          self.emit_movl_rm_offset(X64_R15, X64_R14, 4 * pc_idx);
          stack_pointer += self.emit_conditional_pop_r64(register_map, X64_R15);
          stack_pointer += self.emit_conditional_pop_r64(register_map, X64_R14);
    //      Box::new(move |vm| {
    //        let pc = vm.r3000.pc().wrapping_add(offset);
    //        let pc_hi_bits = pc & 0xf000_0000;
    //        let dest = pc_hi_bits.wrapping_add(shifted_imm26);
    //        log!("jumping to (PC & 0xf0000000) + ({:#x} * 4)\n  = {:#x} + {:#x}\n  = {:#x} after the delay slot",
    //                  imm26, pc_hi_bits, shifted_imm26, dest);
    //        Some(dest)
    //      })
        }
      };
    //  (rs) => {
    //    {
    //      let s = get_rs(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        if rs & 0x0000_0003 != 0 {
    //          let pc = vm.r3000.pc().wrapping_add(offset);
    //          log!("ignoring jumping to R{} = {:#x} and generating an exception", s, rs);
    //          Some(vm.cop0.generate_exception(Cop0Exception::LoadAddress, pc))
    //        } else {
    //          log!("jumping to R{} = {:#x} after the delay slot", s, rs);
    //          Some(rs)
    //        }
    //      })
    //    }
    //  };
    //  (rs $cmp:tt rt) => {
    //    {
    //      let t = get_rt(op);
    //      let s = get_rs(op);
    //      let imm16 = get_imm16(op);
    //      let inc = ((imm16.half_sign_extended() as i32) * 4) as u32;
    //      Box::new(move |vm| {
    //        let rt = vm.r3000.nth_reg(t);
    //        let rs = vm.r3000.nth_reg(s);
    //        if rs $cmp rt {
    //          let pc = vm.r3000.pc().wrapping_add(offset);
    //          let dest = pc.wrapping_add(inc);
    //          log!("jumping to PC + ({:#x} * 4) = {:#x} + {:#x} = {:#x} after the delay slot\n  since R{} {} R{} -> {:#x} {} {:#x}",
    //                    imm16, pc, inc, dest, s, stringify!($cmp), t, rs, stringify!($cmp), rt);
    //          Some(dest)
    //        } else {
    //          log!("skipping jump since R{} {} R{} -> {:#x} {} {:#x} is false",
    //                    s, stringify!($cmp), t, rs, stringify!($cmp), rt);
    //          None
    //        }
    //      })
    //    }
    //  };
    //  (rs $cmp:tt 0) => {
    //    {
    //      let imm16 = get_imm16(op);
    //      let inc = ((imm16 as i16) * 4) as u32;
    //      let s = get_rs(op);
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        log!("op16");
    //        if (rs as i32) $cmp 0 {
    //          let pc = vm.r3000.pc().wrapping_add(offset);
    //          let dest = pc.wrapping_add(inc);
    //          Some(dest)
    //        } else {
    //          None
    //        }
    //      })
    //    }
    //  };
    }
    //macro_rules! call {
    //  (imm26) => {
    //    {
    //      let imm26 = get_imm26(op);
    //      let shifted_imm26 = imm26 * 4;
    //      Box::new(move |vm| {
    //        let pc = vm.r3000.pc().wrapping_add(offset);
    //        let ret = pc.wrapping_add(4);
    //        vm.modified_register = vm.r3000.ra_mut().maybe_set(ret);
    //        log!("R31 = {:#x}", ret);
    //        let pc_hi_bits = pc & 0xf000_0000;
    //        let dest = pc_hi_bits.wrapping_add(shifted_imm26);
    //        log!("jumping to (PC & 0xf0000000) + ({:#x} * 4)\n  = {:#x} + {:#x}\n  = {:#x} after the delay slot",
    //                  imm26, pc_hi_bits, shifted_imm26, dest);
    //        Some(dest)
    //      })
    //    }
    //  };
    //  (rs) => {
    //    {
    //      let d = get_rd(op);
    //      let s = get_rs(op);
    //      Box::new(move |vm| {
    //        let pc = vm.r3000.pc().wrapping_add(offset);
    //        let result = pc.wrapping_add(4);
    //        let rd = vm.r3000.nth_reg_mut(d);
    //        vm.modified_register = rd.maybe_set(result);
    //        log!("op18");
    //        let rs = vm.r3000.nth_reg(s);
    //        if rs & 0x0000_0003 != 0 {
    //          log!("ignoring jumping to R{} = {:#x} and generating an exception", s, rs);
    //          Some(vm.cop0.generate_exception(Cop0Exception::LoadAddress, pc))
    //        } else {
    //          log!("jumping to R{} = {:#x} after the delay slot", s, rs);
    //          Some(rs)
    //        }
    //      })
    //    }
    //  };
    //  (rs $cmp:tt rt) => {
    //    {
    //      let t = get_rt(op);
    //      let s = get_rs(op);
    //      let imm16 = get_imm16(op);
    //      let inc = ((imm16 as i16) * 4) as u32;
    //      Box::new(move |vm| {
    //        let rt = vm.r3000.nth_reg(t);
    //        let rs = vm.r3000.nth_reg(s);
    //        log!("op19");
    //        if *rs $cmp *rt {
    //          let pc = vm.r3000.pc().wrapping_add(offset);
    //          let ret = pc.wrapping_add(4);
    //          vm.modified_register = vm.r3000.ra_mut().maybe_set(ret);
    //          let dest = pc.wrapping_add(inc).wrapping_add(offset);
    //          Some(dest)
    //        } else {
    //          None
    //        }
    //      })
    //    }
    //  };
    //  (rs $cmp:tt 0) => {
    //    {
    //      let s = get_rs(op);
    //      let imm16 = get_imm16(op);
    //      let inc = imm16 * 4;
    //      Box::new(move |vm| {
    //        let rs = vm.r3000.nth_reg(s);
    //        log!("op20");
    //        if (rs as i32) $cmp 0 {
    //          let pc = vm.r3000.pc().wrapping_add(offset);
    //          let ret = pc.wrapping_add(4);
    //          vm.modified_register = vm.r3000.ra_mut().maybe_set(ret);
    //          let dest = pc.wrapping_add(inc);
    //          Some(dest)
    //        } else {
    //          None
    //        }
    //      })
    //    }
    //  };
    //}
    match get_primary_field(op) {
      0x00 => {
        //SPECIAL
        match get_secondary_field(op) {
          0x00 => {
            //SLL
            log!("> SLL");
            compute!(rd = rt shl imm5)
          },
    //      0x02 => {
    //        //SRL
    //        log!("> SRL");
    //        compute!(rd = rt shr imm5)
    //      },
    //      0x03 => {
    //        //SRA
    //        log!("> SRA");
    //        compute!(rd = rt sra imm5)
    //      },
    //      0x04 => {
    //        //SLLV
    //        log!("> SLLV");
    //        compute!(rd = rt shl (rs and 0x1F))
    //      },
    //      0x06 => {
    //        //SRLV
    //        log!("> SRLV");
    //        compute!(rd = rt shr (rs and 0x1F))
    //      },
    //      0x07 => {
    //        //SRAV
    //        log!("> SRAV");
    //        compute!(rd = rt sra (rs and 0x1F))
    //      },
    //      0x08 => {
    //        //JR
    //        log!("> JR");
    //        jump!(rs)
    //      },
    //      0x09 => {
    //        //JALR
    //        log!("> JALR");
    //        call!(rs)
    //      },
    //      0x0C => {
    //        //SYSCALL
    //        log!("> SYSCALL");
    //        Box::new(move |vm| {
    //          let pc = vm.r3000.pc();
    //          Some(vm.cop0.generate_exception(Cop0Exception::Syscall, pc))
    //        })
    //      },
    //      0x0D => {
    //        //BREAK
    //        log!("> BREAK");
    //        Box::new(move |vm| {
    //          todo!("implement a JIT closure for break")
    //        })
    //      },
    //      0x10 => {
    //        //MFHI
    //        log!("> MFHI");
    //        mov!(rd = hi)
    //      },
    //      0x11 => {
    //        //MTHI
    //        log!("> MTHI");
    //        mov!(hi = rs)
    //      },
    //      0x12 => {
    //        //MFLO
    //        log!("> MFLO");
    //        mov!(rd = lo)
    //      },
    //      0x13 => {
    //        //MTLO
    //        log!("> MTLO");
    //        mov!(lo = rs)
    //      },
    //      0x18 => {
    //        //MULT
    //        log!("> MULT");
    //        compute!(hi:lo = rs * rt signed)
    //      },
    //      0x19 => {
    //        //MULTU
    //        log!("> MULTU");
    //        compute!(hi:lo = rs * rt)
    //      },
    //      0x1A => {
    //        //DIV
    //        log!("> DIV");
    //        compute!(hi:lo = rs / rt signed)
    //      },
    //      0x1B => {
    //        //DIVU
    //        log!("> DIVU");
    //        compute!(hi:lo = rs / rt)
    //      },
    //      0x20 => {
    //        //ADD
    //        log!("> ADD");
    //        compute!(rd = rs checked_add rt trap)
    //      },
    //      0x21 => {
    //        //ADDU
    //        log!("> ADDU");
    //        compute!(rd = rs wrapping_add rt)
    //      },
    //      0x22 => {
    //        //SUB
    //        log!("> SUB");
    //        compute!(rd = rs checked_sub rt trap)
    //      },
    //      0x23 => {
    //        //SUBU
    //        log!("> SUBU");
    //        compute!(rd = rs wrapping_sub rt)
    //      },
    //      0x24 => {
    //        //AND
    //        log!("> AND");
    //        compute!(rd = rs and rt)
    //      },
    //      0x25 => {
    //        //OR
    //        log!("> OR");
    //        compute!(rd = rs or rt)
    //      },
    //      0x26 => {
    //        //XOR
    //        log!("> XOR");
    //        compute!(rd = rs xor rt)
    //      },
    //      0x27 => {
    //        //NOR
    //        log!("> NOR");
    //        compute!(rd = rs nor rt)
    //      },
    //      0x2A => {
    //        //SLT
    //        log!("> SLT");
    //        compute!(rd = rs signed_compare rt)
    //      },
    //      0x2B => {
    //        //SLTU
    //        log!("> SLTU");
    //        compute!(rd = rs compare rt)
    //      },
          _ => {
            //invalid opcode
            unreachable!("ran into invalid opcode {:#x}", op)
          }
        }
      },
    //  0x01 => {
    //    //BcondZ
    //    match get_rt(op) {
    //      0x00 => {
    //        //BLTZ
    //        log!("> BLTZ");
    //        jump!(rs < 0)
    //      },
    //      0x01 => {
    //        //BGEZ
    //        log!("> BGEZ");
    //        jump!(rs >= 0)
    //      },
    //      0x80 => {
    //        //BLTZAL
    //        log!("> BLTZAL");
    //        call!(rs < 0)
    //      },
    //      0x81 => {
    //        //BGEZAL
    //        log!("> BGEZAL");
    //        call!(rs >= 0)
    //      },
    //      _ => {
    //        //invalid opcode
    //        unreachable!("ran into invalid opcode")
    //      },
    //    }
    //  },
      0x02 => {
        //J
        log!("> J");
        jump!(imm26)
      },
    //  0x03 => {
    //    //JAL
    //    log!("> JAL");
    //    call!(imm26)
    //  },
    //  0x04 => {
    //    //BEQ
    //    log!("> BEQ");
    //    jump!(rs == rt)
    //  },
    //  0x05 => {
    //    //BNE
    //    log!("> BNE");
    //    jump!(rs != rt)
    //  },
    //  0x06 => {
    //    //BLEZ
    //    log!("> BLEZ");
    //    jump!(rs <= 0)
    //  },
    //  0x07 => {
    //    //BGTZ
    //    log!("> BGTZ");
    //    jump!(rs > 0)
    //  },
    //  0x08 => {
    //    //ADDI
    //    log!("> ADDI");
    //    compute!(rt = rs checked_add signed imm16 trap)
    //  },
      0x09 => {
        //ADDIU
        log!("> ADDIU");
        compute!(rt = rs wrapping_add signed imm16)
      },
    //  0x0A => {
    //    //SLTI
    //    log!("> SLTI");
    //    compute!(rt = rs signed_compare imm16)
    //  },
    //  0x0B => {
    //    //SLTIU
    //    log!("> SLTIU");
    //    compute!(rt = rs compare imm16)
    //  },
    //  0x0C => {
    //    //ANDI
    //    log!("> ANDI");
    //    compute!(rt = rs and imm16)
    //  },
      0x0D => {
        //ORI
        log!("> ORI");
        compute!(rt = rs or imm16)
      },
    //  0x0E => {
    //    //XORI
    //    log!("> XORI");
    //    compute!(rt = rs xor imm16)
    //  },
      0x0F => {
        //LUI
        log!("> LUI");
        compute!(rt = imm16 shl 16)
      },
    //  0x10 => {
    //    //COP0
    //    log!("> COP0");
    //    cop!(cop0)
    //  },
    //  0x11 => {
    //    //COP1
    //    unreachable!("COP1 is not implemented on the PSX")
    //  },
    //  0x12 => {
    //    //COP2
    //    log!("> COP2");
    //    cop!(gte)
    //  },
    //  0x13 => {
    //    //COP3
    //    unreachable!("COP3 is not implemented on the PSX")
    //  },
    //  0x20 => {
    //    //LB
    //    log!("> LB");
    //    mov!(rt = [rs + imm16] read_byte_sign_extended)
    //  },
    //  0x21 => {
    //    //LH
    //    log!("> LH");
    //    mov!(rt = [rs + imm16] read_half_sign_extended)
    //  },
    //  0x22 => {
    //    //LWL
    //    log!("> LWL");
    //    mov!(rt = [rs + imm16] left)
    //  },
    //  0x23 => {
    //    //LW
    //    log!("> LW");
    //    mov!(rt = [rs + imm16] read_word)
    //  },
    //  0x24 => {
    //    //LBU
    //    log!("> LBU");
    //    mov!(rt = [rs + imm16] read_byte)
    //  },
    //  0x25 => {
    //    //LHU
    //    log!("> LHU");
    //    mov!(rt = [rs + imm16] read_half)
    //  },
    //  0x26 => {
    //    //LWR
    //    log!("> LWR");
    //    mov!(rt = [rs + imm16] right)
    //  },
    //  0x28 => {
    //    //SB
    //    log!("> SB");
    //    mov!([rs + imm16] = rt write_byte)
    //  },
    //  0x29 => {
    //    //SH
    //    log!("> SH");
    //    mov!([rs + imm16] = rt write_half)
    //  },
    //  0x2A => {
    //    //SWL
    //    log!("> SWL");
    //    mov!([rs + imm16] = rt left)
    //  },
      0x2B => {
        //SW
        log!("> SW");
        mov!([rs + imm16] = rt write_word)
      },
    //  0x2E => {
    //    //SWR
    //    log!("> SWR");
    //    mov!([rs + imm16] = rt right)
    //  },
    //  0x30 => {
    //    //LWC0
    //    unreachable!("LWC0 is not implemented on the PSX")
    //  },
    //  0x31 => {
    //    //LWC1
    //    unreachable!("LWC1 is not implemented on the PSX")
    //  },
    //  0x32 => {
    //    //LWC2
    //    todo!("lwc2")
    //  },
    //  0x33 => {
    //    //LWC3
    //    unreachable!("LWC3 is not implemented on the PSX")
    //  },
    //  0x38 => {
    //    //SWC0
    //    unreachable!("SWC0 is not implemented on the PSX")
    //  },
    //  0x39 => {
    //    //SWC1
    //    unreachable!("SWC1 is not implemented on the PSX")
    //  },
    //  0x3A => {
    //    //SWC2
    //    todo!("swc2")
    //  },
    //  0x3B => {
    //    //SWC3
    //    unreachable!("SWC3 is not implemented on the PSX")
    //  },
      _ => {
        //invalid opcode
        todo!("ran into unimplemented opcode {:#x?}", op)
      }
    };
    assert_eq!(stack_pointer, frame_pointer);
  }
}
