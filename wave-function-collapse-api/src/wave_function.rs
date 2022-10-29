use std::collections::{HashMap, HashSet};
use serde::Deserialize;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

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
    is_valid_per_state_id: HashMap<&'a str, bool>,
    state_id_index: Option<usize>,
    possible_state_ids_length: usize,
    chosen_from_sort_index: Option<u32>,
    random_sort_index: u32
}

impl<'a> CollapsableNode<'a> {
    fn new(node: &'a Node) -> CollapsableNode {
        let mut neighbor_node_ids: Vec<&str> = Vec::new();
        for neighbor_node_id in node.neighbor_node_ids.iter() {
            neighbor_node_ids.push(&neighbor_node_id);
        }
        let mut possible_state_ids_length: usize = 0;
        let mut possible_state_ids: Vec<&str> = Vec::new();
        let mut is_valid_per_state_id: HashMap<&'a str, bool> = HashMap::new();
        for valid_node_state_id in node.valid_node_state_ids.iter() {
            is_valid_per_state_id.insert(valid_node_state_id, true);
            possible_state_ids.push(valid_node_state_id);
            possible_state_ids_length = possible_state_ids_length + 1;
        }
        CollapsableNode {
            id: &node.id,
            neighbor_node_ids: neighbor_node_ids,
            possible_state_ids: possible_state_ids,
            is_valid_per_state_id: is_valid_per_state_id,
            state_id_index: Option::None,
            possible_state_ids_length: possible_state_ids_length,
            chosen_from_sort_index: Option::None,
            random_sort_index: 0
        }
    }
    fn randomize(&mut self, seed: u64) {
        let mut random_instance = ChaCha8Rng::seed_from_u64(seed);
        self.neighbor_node_ids.shuffle(&mut random_instance);
        self.possible_state_ids.shuffle(&mut random_instance);
        self.random_sort_index = random_instance.next_u32();
    }
    fn get_possible_states_count(&self) -> u32 {
        let possible_states_count: u32 = self.possible_state_ids.len().try_into().expect("The length of self.possible_state_ids should be convertable to a u32.");
        possible_states_count
    }
    fn try_increment_state_id_index(&mut self) -> bool {
        let mut is_successful: bool;

        if let Some(index) = self.state_id_index {
            let mut current_index = index + 1;
            let mut next_index: Option<usize> = Option::None;

            let mut current_possible_state: &str;
            while current_index < self.possible_state_ids_length {
                current_possible_state = self.possible_state_ids.get(index).expect("The possible state index should be inside the bounds of the vector.");
                if *self.is_valid_per_state_id.get(current_possible_state).expect("The dictionary should contain all of the same state ids.") {
                    next_index = Some(current_index);
                    break;
                }
                else {
                    current_index = current_index + 1;
                }
            }

            if next_index.is_none() {
                is_successful = false;
            }
            else {
                self.state_id_index = next_index;
                is_successful = true;
            }
        }
        else {
            if self.possible_state_ids_length == 0 {
                is_successful = false;
            }
            else {
                self.state_id_index = Some(0);
                is_successful = true;
            }
        }

        is_successful
    }
    fn invalidate_possible_state(&mut self, state_id: &'a str) {
        if self.is_valid_per_state_id.contains_key(state_id) {
            self.is_valid_per_state_id.insert(state_id, false);
        }
    }
    fn revalidate_possible_state(&mut self, state_id: &'a str) {
        if self.is_valid_per_state_id.contains_key(state_id) {
            self.is_valid_per_state_id.insert(state_id, true);
        }
    }
}

pub struct CollapsedWaveFunction {
    // TODO fill with final nodes and their state
}

pub struct WaveFunction {
    states: Vec<NodeState>,
    nodes: Vec<Node>,
    nodes_length: usize
}

