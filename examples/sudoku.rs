use std::{collections::HashMap, time::Instant};
use wave_function_collapse::wave_function::{
    Node,
    NodeStateCollection,
    WaveFunction, NodeStateProbability, collapsable_wave_function::{sequential_collapsable_wave_function::SequentialCollapsableWaveFunction, collapsable_wave_function::CollapsableWaveFunction}
};

/// This struct represents a Sudoku puzzle.
struct SudokuPuzzle {
    number_per_row_per_column: Vec<Vec<Option<u8>>>
}

impl SudokuPuzzle {
    fn print(&self) {
        let mut number_per_column_per_row: Vec<Vec<Option<u8>>> = Vec::new();

        for (x_index, number_per_row) in self.number_per_row_per_column.iter().enumerate() {
            for (y_index, number) in number_per_row.iter().enumerate() {
                if y_index == 0 {
                    number_per_column_per_row.push(Vec::new());
                }
                number_per_column_per_row.get_mut(x_index).unwrap().push(number.clone());
            }
        }

        for (_, number_per_column) in number_per_column_per_row.iter().enumerate() {
            println!("-------------------");
            print!("|");
            for (_, number_option) in number_per_column.iter().enumerate() {
                if let Some(number) = number_option {
                    print!("{number}");
                }
                else {
                    print!(" ");
                }
                print!("|");
            }
            println!("");
        }
        println!("-------------------");
    }
    fn get_solution(&self) -> Result<SudokuPuzzle, String> {
        // setup nodes and node state collections
        let mut node_id_per_y_per_x: HashMap<usize, HashMap<usize, String>> = HashMap::new();
        for (x_index, number_per_row) in self.number_per_row_per_column.iter().enumerate() {
            let mut node_id_per_y: HashMap<usize, String> = HashMap::new();
            for (y_index, _) in number_per_row.iter().enumerate() {
                let node_id = format!("node_{}_{}", x_index, y_index);
                node_id_per_y.insert(y_index, node_id);
            }
            node_id_per_y_per_x.insert(x_index, node_id_per_y);
        }

        let mut exclusive_node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        for number in 1u8..10 {
            let mut node_state_ids: Vec<String> = Vec::new();
            for other_number in 1u8..10 {
                if other_number != number {
                    node_state_ids.push(format!("state_{other_number}"));
                }
            }
            let node_state_collection = NodeStateCollection::new(
                format!("exclusive_{number}"),
                format!("state_{number}"),
                node_state_ids
            );
            exclusive_node_state_collections.push(node_state_collection);
        }

        // all other node states are possible expect for "number"
        let mut possible_node_state_collection_per_number: HashMap<u8, NodeStateCollection<String>> = HashMap::new();
        for number in 1u8..10 {
            let mut node_state_ids: Vec<String> = Vec::new();
            for state in 1u8..10 {
                if state != number {
                    node_state_ids.push(format!("state_{state}"));
                }
            }
            let node_state_collection = NodeStateCollection::new(
                format!("specific_{number}_possible"),
                format!("state_{number}"),
                node_state_ids
            );
            possible_node_state_collection_per_number.insert(number, node_state_collection);
        }

        // no node states are possible if "number"
        let mut impossible_node_state_collection_per_number: HashMap<u8, NodeStateCollection<String>> = HashMap::new();
        for number in 1u8..10 {
            let node_state_collection = NodeStateCollection::new(
                format!("specific_{number}_impossible"),
                format!("state_{number}"),
                vec![]
            );
            impossible_node_state_collection_per_number.insert(number, node_state_collection);
        }

        // when from node is "from number" only "destination state" is possible for neighbor
        let mut always_node_state_collection_per_to_number_per_from_number: HashMap<u8, HashMap<u8, NodeStateCollection<String>>> = HashMap::new();
        for from_number in 1u8..10 {
            let from_number_node_state_id = format!("state_{from_number}");
            let mut always_node_state_collection_per_to_number: HashMap<u8, NodeStateCollection<String>> = HashMap::new();
            for to_number in 1u8..10 {
                let to_number_node_state_id = format!("state_{to_number}");

                let node_state_collection = NodeStateCollection::new(
                    format!("from_{from_number}_to_{to_number}"),
                    from_number_node_state_id.clone(),
                    vec![to_number_node_state_id]
                );

                always_node_state_collection_per_to_number.insert(to_number, node_state_collection);
            }
            always_node_state_collection_per_to_number_per_from_number.insert(from_number, always_node_state_collection_per_to_number);
        }

        let mut node_state_collection_per_id: HashMap<String, NodeStateCollection<String>> = HashMap::new();
        let mut nodes: Vec<Node<String>> = Vec::new();
        for (from_x_index, from_number_per_row) in self.number_per_row_per_column.iter().enumerate() {
            for (from_y_index, from_number_option) in from_number_per_row.iter().enumerate() {
                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                for (to_x_index, to_number_per_row) in self.number_per_row_per_column.iter().enumerate() {
                    for (to_y_index, to_number_option) in to_number_per_row.iter().enumerate() {
                        if !(from_x_index == to_x_index && from_y_index == to_y_index) && 
                            (
                                from_x_index == to_x_index ||
                                from_y_index == to_y_index ||
                                ((from_x_index / 3) == (to_x_index / 3) && (from_y_index / 3) == (to_y_index / 3))
                            ) {
                            
                            //println!("Neighbors: ({from_x_index}, {from_y_index}) and ({to_x_index}, {to_y_index})");

                            let mut node_state_collection_ids: Vec<String> = Vec::new();
                            if let Some(from_number) = from_number_option {
                                // if the from node will already have a value
                                if let Some(to_number) = to_number_option {
                                    // and the to node will already have a value
                                    // then when "from" is in the required state, only permit "to"'s required state
                                    // else when "from" is in any other state, permit nothing
                                    
                                    for possible_from_number in 1u8..10 {
                                        let node_state_collection: &NodeStateCollection<String>;
                                        if possible_from_number == *from_number {
                                            node_state_collection = always_node_state_collection_per_to_number_per_from_number.get(&possible_from_number).unwrap().get(&to_number).unwrap();
                                        }
                                        else {
                                            node_state_collection = impossible_node_state_collection_per_number.get(&possible_from_number).unwrap();
                                        }
                                        //println!("When from is {possible_from_number} and to is {to_number} then {:?}", node_state_collection);
                                        node_state_collection_ids.push(node_state_collection.id.clone());
                                        node_state_collection_per_id.insert(node_state_collection.id.clone(), node_state_collection.clone());
                                    }
                                }
                                else {
                                    // and the "to" node is unknown
                                    // then when "from" is in the required state, permit all but that state
                                    // else when "from" is in any other state, permit nothing

                                    for possible_from_number in 1u8..10 {
                                        let node_state_collection: &NodeStateCollection<String>;
                                        if possible_from_number == *from_number {
                                            node_state_collection = possible_node_state_collection_per_number.get(&possible_from_number).unwrap();
                                        }
                                        else {
                                            node_state_collection = impossible_node_state_collection_per_number.get(&possible_from_number).unwrap();
                                        }
                                        //println!("When from is {possible_from_number} and to is unknown then {:?}", node_state_collection);
                                        node_state_collection_ids.push(node_state_collection.id.clone());
                                        node_state_collection_per_id.insert(node_state_collection.id.clone(), node_state_collection.clone());
                                    }
                                }
                            }
                            else {
                                // the "from" node is unknown
                                if let Some(to_number) = to_number_option {
                                    // and the "to" node is known
                                    // then for any possible value of "from" always require the "to" value
                                    for possible_from_number in 1u8..10 {
                                        let node_state_collection = always_node_state_collection_per_to_number_per_from_number.get(&possible_from_number).unwrap().get(to_number).unwrap();
                                        //println!("When from is {possible_from_number} and to is {to_number} then {:?}", node_state_collection);
                                        node_state_collection_ids.push(node_state_collection.id.clone());
                                        node_state_collection_per_id.insert(node_state_collection.id.clone(), node_state_collection.clone());
                                    }
                                }
                                else {
                                    // and the "to" node is always unknown
                                    // then for any possible value of "from" permit anything but that value for "to"
                                    for node_state_collection in exclusive_node_state_collections.iter() {
                                        //println!("When from is unknown and to is unknown then {:?}", node_state_collection);
                                        node_state_collection_ids.push(node_state_collection.id.clone());
                                        node_state_collection_per_id.insert(node_state_collection.id.clone(), node_state_collection.clone());
                                    }
                                }
                            }

                            let to_node_id = format!("node_{}_{}", to_x_index, to_y_index);
                            node_state_collection_ids_per_neighbor_node_id.insert(to_node_id, node_state_collection_ids);
                        }
                        else {
                            //println!("Not neighbors: ({from_x_index}, {from_y_index}) and ({to_x_index}, {to_y_index})");
                        }
                    }
                }
                let mut node_state_ids: Vec<String> = Vec::new();
                for number in 1u8..10 {
                    node_state_ids.push(format!("state_{number}"));
                }
                let node = Node::new(
                    node_id_per_y_per_x.get(&from_x_index).unwrap().get(&from_y_index).unwrap().clone(),
                    NodeStateProbability::get_equal_probability(&node_state_ids),
                    node_state_collection_ids_per_neighbor_node_id
                );
                nodes.push(node);
            }
        }

        let wave_function = WaveFunction::new(nodes, node_state_collection_per_id.values().cloned().collect());
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Ok(collapsed_wave_function) = collapsed_wave_function_result {
            let mut state_per_row_per_column: Vec<Vec<Option<u8>>> = Vec::new();
            for index in 1u8..10 {
                state_per_row_per_column.push(Vec::new());
                for _ in 1u8..10 {
                    state_per_row_per_column[(index as usize) - 1].push(None);
                }
            }
            for (node, node_state) in collapsed_wave_function.node_state_per_node_id.iter() {
                let node_string_split = node.split("_").collect::<Vec<&str>>();
                let x_index = node_string_split[2].parse::<u8>().unwrap();
                let y_index = node_string_split[1].parse::<u8>().unwrap();
                let node_state_string_split = node_state.split("_").collect::<Vec<&str>>();
                let state = node_state_string_split[1].parse::<u8>().unwrap();
                state_per_row_per_column[y_index as usize][x_index as usize] = Some(state);
            }

            let solved_puzzle = SudokuPuzzle {
                number_per_row_per_column: state_per_row_per_column
            };
            Ok(solved_puzzle)
        }
        else {
            Err(collapsed_wave_function_result.err().unwrap())
        }
    }
}

