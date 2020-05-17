use std::io;
use std::collections::VecDeque;
use std::collections::HashSet;
use crate::register::Register;
use crate::register::BitBang;
use crate::r3000::R3000;
use crate::r3000::DelayedWrite;
use crate::r3000::Name;
use crate::cop0::Cop0;
use crate::memory::Memory;
use crate::memory::MemAction;
use crate::memory::MemResponse;
use crate::cd::CD;
use crate::gpu::GPU;
use crate::gte::GTE;
use crate::screen::Screen;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod handle_dma;

pub struct Console {
  //these correspond to physical components
  pub r3000: R3000,
  pub cop0: Cop0,
  pub memory: Memory,
  pub gpu: GPU,
  pub gte: GTE,
  pub cd: CD,
  pub screen: Screen,

  pub next_pc: Option<Register>,
  pub delayed_writes: VecDeque<DelayedWrite>,
  pub modified_register: Option<Name>,
  pub overwritten: HashSet<Register>,
  pub i: u32,
}

impl Console {
  pub const REFRESH_RATE: i64 = 550_000;
  pub fn new(bios_filename: &String, infile: Option<&String>, gpu_logging: bool,
             wx: u32, wy: u32) -> io::Result<Self> {
    let r3000 = R3000::new();
    let cop0 = Default::default();
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
  pub fn resolve_memresponse(&mut self, response: MemResponse) -> Register {
    match response {
      MemResponse::Value(value) => value,
      MemResponse::GPUREAD => self.gpu.gpuread(),
      MemResponse::GPUSTAT => self.gpu.gpustat(),
      MemResponse::CDResponse => self.cd.read_response(),
    }
  }
  pub fn write_byte(&mut self, address: Register, value: Register) {
    self.overwritten.insert(Console::physical(address));
    match self.memory.write_byte(address, value) {
              MemAction::DMA(transfer) => {
                self.handle_dma(transfer);
              },
              MemAction::GpuGp0(value) => self.gpu.write_to_gp0(value),
              MemAction::GpuGp1(value) => self.gpu.write_to_gp1(value),
              MemAction::CDCmd(value) => {
                self.cd.send_command(value);
              },
              MemAction::CDParam(value) => {
                self.cd.send_parameter(value);
              },
              MemAction::Interrupt(irq) => {
                self.cop0.request_interrupt(irq);
              },
              MemAction::None => {
              },
            };
  }
  pub fn write_half(&mut self, address: Register, value: Register) {
    self.overwritten.insert(Console::physical(address));
    match self.memory.write_half(address, value) {
              MemAction::DMA(transfer) => {
                self.handle_dma(transfer);
              },
              MemAction::GpuGp0(value) => self.gpu.write_to_gp0(value),
              MemAction::GpuGp1(value) => self.gpu.write_to_gp1(value),
              MemAction::CDCmd(value) => {
                self.cd.send_command(value);
              },
              MemAction::CDParam(value) => {
                self.cd.send_parameter(value);
              },
              MemAction::Interrupt(irq) => {
                self.cop0.request_interrupt(irq);
              },
              MemAction::None => {
              },
            };
  }
  pub fn write_word(&mut self, address: Register, value: Register) {
    self.overwritten.insert(Console::physical(address));
            match self.memory.write_word(address, value) {
              MemAction::DMA(transfer) => {
                self.handle_dma(transfer);
              },
              MemAction::GpuGp0(value) => self.gpu.write_to_gp0(value),
              MemAction::GpuGp1(value) => self.gpu.write_to_gp1(value),
              MemAction::CDCmd(value) => {
                self.cd.send_command(value);
              },
              MemAction::CDParam(value) => {
                self.cd.send_parameter(value);
              },
              MemAction::Interrupt(irq) => {
                self.cop0.request_interrupt(irq);
              },
              MemAction::None => {
              },
            };
  }
  pub fn physical(address: Register) -> Register {
    const PHYS_MASK: [u32; 8] = [0xffff_ffff, 0xffff_ffff, 0xffff_ffff, 0xffff_ffff,
                                 0x7fff_ffff, 0x1fff_ffff, 0xffff_ffff, 0xffff_ffff];
    let idx = address.upper_bits(3) as usize;
    address & PHYS_MASK[idx]
  }

}

