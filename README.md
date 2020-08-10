# Pineapple

Pineapple is a full stack optimizing compiler and register-based virtual machine built in Rust for my own programming language and is a work in progress.

Pineapple compiles down to Pineapple Bytecode which is a form of pseduo-assembly (though I may make it compile down to ARM64 at some point), and uses its own virtual machine to execute the bytecode.

Instead of using LLVM or other IRs, Pineapple is my attempt to write all of these the phases of a compiler from scratch, including virtual machine that will run the Pineapple Bytecode.

Pineapple build's off of my previous compiler project [Mango](https://github.com/Tamiyo/Mango), a stack based virtual machine also written in  Rust.

### Feature Checklist
_This includes support for, but not necessarily the implementation of sed features. Will make a Trello Board soon that will be more accurate._ 
- [x] Basic Arithmetic / Conditionals
- [x] Variable Assignment
- [x] Print Function
- [x] If / Elif / Else Statement
- [x] While Loop
- [ ] For Loop
- [ ] Functions
- [ ] Closures
- [ ] Garbage Collector
- [ ] Classes
- [ ] Builtin Arrays
- [ ] Builtin Dictionaries
- [ ] Inheritence / Instancing

# Example
_Instead of using cargo you can use the precompiled pineapple executable, though it may not be up to date._

### cargo run test.pi 
Runs test.pi and outputs the result (if any).

### cargo run test.pi -d 
Runs test.pi in debug mode, outputs the result (if any), and outputs a bunch of debug compiler garbage.

### cargo run test.pi -d -o OR cargo run test.pi -do
Runs test.pi in debug and optimization mode, which will run compiler optimizations, outputs a bunch of debug compiler garbage, and outputs the result (if any).
