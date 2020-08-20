# Pineapple

Pineapple is a full stack optimizing compiler and register-based virtual machine built in Rust for my own programming language and is a work in progress.

Pineapple compiles down to Pineapple Bytecode which is a form of pseduo-assembly (though I may make it compile down to ARM64 at some point), and uses its own virtual machine to execute the bytecode.

Instead of using LLVM or other IRs, Pineapple is my attempt to write all of these the phases of a compiler from scratch, including virtual machine that will run the Pineapple Bytecode.

Pineapple build's off of my previous compiler project [Mango](https://github.com/Tamiyo/Mango), a stack based virtual machine also written in  Rust.

# Project Restructuring
At this point in time the compiler doesn't have a stable release state and it is subject to change at any time. I'm constantly refactoring the compiler to improve code quality and performance.