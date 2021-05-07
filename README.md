# Mazetool

A tool to generate mazes of arbitrary size and a pathfinding algorithm to find
routes through the maze.

The design of the software is overcomplicated for the reason of just doing
this a Rust learning experience. Data structures for the maze are also not
the most efficient design.

## Environment setup

Source is available in git, `nitro.dy.fi:/data/git/playground.git`.
Install CMake, Rust and Cargo.

## Build

Building is as simple as running `cargo build` anywhere in the source tree.

## Running

Just do `cargo run -- generate 100 40`.
For some reason this uses a lot of stack space, more then expected for this
level of recursion.

`RUSTFLAGS="-C link-args=-zstack-size=16000000" cargo run -- generate 100 300 >foo`
can be used to increase the stack size.

## Testing

This project is not heavy on testing. Currently there aren't any tests implemented.
Some simple tests could be implemented just to try it out.

Run tests (if any) using `cargo test`.

## Documentation

Documentation is done along with the source code with _rustdoc_.

Generate documentation using `cargo doc`.
