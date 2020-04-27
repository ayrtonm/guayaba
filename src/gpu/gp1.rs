use crate::gpu::GPU;
use crate::register::Register;
use crate::register::BitManipulation;

impl GPU {
  pub fn write_to_gp1(&mut self, value: Register) {
    if self.logging {
      println!("GP1 received {:#x}", value);
    }
    let command = value >> 24;
    match command {
      0x00 => {
        self.command_buffer.clear();
        self.waiting_for_parameters = false;
        self.partial_command = None;
        *self.gpustat.as_mut() = 0x1480_2000;
      },
      0x01 => {
        self.command_buffer.clear();
        self.waiting_for_parameters = false;
        self.partial_command = None;
      },
      0x02 => {
        self.gpustat.as_mut().clear(24);
      },
      0x03 => {
        self.gpustat.as_mut().clear(23).set_mask(command.lowest_bits(1) << 23);
      },
      0x04 => {
        let mask = 0x6000_0000;
        let new_values = (value & 3) << 29;
        self.gpustat.as_mut().clear_mask(mask).set_mask(new_values);
      },
      0x05 => {
        self.display_x = *command.lowest_bits(10).clone().clear(0);
        self.display_y = (command >> 10).lowest_bits(9);
      },
      0x06 => {
        self.display_range_x1 = command.lowest_bits(12);
        self.display_range_x2 = (command >> 12).lowest_bits(12);
      },
      0x07 => {
        self.display_range_y1 = command.lowest_bits(10);
        self.display_range_y2 = (command >> 10).lowest_bits(10);
      },
      0x08 => {
        let mask = 0x003f_4000;
        let new_values = ((value & 0x3f) << 17) | (value & 0x40) << 16 | (value & 0x80) << 14;
        self.gpustat.as_mut().clear_mask(mask).set_mask(new_values);
      },
      _ => {
        todo!("implement this GP1 command {:#x}", command);
      },
    }
  }
}
