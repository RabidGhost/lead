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
1. Get started writing code

### Writing Code

#### Variables
Variables must be declared the before they are used. You can declare a variable with
```
let foo := 42;
```
Once variables are declared, they are mutable.
```
foo := foo * 12;
```
#### Printing
To print a variable, you use the `yield` keyword to yield the value from the virtual machine.
```
yield (34 + foo);
```
```
> 538
```
#### Conditional Execution
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

#### Arrays
You can initialise arrays with a familiar syntax.
```
let arr := [1, 8, 12 * 2];
```
Arrays are statically sized at compile time. To index an array, use square brackets. Array indexing starts at zero.
```
yield arr[0];
```
```sh
> 1
```
Directly yielding an array will display the memory address of the array.


### How to integrate it with WASM
1. If you do not already have `wasm-pack` installed, run
```
cargo install wasm-pack
```

2. From `lead-w`, run
```
wasm-pack build --target web
```