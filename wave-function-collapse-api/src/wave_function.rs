use std::collections::{HashMap, HashSet};
use serde::Deserialize;
use rand::{self, seq::SliceRandom, SeedableRng};

#[derive(Debug, Deserialize)]
pub struct NodeState {
    id: String,
    valid_neighbor_state_ids: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct Node {
    id: String,
    neighbor_node_ids: Vec<String>,
    valid_node_state_ids: Vec<String>
}

struct CollapsableNode<'a> {
    id: &'a str,
    neighbor_node_ids: Vec<&'a str>,
    possible_state_ids: Vec<&'a str>,
    state_id_index: Option<u32>,
    informed_neighbors: bool
}

impl<'a> CollapsableNode<'a> {
    fn new(node: &'a Node) -> CollapsableNode {
        let mut neighbor_node_ids: Vec<&str> = Vec::new();
        for neighbor_node_id in node.neighbor_node_ids.iter() {
            neighbor_node_ids.push(&neighbor_node_id);
        }
        let mut possible_state_ids: Vec<&str> = Vec::new();
        for valid_node_state_id in node.valid_node_state_ids.iter() {
            possible_state_ids.push(&valid_node_state_id);
        }
        CollapsableNode {
            id: &node.id,
            neighbor_node_ids: neighbor_node_ids,
            possible_state_ids: possible_state_ids,
            state_id_index: Option::None,
            informed_neighbors: bool
        }
    }
    fn randomize(&mut self, seed: u64) {
        let mut random_instance = rand::rngs::StdRng::seed_from_u64(seed);
        self.neighbor_node_ids.shuffle(&mut random_instance);
        self.possible_state_ids.shuffle(&mut random_instance);
    }
    fn get_possible_states_count(&self) -> u32 {
        let possible_states_count: u32 = self.possible_state_ids.len().try_into().expect("The length of self.possible_state_ids should be convertable to a u32.");
        if let Some(index) = self.state_id_index {
            
        }
    }
}

pub struct CollapsedWaveFunction {
    // TODO fill with final nodes and their state
}

pub struct WaveFunction {
    states: Vec<NodeState>,
    nodes: Vec<Node>
}

impl WaveFunction {
    pub fn new(states: Vec<NodeState>, nodes: Vec<Node>) -> Self {
        WaveFunction {
            states: states,
            nodes: nodes
        }
    }
    pub fn validate(&self) -> Option<String> {
        let mut error_message = Option::None;

        // verify that only one copy of a state exists in the list of possible states
        let mut state_ids = HashSet::<&str>::new();
        for state in self.states.iter() {
            let state_id: &str = &state.id;
            if state_ids.contains(state_id) {
                error_message = Some(format!("State ID included more than once: {state_id}"));
                break;
            }
            state_ids.insert(state_id);
        }

        if error_message.is_none() {
            // verify that all state neighbor IDs reference other state IDs
            for state in self.states.iter() {
                for valid_neighbor_state_id in state.valid_neighbor_state_ids.iter() {
                    let valid_neighbor_state_id_str: &str = &valid_neighbor_state_id;
                    if !state_ids.contains(valid_neighbor_state_id_str) {
                        let state_id = &state.id;
                        error_message = Some(format!("Failed to find a state with ID {valid_neighbor_state_id} in state {state_id}"));
                        break;
                    }
                }
                if error_message.is_some() {
                    break;
                }
            }
        }

        if error_message.is_none() {
            // verify that only one copy of a node exists in the list of possible nodes
            let mut node_ids = HashSet::<&str>::new();
            for node in self.nodes.iter() {
                let node_id: &str = &node.id;
                if node_ids.contains(node_id) {
                    error_message = Some(format!("Node ID included more than once: {node_id}"));
                    break;
                }
                node_ids.insert(node_id);
            }

            if error_message.is_none() {
                // verify that all node neighbor IDs reference other node IDs
                for node in self.nodes.iter() {
                    for neighbor_node_id in node.neighbor_node_ids.iter() {
                        let neighbor_node_id_str: &str = &neighbor_node_id;
                        if !node_ids.contains(neighbor_node_id_str) {
                            let node_id = &node.id;
                            error_message = Some(format!("Failed to find a node with ID {neighbor_node_id} in node {node_id}"));
                            break;
                        }
                    }
                    if error_message.is_some() {
                        break;
                    }
                }
            }
        }

        return error_message;
    }
    pub fn collapse(&self) -> Result<CollapsedWaveFunction, String> {
        let mut collapsable_nodes: Vec<CollapsableNode> = Vec::new();
        for node in self.nodes.iter() {
            let collapsable_node = CollapsableNode::new(node);
            collapsable_nodes.push(collapsable_node);
        }

        // set sort necessary
        // while the collapsable node index is less than the total number of collapsable nodes

        // sort by (1) is_some() state_id_index and then (2) least possible states being first

        // try to increment the current collapsable node state id index (maybe just going from None to Some(0))

        // if failed to increment (then we need to try a different state for the most recent parent)

        //      set state id index to None
        //      decrement current collapsale node index
        //      set sort unnecessary

        // else

        //      increment current collapsable node index
        //      set sort necessary

        

        Err(String::from("Not Implemented"))
    }
}