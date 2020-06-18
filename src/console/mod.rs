use std::io;
use std::collections::{VecDeque, HashSet};
use crate::register::BitTwiddle;
use r3000::R3000;
use cop0::Cop0;
use memory::{Memory, MemAction, MemResponse};
use cd::CD;
use gpu::GPU;
use gte::GTE;
use screen::Screen;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub mod r3000;
pub mod cop0;
mod memory;
mod dma;
mod gte;
mod gpu;
mod screen;
mod cd;
mod handle_dma;

pub trait MaybeSet {
  fn maybe_set(self, value: u32) -> Option<Name>;
}

//different types of register names
//these are for improved readability when doing delayed register writes
#[derive(Debug,PartialEq)]
pub enum Name {
  Rn(u32),
  Hi,
  Lo,
}

//this represents a delayed write operation
#[derive(Debug)]
pub struct DelayedWrite {
  register_name: Name,
  value: u32,
}

impl DelayedWrite {
  pub fn new(register_name: Name, value: u32) -> Self {
    DelayedWrite {
      register_name,
      value,
    }
  }
  pub fn name(&self) -> &Name {
    &self.register_name
  }
  pub fn value(&self) -> u32 {
    self.value
  }
}

macro_rules! handle_action {
  ($write:expr, $self:ident) => {
    match $write {
      MemAction::DMA(transfer) => {
        $self.handle_dma(transfer);
      },
      MemAction::GpuGp0(value) => $self.gpu.write_to_gp0(value),
      MemAction::GpuGp1(value) => $self.gpu.write_to_gp1(value),
      MemAction::CDCmd(value) => {
        $self.cd.send_command(value);
      },
      MemAction::CDParam(value) => {
        $self.cd.send_parameter(value);
      },
      MemAction::CDCmdParam(cmd, param) => {
        $self.cd.send_command(cmd);
        $self.cd.send_parameter(param);
      },
      MemAction::Interrupt(irq) => {
        $self.cop0.request_interrupt(irq);
      },
      MemAction::None => {
      },
    };
  }
}

macro_rules! handle_response {
  ($read:expr, $self:ident) => {
    match $read {
      MemResponse::Value(value) => value,
      MemResponse::GPUREAD => $self.gpu.gpuread(),
      MemResponse::GPUSTAT => $self.gpu.gpustat(),
      MemResponse::CDResponse => $self.cd.read_response(),
    }
  }
}

pub struct Console {
  //these correspond to physical components
  pub r3000: R3000,
  pub cop0: Cop0,
  pub memory: Memory,
  pub gpu: GPU,
  pub gte: GTE,
  pub cd: CD,
  pub screen: Screen,

  pub next_pc: Option<u32>,
  pub delayed_writes: VecDeque<DelayedWrite>,
  pub modified_register: Option<Name>,
  pub overwritten: HashSet<u32>,
  pub i: u32,
}

impl Console {
  pub const REFRESH_RATE: i64 = 550_000;
  pub fn new(bios_filename: &String, infile: Option<&String>, gpu_logging: bool,
             wx: u32, wy: u32) -> io::Result<Self> {
    let r3000 = R3000::new();
    let cop0: Cop0 = Default::default();
    let memory = Memory::new(bios_filename)?;
    let gpu = GPU::new(gpu_logging);
    let gte = Default::default();
    let cd = CD::new(infile);
    let screen = Screen::new(wx, wy);
    let delayed_writes = VecDeque::new();
    Ok(Self {
      r3000,
      cop0,
      memory,
      gpu,
      gte,
      cd,
      screen,
      next_pc: None,
      delayed_writes,
      modified_register: None,
      i: 0,
      overwritten: Default::default(),
    })
  }
  pub fn handle_events(&mut self) -> bool {
    let event_rate: u32 = 100_000;
    if self.i % event_rate == 0 {
      for event in self.screen.event_pump().poll_iter() {
        match event {
          Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            println!("Executed {} steps", self.i);
            return false;
          },
          Event::KeyDown { keycode: Some(Keycode::S), .. } => {
            println!("You pressed X");
          },
          Event::KeyDown { keycode: Some(Keycode::D), .. } => {
            println!("You pressed ◯");
          },
          Event::KeyDown { keycode: Some(Keycode::A), .. } => {
            println!("You pressed □");
          },
          Event::KeyDown { keycode: Some(Keycode::W), .. } => {
            println!("You pressed △");
          },
          Event::KeyDown { keycode: Some(Keycode::K), .. } => {
            println!("You pressed down");
          },
          Event::KeyDown { keycode: Some(Keycode::L), .. } => {
            println!("You pressed right");
          },
          Event::KeyDown { keycode: Some(Keycode::J), .. } => {
            println!("You pressed left");
          },
          Event::KeyDown { keycode: Some(Keycode::I), .. } => {
            println!("You pressed up");
          },
          Event::Quit {..} => panic!(""),
          _ => {},
        }
      }
    }
    true
  }
  pub extern fn read_byte_sign_extended(&mut self, address: u32) -> u32 {
    handle_response!(self.memory.read_byte_sign_extended(address), self)
  }
  pub extern fn read_half_sign_extended(&mut self, address: u32) -> u32 {
    handle_response!(self.memory.read_half_sign_extended(address), self)
  }
  pub extern fn read_byte(&mut self, address: u32) -> u32 {
    handle_response!(self.memory.read_byte(address), self)
  }
  pub extern fn read_half(&mut self, address: u32) -> u32 {
    handle_response!(self.memory.read_half(address), self)
  }
  pub extern fn read_word(&mut self, address: u32) -> u32 {
    handle_response!(self.memory.read_word(address), self)
  }
  pub extern fn write_byte(&mut self, address: u32, value: u32) {
    self.overwritten.insert(Console::physical(address));
    handle_action!(self.memory.write_byte(address, value), self);
  }
  pub extern fn write_half(&mut self, address: u32, value: u32) {
    self.overwritten.insert(Console::physical(address));
    handle_action!(self.memory.write_half(address, value), self);
  }
  pub extern fn write_word(&mut self, address: u32, value: u32) {
    self.overwritten.insert(Console::physical(address));
    handle_action!(self.memory.write_word(address, value), self);
  }
  pub fn physical(address: u32) -> u32 {
    const PHYS_MASK: [u32; 8] = [0xffff_ffff, 0xffff_ffff, 0xffff_ffff, 0xffff_ffff,
                                 0x7fff_ffff, 0x1fff_ffff, 0xffff_ffff, 0xffff_ffff];
    let idx = address.upper_bits(3) as usize;
    address & PHYS_MASK[idx]
  }

}

