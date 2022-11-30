# WaveFunctionCollapse
Converts nodes and their restrictions into a collapsed node state based on the selected algorithm.

## Features

- Simple constructor inputs of nodes and their respective constraints
- Allows for either a full sequential search of all possible solutions or a random search for more heterogenious solutions
- Examples showing how different constraint problems can be solved via the different algorithms
- Returns if the wave function cannot be collapsed if the sequential search is used

## Usage

_Sudoku example_
```shell
cargo run --release --example sudoku
```

_Landscape example_
```shell
cargo run --release --example landscape
```
