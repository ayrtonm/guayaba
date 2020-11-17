#![feature(llvm_asm)]
use interpreter::Interpreter;
use jit::caching_interpreter::CachingInterpreter;
use jit::x64_jit::X64JIT;
use std::convert::TryInto;
use std::env;
use std::io;

mod common;
mod console;
mod interpreter;
mod jit;
mod register;

fn get_arg<'a>(args: &'a Vec<String>, flags: &[&str]) -> Option<&'a String> {
    args.iter()
        .position(|s| flags.iter().any(|t| *t == *s))
        .map(|idx| args.iter().nth(idx + 1))
        .flatten()
}

fn check_flag(args: &Vec<String>, flags: &[&str]) -> bool {
    args.iter().any(|s| flags.iter().any(|t| *t == *s))
}

const DEFAULT_X: u32 = 1024;
const DEFAULT_Y: u32 = 512;
const DEFAULT_RESOLUTION: [u32; 2] = [DEFAULT_X, DEFAULT_Y];
//print help
const HELP_FLAGS: [&str; 2] = ["-h", "--help"];
//use the caching interpreter
const CACHE_FLAGS: [&str; 2] = ["-c", "--cache"];
//use the x64 JIT
const JIT_FLAGS: [&str; 2] = ["-j", "--jit"];
//optimize the caching interpreter or x64 JIT
const OPT_FLAGS: [&str; 2] = ["-o", "--optimize"];
//specify the BIOS
const BIOS_FLAGS: [&str; 2] = ["-b", "--bios"];
//specify the input file
const INFILE_FLAGS: [&str; 2] = ["-i", "--input"];
//run for a given number of steps
const STEPS_FLAGS: [&str; 2] = ["-n", "--steps"];
//print logging info
const LOG_FLAGS: [&str; 2] = ["-l", "--log"];
//print GPU logging info
const GPULOG_FLAGS: [&str; 2] = ["-g", "--gpu"];
//set resolution
const RESOLUTION_FLAGS: [&str; 2] = ["-s", "--size"];
const ALL_FLAGS: [([&str; 2], Option<&str>); 10] = [
    (HELP_FLAGS, None),
    (CACHE_FLAGS, None),
    (JIT_FLAGS, None),
    (OPT_FLAGS, None),
    (BIOS_FLAGS, Some("BIOS")),
    (INFILE_FLAGS, Some("INFILE")),
    (LOG_FLAGS, None),
    (GPULOG_FLAGS, None),
    (RESOLUTION_FLAGS, Some("WIDTHxHEIGHT")),
    (STEPS_FLAGS, Some("n")),
];

fn print_help() {
    println!("guayaba [OPTION...] -b BIOS -i INFILE");
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
    let cache = check_flag(&args, &CACHE_FLAGS);
    let jit = check_flag(&args, &JIT_FLAGS);
    let opt = check_flag(&args, &OPT_FLAGS);
    let steps = get_arg(&args, &STEPS_FLAGS)
        .map(|steps| steps.parse::<u32>().ok())
        .flatten();
    let logging = check_flag(&args, &LOG_FLAGS);
    let gpu_logging = check_flag(&args, &GPULOG_FLAGS);
    let [wx, wy] = get_arg(&args, &RESOLUTION_FLAGS).map_or(DEFAULT_RESOLUTION, |resolution| {
        (*resolution
            .split("x")
            .take(2)
            .enumerate()
            .map(|(i, x)| x.parse::<u32>().unwrap_or(DEFAULT_RESOLUTION[i]))
            .collect::<Vec<u32>>()
            .into_boxed_slice())
        .try_into()
        .unwrap_or(DEFAULT_RESOLUTION)
    });

    //if the optimize flag was enabled without a JIT
    //or if two types of JIT are enabled, print help
    if help || (opt && !(cache || jit)) || (cache && jit) {
        print_help();
    } else {
        match bios {
            Some(bios_filename) => {
                if cache {
                    CachingInterpreter::new(bios_filename, infile, gpu_logging, wx, wy)?
                        .run(steps, opt, logging);
                } else if jit {
                    X64JIT::new(bios_filename, infile, gpu_logging, wx, wy)?
                        .run(steps, opt, logging)?;
                } else {
                    Interpreter::new(bios_filename, infile, gpu_logging, wx, wy)?
                        .run(steps, logging);
                }
            },
            None => {
                print_help();
            },
        }
    }
    Ok(())
}
