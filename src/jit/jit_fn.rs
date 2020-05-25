use memmap::Mmap;

pub struct JIT_Fn {
  mmap: Mmap,
  pub name: u64,
}

impl JIT_Fn {
  pub fn new(mmap: Mmap, name: u64) -> Self {
    JIT_Fn {
      mmap,
      name,
    }
  }
}
