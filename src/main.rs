use std::io;
use std::env;
use interpreter::Interpreter;

mod common;
mod interpreter;
mod r3000;
mod register;
mod memory;
mod cd;

fn get_arg<'a>(args: &'a Vec<String>, flags: &[&str]) -> Option<&'a String> {
  args.iter()
      .position(|s| flags.iter().any(|t| *t == *s))
      .map(|idx| args.iter().nth(idx + 1))
      .flatten()
}

fn check_flag(args: &Vec<String>, flags: &[&str]) -> bool {
  args.iter()
      .any(|s| flags.iter().any(|t| *t == *s))
}

fn print_help() {
  println!("rspsx [OPTION...] -b BIOS -i INFILE");
  println!("");
  println!("  -h, --help");
  println!("  -i, --input INFILE");
  println!("  -b, --bios BIOS");
  println!("  -t, --test steps");
  println!("");
}

fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();

  let bios_flags = ["-b", "--bios"];
  let infile_flags = ["-i", "--input"];
  let help_flags = ["-h", "--help"];
  let test_flags = ["-t", "--test"];

  let bios = get_arg(&args, &bios_flags);
  let infile = get_arg(&args, &infile_flags);
  let help = check_flag(&args, &help_flags);
  let test = get_arg(&args, &test_flags).map(|test| test.parse::<u32>().unwrap());

  if help {
    print_help();
  } else {
    match bios {
      Some(bios_filename) => {
        Interpreter::new(bios_filename, infile)?.run(test);
      },
      None => {
        print_help();
      },
    }
  }
  Ok(())
}
