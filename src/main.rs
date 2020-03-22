use std::io;
use std::env;

mod common;
mod interpreter;
mod r3000;
mod register;
mod memory;
mod cd;

use interpreter::Interpreter;

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
  println!("rps [OPTION...] -b BIOS -i INFILE");
  println!("Rust Playstation Emulator");
  println!("");
  println!("  -h, --help");
  println!("  -i, --input INFILE");
  println!("  -b, --bios BIOS");
  println!("");
}

fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();

  let bios_flags = ["-b", "--bios"];
  let infile_flags = ["-i", "--input"];
  let help_flags = ["-h", "--help"];

  let bios = get_arg(&args, &bios_flags);
  let infile = get_arg(&args, &infile_flags);
  let help = check_flag(&args, &help_flags);

  if help {
    print_help();
  } else {
    match bios {
      Some(bios_filename) => {
        let mut vm = Interpreter::new(bios_filename, infile)?;
        vm.run();
      },
      None => {
        print_help();
      },
    }
  }
  Ok(())
}
