#![feature(asm)]
use std::io;
use memmap::{Mmap, MmapMut};

fn main() -> io::Result<()> {
  let mut mmap = MmapMut::map_anon(4096)?;
  mmap[0] = 0x48;
  mmap[1] = 0x8b;
  mmap[2] = 0xc7;
  mmap[3] = 0xc3;
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
