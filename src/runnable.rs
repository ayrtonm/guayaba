pub trait Runnable {
  fn run(&mut self, n: Option<u32>, logging: bool);
}
