use crate::console::Console;

pub struct Stub(Box<dyn Fn(&mut Console) -> Option<u32>>);

impl Stub {
  pub fn new(op: u32) -> Self {
    Stub(Box::new(|vm| None))
  }
  pub fn execute(&self, console: &mut Console) -> Option<u32> {
    self.0(console)
  }
}
