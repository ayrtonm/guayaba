use crate::register::BitTwiddle;
use cd::CD;
use cop0::Cop0;
use cop0::Cop0Exception;
use gpu::GPU;
use gte::GTE;
use memory::{MemAction, MemResponse, Memory};
use r3000::R3000;
use screen::Screen;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::{HashSet, VecDeque};
use std::convert::TryInto;
use std::fs::{metadata, File};
use std::io;
use std::io::Read;

mod cd;
pub mod cop0;
mod dma;
mod gpu;
mod gte;
mod handle_dma;
mod memory;
pub mod r3000;
mod screen;

pub trait MaybeSet {
    fn maybe_set(self, value: u32) -> Option<Name>;
}

//different types of register names
//these are for improved readability when doing delayed register writes
#[derive(Debug, PartialEq)]
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
            MemAction::None => {},
        };
    };
}

macro_rules! handle_response {
    ($read:expr, $self:ident) => {
        match $read {
            MemResponse::Value(value) => value,
            MemResponse::GPUREAD => $self.gpu.gpuread(),
            MemResponse::GPUSTAT => $self.gpu.gpustat(),
            MemResponse::CDResponse => $self.cd.read_response(),
        }
    };
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

    pub fn new(
        bios_filename: &String, infile: Option<&String>, gpu_logging: bool, wx: u32, wy: u32,
    ) -> io::Result<Self> {
        let mut r3000 = R3000::new();
        let cop0: Cop0 = Default::default();
        let mut memory = Memory::new(bios_filename)?;

        infile.map(|name| {
            let mut file = File::open(name).expect("Unable to open input file");
            let filesize = metadata(name)
                .expect("Unable to get input file metadata")
                .len();
            if filesize % 0x800 != 0 {
                println!("Warning: PSEXE has an invalid filesize");
            }
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            let words = buf
                .chunks(4)
                .map(|c| {
                    u32::from_ne_bytes(c.try_into().expect("Couldn't turn 4-byte chunk into a u32"))
                })
                .collect::<Vec<u32>>();
            *r3000.pc_mut() = words[0x10 / 4];
            r3000.nth_reg_mut(28).maybe_set(words[0x14 / 4]);
            r3000
                .nth_reg_mut(29)
                .maybe_set(words[0x30 / 4] + words[0x34 / 4]);
            r3000
                .nth_reg_mut(30)
                .maybe_set(words[0x30 / 4] + words[0x34 / 4]);

            let dest_addr = words[0x18 / 4];
            let filesize = words[0x1C / 4] >> 2;
            for (n, word) in words[0x800 / 4..].iter().enumerate() {
                memory.write_word(dest_addr + (4 * n as u32), *word);
            }
        });

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
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        println!("Executed {} steps", self.i);
                        return false
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::S),
                        ..
                    } => {
                        println!("You pressed X");
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::D),
                        ..
                    } => {
                        println!("You pressed ◯");
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::A),
                        ..
                    } => {
                        println!("You pressed □");
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::W),
                        ..
                    } => {
                        println!("You pressed △");
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::K),
                        ..
                    } => {
                        println!("You pressed down");
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::L),
                        ..
                    } => {
                        println!("You pressed right");
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::J),
                        ..
                    } => {
                        println!("You pressed left");
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::I),
                        ..
                    } => {
                        println!("You pressed up");
                    },
                    Event::Quit { .. } => panic!(""),
                    _ => {},
                }
            }
        }
        true
    }

    pub extern "C" fn read_byte_sign_extended(&mut self, address: u32) -> u32 {
        handle_response!(self.memory.read_byte_sign_extended(address), self)
    }

    pub extern "C" fn read_half_sign_extended(&mut self, address: u32) -> u32 {
        handle_response!(self.memory.read_half_sign_extended(address), self)
    }

    pub extern "C" fn read_byte(&mut self, address: u32) -> u32 {
        handle_response!(self.memory.read_byte(address), self)
    }

    pub extern "C" fn read_half(&mut self, address: u32) -> u32 {
        handle_response!(self.memory.read_half(address), self)
    }

    pub extern "C" fn read_word(&mut self, address: u32) -> u32 {
        handle_response!(self.memory.read_word(address), self)
    }

    pub extern "C" fn write_byte(&mut self, address: u32, value: u32) {
        self.overwritten.insert(Console::physical(address));
        handle_action!(self.memory.write_byte(address, value), self);
    }

    pub extern "C" fn write_half(&mut self, address: u32, value: u32) {
        self.overwritten.insert(Console::physical(address));
        handle_action!(self.memory.write_half(address, value), self);
    }

    pub extern "C" fn write_word(&mut self, address: u32, value: u32) {
        self.overwritten.insert(Console::physical(address));
        handle_action!(self.memory.write_word(address, value), self);
    }

    pub extern "C" fn print_value(value: u32) {
        println!("{:#x?}", value);
    }

    pub fn physical(address: u32) -> u32 {
        const PHYS_MASK: [u32; 8] = [
            0xffff_ffff,
            0xffff_ffff,
            0xffff_ffff,
            0xffff_ffff,
            0x7fff_ffff,
            0x1fff_ffff,
            0xffff_ffff,
            0xffff_ffff,
        ];
        let idx = address.upper_bits(3) as usize;
        address & PHYS_MASK[idx]
    }

    pub fn generate_exception(&mut self, kind: Cop0Exception, current_pc: u32) -> u32 {
        self.cop0.generate_exception(kind, current_pc)
    }
}
