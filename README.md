# Pineapple

Pineapple is a full stack optimizing compiler and register-based virtual machine built in Rust for my own programming language and is a work in progress.

Pineapple compiles down to Pineapple Bytecode which is a form of pseduo-assembly (though I may make it compile down to ARM64 at some point), and uses its own virtual machine to execute the bytecode.

Instead of using LLVM or other IRs, Pineapple is my attempt to write all of these the phases of a compiler from scratch, including virtual machine that will run the Pineapple Bytecode.

Pineapple build's off of my previous compiler project [Mango](https://github.com/Tamiyo/Mango), a stack based virtual machine also written in  Rust.

## Example
### cargo run test.pi 
Runs test.pi and outputs the result (if any).

### cargo run test.pi -d 
Runs test.pi in debug mode, which outprints a bunch of compiler garbage at the moment.

### cargo run test.pi -do
Runs test.pi in debug and optimization mode, which will run compiler optimizations on the code.