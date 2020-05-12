# RSX - A PlayStation 1 Emulator
RSX is a PS1 emulator started as a way for me to understand a relatively "simple" MIPS-based system before moving on to emulating more [modern consoles](https://github.com/ayrtonm/gecko). Recently it's evolved into a way to test various interpreters and just-in-time (JIT) compilers and currently includes:

 - a standard interpreter
 - a dummy JIT that "compiles" PS1 code to a sequence of closures

While there is no expectation that the dummy JIT will be any faster than the interpreter, it's a good way to compare the cost of cache invalidation and other JIT overhead with the potential gains of using assembly in a real JIT. Cache invalidation in particular is currently a very expensive process which highlights the importance of comparing the dummy JIT to the interpreter. There are also plans for the following:

 - a JIT that compiles to web-assembly bytecode which is then compiled to native machine code
 - a JIT that compiles to x86 assembly

## Useful references
This project is primarily based on the [rustation guide](https://svkt.org/~simias/guide.pdf) and the [No$ specs](http://problemkaputt.de/psx-spx.htm).
