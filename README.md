# InkML

This is currently a work in progress Rust application. 
The goal is to create a renderer capable of displaying InkML files and to
expand it to be capable of accepting mouse/touch input to draw

## Usage

Run the program and draw: `cargo run --bin inkmlrs`.
If you press "w", it will write the currently rendered inkml to the console.

You may also specify files to read/write from. See the help page: `cargo run --bin inkmlrs -- --help`
