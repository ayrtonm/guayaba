use crate::gpu::GPU;
use crate::gpu::Command;
use crate::register::Register;
use crate::register::BitManipulation;

impl GPU {
  pub fn exec_next_gp0_command(&mut self) {
    let command = self.command_buffer.pop_front();
    match command {
      Some(command) => {
        match command.id() {
          0x00 => {
          },
          0x01 => {
            self.gpustat.as_mut().set(24);
          },
          0x04..=0x1e | 0xe0 | 0xe7..=0xef => {
          },
          0x28 => {
            println!("rendered an opaque monochrome four-point polygon");
          },
          0xa0 => {
            println!("copy rectangle to VRAM");
          },
          0xc0 => {
            println!("copy rectangle from VRAM");
          },
          0xe1 => {
            let mask = 0x0000_83ff;
            let command = command.as_ref(0) & mask;
            self.gpustat.as_mut().clear_mask(mask).set_mask(command);
          },
          0xe2 => {
            let command = command.as_ref(0);
            self.texture_mask_x = command.lowest_bits(5);
            self.texture_mask_y = (command >> 5).lowest_bits(5);
            self.texture_offset_x = (command >> 10).lowest_bits(5);
            self.texture_offset_y = (command >> 15).lowest_bits(5);
          },
          0xe3 => {
            let command = command.as_ref(0);
            self.drawing_min_x = command.lowest_bits(10);
            self.drawing_min_y = (command >> 10).lowest_bits(9);
          },
          0xe4 => {
            let command = command.as_ref(0);
            self.drawing_max_x = command.lowest_bits(10);
            self.drawing_max_y = (command >> 10).lowest_bits(9);
          },
          0xe5 => {
            let command = command.as_ref(0);
            self.drawing_offset_x = command.lowest_bits(11);
            self.drawing_offset_y = (command >> 11).lowest_bits(11);
          },
          0xe6 => {
            let command = command.as_ref(0);
            let mask = command.lowest_bits(2) << 11;
            self.gpustat.as_mut().clear(11).clear(12).set_mask(mask);
          },
          _ => {
            todo!("implement this GP0 command {:#x}", command.id());
          },
        }
      },
      None => {
      },
    }
  }
  pub fn write_to_gp0(&mut self, value: Register) {
    //println!("GP0 received {:#x}", value);
    if !self.waiting_for_parameters {
      let cmd = Command::new(value);
      if cmd.completed() {
        println!("GP0 received command {:#x?}", cmd);
        self.command_buffer.push_back(cmd);
      } else {
        self.partial_command = Some(cmd);
        self.waiting_for_parameters = true;
      }
    } else {
      let mut cmd = self.partial_command.take().expect("Expected a partial command in the GPU");
      cmd.append_parameters(value);
      if cmd.completed() {
        println!("GP0 received command {:#x?}", cmd);
        self.command_buffer.push_back(cmd);
        self.waiting_for_parameters = false;
      } else {
        self.partial_command = Some(cmd);
      }
    }
  }
}