impl WaveFunction {
    pub fn new(states: Vec<NodeState>, nodes: Vec<Node>) -> Self {
        let nodes_length: usize = nodes.len();
        WaveFunction {
            states: states,
            nodes: nodes,
            nodes_length: nodes_length
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
        let mut collapsable_node_index: usize = 0;
        let mut collapsable_nodes: Vec<CollapsableNode> = Vec::new();
        let mut collapsable_node_index_per_node_id: HashMap<&str, usize> = HashMap::new();
        for node in self.nodes.iter() {
            let collapsable_node = CollapsableNode::new(node);
            collapsable_nodes.push(collapsable_node);
            collapsable_node_index_per_node_id.insert(&node.id, collapsable_node_index);
            collapsable_node_index = collapsable_node_index + 1;
        }

        // set sort necessary
        // set error message as None
        // while no error message and the collapsable node index is less than the total number of collapsable nodes

        //      if sort is necessary
        //          sort by (1) chosen from sorted collapsable nodes vector index (in order to maintain the chosen order) and then (2) least possible states being first (in order to adjust the next possible nodes to pick the most restricted nodes first)

        //      if current collapsable node has Some state id index
        //          inform neighbors that this state id is now available again (if applicable)
        
        //      try to increment the current collapsable node state id index (maybe just going from None to Some(0))

        //      if succeeded to increment
        //          inform neighbors of new state
        //          TODO react to neighbor no longer having any valid states
        //          increment current collapsable node index
        //          set sort necessary
        //      else (then we need to try a different state for the most recent parent that has the current node as a neighbor)
        //          while not yet errored
        //              if current collapsable node index is the first node (then the nodes have been exhausted)
        //                  set error message
        //              else
        //                  set current collapsable node's state id index to None
        //                  decrement current collapsale node index
        //                  if one of the newly current collapsable node's neighbors is the original collapsable node
        //                      break
        //          set sort unnecessary

        let mut current_collapsable_node_index: usize = 0;
        let mut is_sort_necessary = true;
        let mut is_unable_to_collapse = false;
        while !is_unable_to_collapse && current_collapsable_node_index != self.nodes_length {
            if is_sort_necessary {
                collapsable_nodes.sort_by(|a, b| {

                    let comparison: std::cmp::Ordering;
                    if let Some(a_chosen_from_sort_index) = a.chosen_from_sort_index {
                        if let Some(b_chosen_from_sort_index) = b.chosen_from_sort_index {
                            comparison = a_chosen_from_sort_index.cmp(&b_chosen_from_sort_index);
                        }
                        else {
                            comparison = std::cmp::Ordering::Less
                        }
                    }
                    else if b.chosen_from_sort_index.is_some() {
                        comparison = std::cmp::Ordering::Greater
                    }
                    else {
                        let a_possible_states_count = a.get_possible_states_count();
                        let b_possible_states_count = b.get_possible_states_count();

                        if b_possible_states_count < a_possible_states_count {
                            comparison = std::cmp::Ordering::Greater
                        }
                        else if b_possible_states_count == a_possible_states_count {
                            comparison = std::cmp::Ordering::Equal
                        }
                        else {
                            comparison = std::cmp::Ordering::Less
                        }
                    }
                    comparison
                })
            }

            let mut neighbor_node_ids: Vec<&str> = Vec::new();
            let current_state_id_option: Option<&str>;

            {
                let current_collapsable_node = collapsable_nodes.get(current_collapsable_node_index).expect("The collapsable node index should be within range.");
                if let Some(state_id_index) = current_collapsable_node.state_id_index {
                    current_state_id_option = Some(current_collapsable_node.possible_state_ids[state_id_index]);
                    for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                        neighbor_node_ids.push(neighbor_node_id);
                    }
                }
                else {
                    current_state_id_option = Option::None;
                }
            }

            if let Some(current_state_id) = current_state_id_option {
                for neighbor_node_id in neighbor_node_ids {
                    let collapsable_node_index = collapsable_node_index_per_node_id.get(neighbor_node_id).expect("The neighbor should exist in the hashmap since it was verified previously.");
                    let collapsable_node = collapsable_nodes.get_mut(*collapsable_node_index).expect("The provided index should exist in the vector.");
                    collapsable_node.revalidate_possible_state(current_state_id);
                }
            }

            let is_incremented = collapsable_nodes.get_mut(current_collapsable_node_index).expect("The collapsable node should exist at this index.").try_increment_state_id_index();

            if is_incremented {

                let mut neighbor_node_ids: Vec<&str> = Vec::new();
                let current_state_id_option: Option<&str>;

                {
                    let current_collapsable_node = collapsable_nodes.get(current_collapsable_node_index).expect("The collapsable node index should be within range.");
                    if let Some(state_id_index) = current_collapsable_node.state_id_index {
                        current_state_id_option = Some(current_collapsable_node.possible_state_ids[state_id_index]);
                        for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                            neighbor_node_ids.push(neighbor_node_id);
                        }
                    }
                    else {
                        current_state_id_option = Option::None;
                    }
                }

                let current_state_id = current_state_id_option.expect("The current state should be set for the current node if the increment succeeded.");

                for neighbor_node_id in neighbor_node_ids {
                    let collapsable_node_index = collapsable_node_index_per_node_id.get(neighbor_node_id).expect("The neighbor should exist in the hashmap since it was verified previously.");
                    let collapsable_node = collapsable_nodes.get_mut(*collapsable_node_index).expect("The provided index should exist in the vector.");
                    collapsable_node.invalidate_possible_state(current_state_id);
                }

                current_collapsable_node_index = current_collapsable_node_index + 1;
                is_sort_necessary = true;
            }
            else {
                let original_collapsable_node_id = collapsable_nodes.get(current_collapsable_node_index).expect("The index should still be within the range of collapsable nodes.").id;
                while !is_unable_to_collapse {
                    if current_collapsable_node_index == 0 {
                        is_unable_to_collapse = true;
                    }
                    else {
                        collapsable_nodes.get_mut(current_collapsable_node_index).expect("The node should exist at this index.").state_id_index = Option::None;
                        current_collapsable_node_index = current_collapsable_node_index - 1;
                        for neighbor_node_id in collapsable_nodes.get(current_collapsable_node_index).expect("The node index should exist in the range of collapsable nodes since the earlier if-statement would trigger leaving the while loop.").neighbor_node_ids.iter() {
                            if *neighbor_node_id == original_collapsable_node_id {
                                break;
                            }
                        }
                    }
                }
                is_sort_necessary = false;
            }
        }

        Err(String::from("Not Implemented"))
    }
}