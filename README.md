# WaveFunctionCollapse
Converts nodes and their constraints upon other nodes into a fully collapsed node state based on the selected algorithm. This can be used to solve any degree of complexity from dense node graphs to Sudoku to procedurally-generated game elements and more.

## Features

- Basic interface to make usage easy to try out
  - The graph does **not** need to be fully connected
  - Any missing constraints between two nodes imply that the former node, for that state, has no impact on the neighbor node
- Allows for tailoring the algorithm to the problem
  - A full sequential search of all possible solutions when it is known that very few, one, or no solutions are possible
    - Can determine if the wave function is not collapsable
  - A random search for more heterogenious solutions when many solutions are possible, but may never complete given certain circumstances
- Different probabilities per state per node can be suggested to allow for either faster results or different random results (based on the algorithm used)
- Examples showing how different constraint problems can be solved via the different algorithms
- The wave function can be saved and loaded from file

## Usage

To use this framework, you will first want to determine the following:
- What is the type of your node states?
  - An enum because there are only specific states?
  - A String because it may be unknown at compile time?
  - A u64 because it is a reference to an identifier in a database?
- What does your graph of nodes look like?
  - Is it a flat grid like a checkerboard?
  - Is it a 3D grid like a voxel game (ex: Minecraft)?
- What node states, for any specific node, would permit which other node states for its neighbor nodes?
  - Can direct neighbors of nodes not have the same state as the current node?
  - Are only certain states possible when the node is in this special state?

Once these are answered, you can construct the vector of nodes and the vector of node state collections that those nodes reference for their permissive relationships. Please examine a relevant example to see how to the construction of nodes and node state collections occurs.

## Examples

_Sudoku example_

This example demonstrates usage of a sequential wave function collapse algorithm.
```shell
cargo run --release --example sudoku
```

_Landscape example_

This example demonstrates usage of an accommodating wave function collapse algorithm.
```shell
cargo run --release --example landscape
```

_Sparse example_

This example demonstrates usage of an accommodating wave function collapse algorithm along with more sparse probabilities.
```shell
cargo run --release --example sparse
```

_Zebra puzzle example_

This example demonstrates usage of a sequential wave function collapse algorithm for word problems like the Zebra Puzzle.
```shell
cargo run --release --example zebra_puzzle
```

## Complex problems

_Shared conditions between nodes_

When you want to say that "when node 1 is in state A and node 2 is in state B then node 3 can only be in state C", you will end up needing to have multiple variations of the same node state such that our previously mentioned state "B" would be equivalent to "B when node 1 is A". This would permit you to specify for node 2 that when it is in the state "B when node 1 is A", then it will only permit node 3 to be in state C.

**Example coming soon**
