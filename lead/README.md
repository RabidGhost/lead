## Getting Started
1. You'll need Rust and its tools to use this project, you can install them via [rustup](https://rustup.rs).
2. Clone the repo
   ```sh
   git clone https://github.com/RabidGhost/lead.git
   ```
3. Run the example project with
   ```sh
   cargo run -- run example.ed
   ```
4. Get started writing code

### Writing Code

Variables must be declared the fisrt time they are used. You can declare a variable with
```
let foo := 42;
```
Once variables are declared, they are mutable.
```
foo := foo * 12;
```
To print a variable, you use the `yield` keyword to yield the value from the virtual machine.
```
yield (34 + foo);
```
```
> 538
```

You can conditionally execute code with `if`, for example.
```
let bar := 12;
let foo := 4;
yield foo;
if ((foo + bar) - 6) <= 10 {
	yield 7;
}
```
```
> 4
> 7
```

### Building to AIR
Code can also be build to an Assembly Intermediate Representation (AIR). You can build to AIR via the `build` command, for example `cargo run -- build example.ed` builds to
```
*example AIR here*
```