fn main() {

    let start = Instant::now();

    let mut number_per_row_per_column: Vec<Vec<Option<u8>>> = Vec::new();
    number_per_row_per_column.push(vec![None,    Some(7), Some(3), Some(2), None,    Some(4), Some(6), Some(9), Some(1)]);
    number_per_row_per_column.push(vec![None,    Some(2), Some(8), None,    None,    Some(6), None,    None,    Some(7)]);
    number_per_row_per_column.push(vec![None,    None,    Some(6), Some(1), None,    Some(7), None,    None,    Some(8)]);
    number_per_row_per_column.push(vec![None,    Some(1), Some(5), Some(7), Some(6), Some(3), None,    Some(2), Some(4)]);
    number_per_row_per_column.push(vec![Some(6), None,    None,    None,    None,    None,    Some(8), Some(7), None]);
    number_per_row_per_column.push(vec![Some(7), None,    None,    Some(9), None,    None,    None,    None,    None]);
    number_per_row_per_column.push(vec![Some(3), None,    Some(1), Some(6), None,    None,    None,    None,    None]);
    number_per_row_per_column.push(vec![Some(2), Some(8), None,    Some(5), Some(4), Some(9), Some(3), None,    None]);
    number_per_row_per_column.push(vec![None,    Some(6), None,    Some(8), None,    None,    None,    None,    None]);
    let puzzle = SudokuPuzzle {
        number_per_row_per_column: number_per_row_per_column
    };
    puzzle.print();

    let solution_result = puzzle.get_solution();

    if let Ok(solution) = solution_result {
        solution.print();
    }
    else {
        println!("Error: {}", solution_result.err().unwrap());
    }

    let duration = start.elapsed();
    println!("Duration: {:?}", duration);
}