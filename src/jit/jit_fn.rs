use memmap::Mmap;

pub struct JIT_Fn {
  mmap: Mmap,
  f: fn(),
}

impl JIT_Fn {
  pub fn new(mmap: Mmap, f: fn()) -> Self {
    JIT_Fn {
      mmap,
      f,
    }
  }
  pub fn execute(&self) {
    (self.f)()
  }
}
