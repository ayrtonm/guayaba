#![feature(asm)]
use std::io;
use memmap::{Mmap, MmapMut};

fn main() -> io::Result<()> {
  let mut buffer = Vec::new();
  buffer.push(0x48);
  buffer.push(0x8b);
  buffer.push(0xc7);
  buffer.push(0xc3);
  let mut mmap = MmapMut::map_anon(buffer.len())?;
  mmap.copy_from_slice(&buffer);
  let mmap: Mmap = mmap.make_exec()?;
  let addr = mmap.as_ptr();
  unsafe {
    let f = std::mem::transmute::<*const u8, fn(u32) -> u32>(addr);
    for i in 0..=10 {
      println!("{}", f(i));
    }
  };
  Ok(())
}
