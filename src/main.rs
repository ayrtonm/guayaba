use std::io;
use std::env;
use interpreter::Interpreter;

mod common;
mod interpreter;
mod r3000;
mod cop0;
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

const HELP_FLAGS: [&str;2] = ["-h", "--help"];
const BIOS_FLAGS: [&str;2] = ["-b", "--bios"];
const INFILE_FLAGS: [&str;2] = ["-i", "--input"];
const TEST_FLAGS: [&str;2] = ["-t", "--test"];
const ALL_FLAGS: [([&str;2],Option<&str>);4] = [(HELP_FLAGS, None),
                                                (BIOS_FLAGS, Some("BIOS")),
                                                (INFILE_FLAGS, Some("INFILE")),
                                                (TEST_FLAGS, Some("n"))];

fn print_help() {
  println!("rspsx [OPTION...] -b BIOS -i INFILE");
  println!("");
  for flags in &ALL_FLAGS {
    for f in &flags.0 {
      print!("  {}", f.to_string());
    }
    flags.1.map(|example| print!(" {}", example.to_string()));
    println!("");
  }
  println!("");
}

fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let bios = get_arg(&args, &BIOS_FLAGS);
  let infile = get_arg(&args, &INFILE_FLAGS);
  let help = check_flag(&args, &HELP_FLAGS);
  let test = get_arg(&args, &TEST_FLAGS).map(|test| test.parse::<u32>().unwrap());

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
