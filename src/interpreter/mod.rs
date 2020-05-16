use std::io;
use std::time::Instant;
use crate::console::Console;

mod opcodes;

pub struct Interpreter {
  console: Console,
}

impl Interpreter {
  pub fn run(&mut self, n: Option<u32>, logging: bool) {
    let start_time = Instant::now();
    const refresh_rate: i64 = 550_000;
    let mut refresh_timer: i64 = refresh_rate;
    loop {
      if logging {
        println!("  ");
        println!("{} ----------------------", self.console.i);
      }
      self.step(logging);
      self.console.i += 1;
      n.map(|n| {
        if self.console.i == n {
          let end_time = Instant::now();
          panic!("Executed {} steps in {:?}", self.console.i, end_time - start_time);
        };
      });
      refresh_timer -= 1;
      if refresh_timer < 0 {
        self.console.screen.refresh_window();
        refresh_timer = refresh_rate;
      }
      if !self.console.handle_events() {
        return
      }
    }
  }
  pub fn new(bios_filename: &String, infile: Option<&String>, gpu_logging: bool,
             wx: u32, wy: u32) -> io::Result<Self> {
    let console = Console::new(bios_filename, infile, gpu_logging, wx, wy)?;
    Ok(Self {
      console,
    })
  }
  //this steps through the logic pertaining to the physical components of the playstation
  fn step(&mut self, logging: bool) {
    //get opcode from memory at program counter
    let op = self.console.resolve_memresponse(self.console.memory.read_word(self.console.r3000.pc()));
    if logging {
      println!("read opcode {:#x} from [{:#x}]", op, self.console.r3000.pc());
    }
    //the instruction following each jump is always executed before updating the pc
    //increment the program counter
    *self.console.r3000.pc_mut() = self.console.next_pc
                           .take()
                           .map_or_else(|| self.console.r3000.pc().wrapping_add(4),
                                        |next_pc| next_pc);
    self.console.next_pc = self.execute_opcode(op, logging);
    self.console.gpu.exec_next_gp0_command().map(|object| self.console.screen.draw(object));
    self.console.cd.exec_command();
  }
}
