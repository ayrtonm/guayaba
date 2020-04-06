pub fn read_byte_from_array(arr: &[u8], address: u32) -> u32 {
  let address = address as usize;
  arr[address] as u32
}
pub fn read_half_from_array(arr: &[u8], address: u32) -> u32 {
  let address = address as usize;
  arr[address] as u32 |
  (arr[address + 1] as u32) << 8
}
pub fn read_word_from_array(arr: &[u8], address: u32) -> u32 {
  let address = address as usize;
  arr[address] as u32 |
  (arr[address + 1] as u32) << 8 |
  (arr[address + 2] as u32) << 16 |
  (arr[address + 3] as u32) << 24
}

pub fn write_byte_to_array(arr: &mut [u8], address: u32, value: u32) {
  let address = address as usize;
  arr[address] = (value & 0x0000_00ff) as u8;
}
pub fn write_half_to_array(arr: &mut [u8], address: u32, value: u32) {
  let address = address as usize;
  arr[address] = (value & 0x0000_00ff) as u8;
  arr[address + 1] = ((value >> 8) & 0x0000_00ff) as u8;
}
pub fn write_word_to_array(arr: &mut [u8], address: u32, value: u32) {
  let address = address as usize;
  arr[address] = (value & 0x0000_00ff) as u8;
  arr[address + 1] = ((value >> 8) & 0x0000_00ff) as u8;
  arr[address + 2] = ((value >> 16) & 0x0000_00ff) as u8;
  arr[address + 3] = ((value >> 24) & 0x0000_00ff) as u8;
}
