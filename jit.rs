fn main() {
  let x = 2;
  let functions: Vec<Box<dyn Fn(u32) -> u32>> = vec![
    Box::new(identity),
    Box::new(add_three),
    Box::new(mult_two)
  ];
  let source = vec![1, 2, 2, 1];
  let program = source.iter().map(|&n| &functions[n]);
  let input = 5;
  let output = program.fold(input, |x, f| f(x));

  let alt_func: Vec<Box<dyn Fn()>> = vec![
    Box::new(do_nothing_0),
    Box::new(do_nothing_1)
  ];
  let alt_source = vec![0,1,0];
  let alt_program = alt_source.iter().map(|&n| &alt_func[n]);
  let alt_output = alt_program.fold((), |_, f| f());
  println!("main {}", output);
}

fn identity(x: u32) -> u32 {
  println!("called identity");
  x
}
fn add_three(x: u32) -> u32 {
  println!("called add_three");
  x + 3
}

fn mult_two(x: u32) -> u32 {
  println!("called mult_two");
  x * 2
}

fn do_nothing_0() {
  println!("called do_nothing_0");
}

fn do_nothing_1() {
  println!("called do_nothing_1");
}
