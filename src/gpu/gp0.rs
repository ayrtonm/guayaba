use crate::gpu::GPU;
use crate::gpu::Command;
use crate::register::Register;
use crate::register::BitManipulation;
use crate::screen::Drawable;

impl GPU {
  fn object_within_limits(xpos: &Vec<i16>, ypos: &Vec<i16>) -> bool {
    let xmax = 1023;
    let ymax = 511;
    let xdiff = xpos.iter().max().unwrap() - xpos.iter().min().unwrap();
    let ydiff = ypos.iter().max().unwrap() - ypos.iter().min().unwrap();
    xdiff < xmax && ydiff < ymax
  }
  fn zip_positions(xpos: Vec<i16>, ypos: Vec<i16>) -> Vec<i16> {
    xpos.into_iter()
        .zip(ypos.into_iter())
        .map(|(a,b)| vec![a,b])
        .flatten()
        .collect()
  }
  pub fn exec_next_gp0_command(&mut self) -> Option<Drawable> {
    let command = self.command_buffer.pop_front();
    match command {
      Some(command) => {
        match command.id() {
          0x00 => {
          },
          0x01 => {
          },
          0x04..=0x1e | 0xe0 | 0xe7..=0xef => {
          },
          0x28 => {
            if self.logging {
              println!("rendered an opaque monochrome four-point polygon");
            }
            let xpos = command.get_xpos_consecutive();
            let ypos = command.get_ypos_consecutive();
            if GPU::object_within_limits(&xpos, &ypos) {
              let positions = GPU::zip_positions(xpos, ypos);
              let monochrome = command.get_monochrome();
              return Some(Drawable::new(positions, monochrome));
            }
          },
          0x2c => {
            let xpos = command.get_xpos_every_other();
            let ypos = command.get_ypos_every_other();
            if GPU::object_within_limits(&xpos, &ypos) {
              let positions = GPU::zip_positions(xpos, ypos);
              let cols = command.get_monochrome();
              return Some(Drawable::new(positions, cols));
            }
          },
          0x30 => {
            let xpos = command.get_xpos_every_other();
            let ypos = command.get_ypos_every_other();
            if GPU::object_within_limits(&xpos, &ypos) {
              let positions = GPU::zip_positions(xpos, ypos);
              let cols = command.get_colors();
              return Some(Drawable::new(positions, cols));
            }
          },
          0x38 => {
            if self.logging {
              println!("rendered an shaded four-point polygon");
            }
            let xpos = command.get_xpos_every_other();
            let ypos = command.get_ypos_every_other();
            if GPU::object_within_limits(&xpos, &ypos) {
              let positions = GPU::zip_positions(xpos, ypos);
              let cols = command.get_colors();
              return Some(Drawable::new(positions, cols));
            }
          },
          0xa0 => {
            if self.logging {
              println!("copy rectangle to VRAM");
            }
          },
          0xc0 => {
            if self.logging {
              println!("copy rectangle from VRAM");
            }
          },
          0xe1 => {
            let mask = 0x0000_83ff;
            let command = command.idx(0) & mask;
            self.gpustat.as_mut().clear_mask(mask).set_mask(command);
          },
          0xe2 => {
            let command = command.idx(0);
            self.texture_mask_x = command.lowest_bits(5);
            self.texture_mask_y = (command >> 5).lowest_bits(5);
            self.texture_offset_x = (command >> 10).lowest_bits(5);
            self.texture_offset_y = (command >> 15).lowest_bits(5);
          },
          0xe3 => {
            let command = command.idx(0);
            self.drawing_min_x = command.lowest_bits(10);
            self.drawing_min_y = (command >> 10).lowest_bits(9);
          },
          0xe4 => {
            let command = command.idx(0);
            self.drawing_max_x = command.lowest_bits(10);
            self.drawing_max_y = (command >> 10).lowest_bits(9);
          },
          0xe5 => {
            let command = command.idx(0);
            self.drawing_offset_x = command.lowest_bits(11);
            self.drawing_offset_y = (command >> 11).lowest_bits(11);
          },
          0xe6 => {
            let command = command.idx(0);
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
    None
  }
  pub fn write_to_gp0(&mut self, value: Register) {
    //println!("GP0 received {:#x}", value);
    let cmd = match self.waiting_for_parameters {
      true => {
        let mut cmd = self.partial_command.take()
                                          .expect("Expected a partial command in the GPU");
        cmd.append_parameters(value);
        cmd
      },
      false => {
        Command::new(value)
      },
    };
    self.try_push_command(cmd);
  }
  fn try_push_command(&mut self, cmd: Command) {
    match cmd.completed() {
      true => {
        if self.logging {
          println!("GP0 received command {:#x?}", cmd);
        }
        self.command_buffer.push_back(cmd);
        self.waiting_for_parameters = false;
      },
      false => {
        self.partial_command = Some(cmd);
        self.waiting_for_parameters = true;
      },
    }
  }
  fn filled_buffer(&self) -> usize {
    self.command_buffer.iter().fold(0, |acc, command| acc + command.num_words())
  }
}
