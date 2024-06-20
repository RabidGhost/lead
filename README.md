## What is Lead?
Lead is a C style pseudo language for visualising compilatition of simple high level langauge constructs into assembly. `leadc` can currently compile to a low level intermediate format called AIR. You can view this output from the compiler using the `build` command, or run it in the included virtual machine with `run`. For more information about building and viewing the compiler workflow, see the section Lexing and Parsing. A GUI for visualising compilation is in progress.


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

## Lexing and Parsing
`leadc` can also provide outputs of its internal structures during the compilation process. This is provided in the way of the `lex` and `parse` commands, that display the processed tokens and syntax tree respectiveley.
