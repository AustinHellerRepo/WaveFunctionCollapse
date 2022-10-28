use std::collections::{HashMap, HashSet};
use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct NodeState {
    id: String,
    valid_neighbor_state_ids: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct Node {
    id: String,
    neighbor_node_ids: Vec<String>,
    valid_node_states: Vec<String>
}

pub struct CollapsedWaveFunction {

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
        Err(String::from("Not Implemented"))
    }
}