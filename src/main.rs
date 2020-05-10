use std::io;
use std::env;
use std::convert::TryInto;
use interpreter::Interpreter;
use jit::JIT;
use runnable::Runnable;

mod runnable;
mod common;
mod interpreter;
mod jit;
mod r3000;
mod cop0;
mod register;
mod memory;
mod cd;
mod dma;
mod gte;
mod gpu;
mod screen;

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

const DEFAULT_X: u32 = 1024;
const DEFAULT_Y: u32 = 512;
const DEFAULT_RESOLUTION: [u32; 2] = [DEFAULT_X, DEFAULT_Y];
const HELP_FLAGS: [&str;2] = ["-h", "--help"];
const JIT_FLAGS: [&str;2] = ["-j", "--jit"];
const BIOS_FLAGS: [&str;2] = ["-b", "--bios"];
const INFILE_FLAGS: [&str;2] = ["-i", "--input"];
const STEPS_FLAGS: [&str;2] = ["-n", "--steps"];
const LOG_FLAGS: [&str;2] = ["-l", "--log"];
const GPULOG_FLAGS: [&str;2] = ["-g", "--gpu"];
const RESOLUTION_FLAGS: [&str;2] = ["-s", "--size"];
const ALL_FLAGS: [([&str;2],Option<&str>);8] = [(HELP_FLAGS, None),
                                                (JIT_FLAGS, None),
                                                (BIOS_FLAGS, Some("BIOS")),
                                                (INFILE_FLAGS, Some("INFILE")),
                                                (LOG_FLAGS, None),
                                                (GPULOG_FLAGS, None),
                                                (RESOLUTION_FLAGS, Some("WIDTHxHEIGHT")),
                                                (STEPS_FLAGS, Some("n"))];

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
  let jit = check_flag(&args, &JIT_FLAGS);
  let steps = get_arg(&args, &STEPS_FLAGS).map(|steps| steps.parse::<u32>().ok())
                                          .flatten();
  let logging = check_flag(&args, &LOG_FLAGS);
  let gpu_logging = check_flag(&args, &GPULOG_FLAGS);
  let [wx, wy] = get_arg(&args, &RESOLUTION_FLAGS).map_or(DEFAULT_RESOLUTION,
    |resolution| {
      (*resolution.split("x")
                  .take(2)
                  .enumerate()
                  .map(|(i, x)|
                         x.parse::<u32>()
                          .unwrap_or(DEFAULT_RESOLUTION[i]))
                  .collect::<Vec<u32>>()
                  .into_boxed_slice()).try_into().unwrap_or(DEFAULT_RESOLUTION)
    }
  );

  if help {
    print_help();
  } else {
    match bios {
      Some(bios_filename) => {
        if !jit {
          Interpreter::new(bios_filename, infile, gpu_logging, wx, wy)?.run(steps, logging);
        } else {
          JIT::new(bios_filename, infile, gpu_logging, wx, wy)?.run(steps, logging);
        }
      },
      None => {
        print_help();
      },
    }
  }
  Ok(())
}
