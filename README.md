# Guayaba - A PlayStation 1 Emulator
Guayaba is a PS1 emulator started as a way for me to understand a relatively simple MIPS-based system before moving on to emulating more [modern consoles](https://github.com/ayrtonm/gecko). Recently it's evolved into a way to test various interpreters and just-in-time (JIT) compilers and currently includes:

 - a standard interpreter
 - a caching interpreter that compiles PS1 MIPS code to a sequence of closures
 - a JIT that compiles to x86-64 assembly (WIP)

Note that the caching interpreter is effectively a very inefficient, high-level JIT compiler. While there is no expectation that the caching interpreter will be any faster than the standard, it's a good way to compare the cost of cache invalidation and other JIT overhead with the potential gains of using assembly in a real JIT. Cache invalidation in particular is currently a very expensive process which highlights the importance of comparing the caching interpreter to the standard.

## Usage
    guayaba [OPTION...] -b BIOS -i INFILE
    
      -h  --help                 print this message
      -c  --cache                use the caching interpreter
      -j  --jit                  use the x86-64 JIT
      -o  --optimize             enable optimizations in the caching interpreter
      -b  --bios BIOS            specify BIOS file
      -i  --input INFILE         specify input file
      -l  --log                  print logging info to stdout
      -g  --gpu                  print gpu-specifiy logging info to stdout
      -s  --size WIDTHxHEIGHT    specify window size
      -n  --steps n              execute at least n opcodes then quit

## Useful references
### PS1 Documentation
This project is primarily based on the [rustation guide](https://svkt.org/~simias/guide.pdf) and the [No$ specs](http://problemkaputt.de/psx-spx.htm).
### JIT compilation
JIT compilation, also known as dynamic recompilation, is common enough that there's tons of stuff explaining the technique. The low-level details of implementing a JIT that recompiles to native machine code (i.e. the x64 JIT) are a bit more esoteric, but [this tutorial](https://github.com/spencertipping/jit-tutorial) was a good starting point.
