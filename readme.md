# Pineapple

### Overview
Pineapple is a fullstack optimizing compiler built for my own programming language. Pineapple is my attempt to build a compiler completely from scratch; without the use of some of the more popular ir frameworks like LLVM. This project is meant to serve as a personal research project to help me understand the innerworkings of compilers and language design, meaning that it's probably not going to have the most efficient / beautiful codebase on the daily but I'll be sure to try to keep it readable in case anyone is interesting in looking at the internals.

Pineapple compiles to Pineapple Bytecode that resembles assembly architectures. This bytecode gets run by a virtual machine, also built from scratch. While the virtual machine is included to run the code, the focus of Pineapple is on the compiler / optimizations.

### Features
- The compiler takes advantage of SSA (Static Single Assignment) form, which gives it access to some juicy performance optimizations.

- The language is statically typed.

- The language has many primitive types: (
    <br>&nbsp;&nbsp;&nbsp;&nbsp;*UInt8*, *UInt16*, *UInt32*, *UInt64*, *UInt*, *UInt128*,
    <br>&nbsp;&nbsp;&nbsp;&nbsp;*Int8*, *Int16*, *Int32*, *Int64*, *Int*, *Int128*,
    <br>&nbsp;&nbsp;&nbsp;&nbsp;*Float32*, *Float64*
    <br>&nbsp;&nbsp;&nbsp;&nbsp;*Bool*,
    <br>&nbsp;&nbsp;&nbsp;&nbsp;*String*
<br>)

- The compiler supports type casting.
  
- The compiler can compile:
    - If statements
    - While statements
    - Variable assignment
    - Arithmetic expressions
    - Function definitions and invocations
    - Recursive functions calls
    - Typesafe

### Improvements
- *Please note that at any give time there can be many problems with different portions of the compiler. I will try to remedy them asap when working on it.*
- Further improvements besides general coding practices would be to improve the performance of the VM. However, as this is not the goal of the project, this probably won't happen for a while.
- There are still many things that I plan on adding to this project such as Arrays, Objects, Classes, HashMaps, Builtin Functions, Imports.

### Running the Project
I won't be including a release schedule anytime soon. If you want to run the project you'll have to have Rust installed on your machine to run the project.

The CLI accepts multiple arguments described below.
- -d : Runs in debug mode. This will spit out lots of compiler garbage that I use to debug.

- -p : Runs in profiling mode. This will keep track of the execution time of different parts of the compiler and spit out the result.

- -o : Runs in optimization mode. Will attempt to optimize the code as much as possible with the optimizations that I have written.

```
cargo run {FILE_NAME} {ARGS}
```

### Samples
#### Recursive Fibonacci
```
#fibo(n: int): int {
    if (n <= 1) {
        return n;
    } else {
        return fibo(n - 1) + fibo(n - 2);
    }
}

#main() {
    print(fibo(9) as float64);
}
```
```
>> 34.0
```

#### Function with Casting
```
#add(a: int8, b: uint8): float64 {
    return ((a as uint8) + b) as float64;
}

#main() {
    a: int8 = 4;
    b: uint8 = 7;
    print(add(a, b));
}
```
```
>> 11.0
```
