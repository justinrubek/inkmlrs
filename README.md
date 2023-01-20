# InkML

This is currently a work in progress Rust application. 
The goal is to create a renderer capable of displaying InkML files and to
expand it to be capable of accepting mouse/touch input to draw

![image](https://user-images.githubusercontent.com/25621857/213822190-852d8285-71e3-4766-98cf-dfaffc78d20f.png)


## Usage

Run the program and draw: `cargo run --bin inkmlrs`.
If you press "w", it will write the currently rendered inkml to the console.

You may also specify files to read/write from. See the help page: `cargo run --bin inkmlrs -- --help`
