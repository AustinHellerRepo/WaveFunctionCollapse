use std::{collections::{HashMap, HashSet}, cell::Cell};
use serde::Deserialize;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use bitvec::prelude::*;

#[derive(Debug, Deserialize)]
pub struct Node {
    id: String,
    node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>>
}

impl<'a> Node {
    fn get_possible_node_states(&self, node_state_collection_per_id: &'a HashMap<&'a str, &'a NodeStateCollection>) -> Result<Vec<&str>, String> {
        
        let mut possible_node_state_ids_option: Option<Vec<&str>> = Option::None;
        let mut possible_node_state_ids_length: usize = 0;
        
        for (neighbor_node_id_string, node_state_collection_ids) in self.node_state_collection_ids_per_neighbor_node_id.iter() {
            let neighbor_node_id: &str = neighbor_node_id_string;
            // either fill the possible node states or verify other permitted node state collections
            
            let mut possible_node_state_ids_vector: Vec<&str> = Vec::new();
            // store the possible node states
            
            if possible_node_state_ids_option.is_none() {
                for node_state_collection_id_string in node_state_collection_ids.iter() {
                    let node_state_collection_id: &str = node_state_collection_id_string;
                    let node_state_collection = node_state_collection_per_id.get(node_state_collection_id).unwrap();
                    let node_state_id: &str = &node_state_collection.node_state_id;
        
                    // store this node state as a possible node state
                    if !possible_node_state_ids_vector.contains(&node_state_id) {
                        possible_node_state_ids_vector.push(node_state_id);
                        possible_node_state_ids_length += 1;
                    }
                    else {
                        let node_id = &self.id;
                        return Err(format!("Duplicate node state id found in node {node_id}."));
                    }
                }
            }
            else {
                let possible_node_state_ids_vector: Vec<&str> = Vec::new();
                let mut possible_node_state_ids_vector_length: usize = 0;

                // store the possible node states
                for node_state_collection_id_string in node_state_collection_ids.iter() {
                    let node_state_collection_id: &str = node_state_collection_id_string;
                    let node_state_collection = node_state_collection_per_id.get(node_state_collection_id).unwrap();
                    let node_state_id: &str = &node_state_collection.node_state_id;

                    // store this node state as a possible node state
                    if possible_node_state_ids_vector.contains(&node_state_id) {
                        let node_id = &self.id;
                        return Err(format!("Duplicate node state id found in node {node_id}."));
                    }
                    else {
                        possible_node_state_ids_vector_length += 1;
                        if !possible_node_state_ids_option.as_ref().unwrap().contains(&node_state_id) {
                            return Err(format!("Unexpected node state {node_state_id} in permitted node state collection {node_state_collection_id}."));
                        }
                    }
                }

                if possible_node_state_ids_vector_length != possible_node_state_ids_length {
                    return Err(format!("Missing at least one node state in neighbor node {neighbor_node_id}"));
                }
            }
        }

        match &possible_node_state_ids_option {
            Some(possible_node_state_ids) => {
                return Ok(possible_node_state_ids.to_owned());
            }
            None => {
                return Err(String::from("This node does not contain any neighbors."));
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NodeStateCollection {
    id: String,
    node_state_id: String,
    node_state_ids: Vec<String>
}

struct CollapsableNode<'a> {
    // the node id that this collapsable node refers to
    id: &'a str,
    // this nodes list of neighbor node ids
    neighbor_node_ids: Vec<&'a str>,
    // the full list of possible node states
    node_state_ids: Vec<&'a str>,
    // the length of the node_state_ids object
    node_state_ids_length: usize,
    // the possible masks to provide to this node's neighbors based on the current state of this node
    node_state_mask_per_node_state_id_per_neighbor_node_id: HashMap<&'a str, HashMap<&'a str, BitVec>>,
    // the current masks that this node's neighbors will reference
    current_node_state_mask_per_neighbor_node_id: HashMap<&'a str, Cell<Option<&'a BitVec>>>,
    // the masks applied to this node from the neighbors, Option::None means that there is no restriction
    applied_node_state_masks: Vec<Cell<Option<&'a BitVec>>>,
    // this holds the result of bitwise-anding the neighbor masks
    current_is_valid_per_node_state_id: HashMap<&'a str, bool>,
    // the current index running over the permitted mask bits, None if not yet started
    current_node_state_id_index: Option<usize>,
    // the index of traversed nodes based on the sorted vector of nodes as they are chosen for state determination
    current_chosen_from_sort_index: Option<u32>,
    // if the collapsable node is fully restricted and is a simplification of current_is_valid_per_node_state_id being all false
    current_is_fully_restricted: bool,
    // a random sort value for adding randomness to the process between runs (if randomized)
    random_sort_index: u32
}

impl<'a> CollapsableNode<'a> {
    fn new(node: &'a Node, node_per_id: &'a HashMap<&'a str, &'a Node>, node_state_collection_per_id: &'a HashMap<&'a str, &'a NodeStateCollection>, cell_per_neighbor_node_id_per_node_id: &HashMap<&str, HashMap<&str, Cell<Option<&BitVec>>>>) -> CollapsableNode {
        // get the neighbors for this node
        let mut neighbor_node_ids: Vec<&str> = Vec::new();
        // contains the possible node states and is None so that only the first neighbor will supply the possible values while the others are checked to ensure that they consistent
        let mut possible_node_state_ids: Vec<&str> = node.get_possible_node_states(node_state_collection_per_id).expect("The node should be able to provide its possible node states if it knows how its states related to its neighbors.");
        // get the possible node states; pulled from the possible states and how they impact the node's neighbors
        let mut possible_node_state_ids_length: usize = possible_node_state_ids.len();
        // contains the mask to apply to the neighbor when this node is in a specific state
        let mut node_state_mask_per_node_state_id_per_neighbor_node_id: HashMap<&'a str, HashMap<&'a str, BitVec>> = CollapsableNode::get_node_state_mask_per_node_state_id_per_neighbor_node_id(node, node_per_id, node_state_collection_per_id);
        // contains the default (None) applied masks that my neighbors would give to me
        let mut current_node_state_mask_per_neighbor_node_id : HashMap<&str, Cell<Option<&BitVec>>> = HashMap::new();

        for neighbor_node_id_string in node.node_state_collection_ids_per_neighbor_node_id.keys() {
            let neighbor_node_id: &str = neighbor_node_id_string;
            neighbor_node_ids.push(neighbor_node_id);
        }

        let node_id: &str = &node.id;

        for (neighbor_node_id_string, mask_cell) in cell_per_neighbor_node_id_per_node_id.get(node_id).expect("The node id should exist in the hashmap.") {
            current_node_state_mask_per_neighbor_node_id.insert(neighbor_node_id_string, *mask_cell);
        }

        let mut current_is_valid_per_node_state_id: HashMap<&str, bool> = HashMap::new();
        for possible_node_state_id in possible_node_state_ids.iter() {
            current_is_valid_per_node_state_id.insert(&possible_node_state_id, true);
        }

        CollapsableNode {
            id: &node.id,
            neighbor_node_ids: neighbor_node_ids,
            node_state_ids: possible_node_state_ids,
            node_state_ids_length: possible_node_state_ids_length,
            node_state_mask_per_node_state_id_per_neighbor_node_id: node_state_mask_per_node_state_id_per_neighbor_node_id,
            current_node_state_mask_per_neighbor_node_id: current_node_state_mask_per_neighbor_node_id,
            applied_node_state_masks: Vec::new(),
            current_is_valid_per_node_state_id: current_is_valid_per_node_state_id,
            current_node_state_id_index: Option::None,
            current_chosen_from_sort_index: Option::None,
            current_is_fully_restricted: (possible_node_state_ids_length == 0),
            random_sort_index: 0
        }
    }
    fn randomize(&mut self, seed: u64) {
        let mut random_instance = ChaCha8Rng::seed_from_u64(seed);
        self.neighbor_node_ids.shuffle(&mut random_instance);
        self.node_state_ids.shuffle(&mut random_instance);
        self.random_sort_index = random_instance.next_u32();
    }
    fn try_increment_state_id_index(&'a mut self) -> bool {
        let mut is_successful: bool;

        if let Some(index) = self.current_node_state_id_index {
            let mut current_index = index + 1;
            let mut next_index: Option<usize> = Option::None;

            let mut current_possible_state: &str;
            while current_index < self.node_state_ids_length {
                current_possible_state = self.node_state_ids.get(index).expect("The possible state index should be inside the bounds of the vector.");
                if *self.current_is_valid_per_node_state_id.get(current_possible_state).expect("The dictionary should contain all of the same state ids.") {
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
                self.current_node_state_id_index = next_index;
                is_successful = true;
            }
        }
        else {
            if self.node_state_ids_length == 0 {
                is_successful = false;
            }
            else {
                self.current_node_state_id_index = Some(0);
                is_successful = true;
            }
        }

        if is_successful {
            let current_possible_state: &str = self.node_state_ids.get(self.current_node_state_id_index.unwrap()).unwrap();
            for (neighbor_node_id, mask_box) in self.current_node_state_mask_per_neighbor_node_id.iter() {
                let mask: &'a BitVec = self.node_state_mask_per_node_state_id_per_neighbor_node_id.get(neighbor_node_id).unwrap().get(current_possible_state).unwrap();
                mask_box.replace(Option::Some(mask));
            }
        }

        is_successful
    }
    fn set_applied_node_state_masks(&'a mut self, applied_node_state_masks: Vec<Cell<Option<&'a BitVec>>>) {
        for mask_box in applied_node_state_masks.iter() {
            self.applied_node_state_masks.push(mask_box.to_owned());
        }
    }
    fn get_mask_box_references_for_node_id(collapsable_nodes: &'a Vec<CollapsableNode<'a>>, node_id: &'a str) -> Vec<Cell<Option<&'a BitVec>>> {
        let mut mask_boxes: Vec<Cell<Option<&BitVec>>> = Vec::new();

        for collapsable_node in collapsable_nodes.iter() {
            for neighbor_node_id in collapsable_node.neighbor_node_ids.iter() {
                if node_id == *neighbor_node_id {
                    let mask_box = collapsable_node.current_node_state_mask_per_neighbor_node_id.get(node_id).expect("This mask should exist since it was just checked for.");
                    mask_boxes.push(mask_box.to_owned());
                }
            }
        }

        mask_boxes
    }
    fn get_node_state_mask_per_node_state_id_per_neighbor_node_id(node: &'a Node, node_per_id: &'a HashMap<&'a str, &'a Node>, node_state_collection_per_id: &'a HashMap<&'a str, &NodeStateCollection>) -> HashMap<&'a str, HashMap<&'a str, BitVec>> {

        // for each neighbor node
        //      for each possible state for this node
        //          create a mutable bit vector
        //          for each possible node state for the neighbor node
        //              get if the neighbor node state is permitted by this node's possible node state
        //              push the boolean into bit vector
        //          push bit vector into hashmap of mask per node state per neighbor node

        let mut node_state_mask_per_node_state_id_per_neighbor_node_id: HashMap<&str, HashMap<&str, BitVec>> = HashMap::new();

        for (neighbor_node_id_string, node_state_collection_ids) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
            let neighbor_node_id: &str = neighbor_node_id_string;

            // determine the masks for this neighbor based on its possible node states
            let neighbor_node_state_ids = node_per_id.get(neighbor_node_id).expect("The neighbor should exist in the complete list of nodes.").get_possible_node_states(&node_state_collection_per_id).expect("The neighbor node should have some node states.");

            // stores the mask per node state for this node as it pertains to the neighbor
            let mut node_state_mask_per_node_state_id: HashMap<&str, BitVec> = HashMap::new();

            // this loop ultimately is over each possible state of this node
            for node_state_collection_id_string in node_state_collection_ids.iter() {
                let node_state_collection_id: &str = &node_state_collection_id_string;
                let node_state_collection = node_state_collection_per_id.get(node_state_collection_id).expect("The node state collection id should exist in the complete list of node state collections.");

                // each possible node state for this node should have a mask for this neighbor
                let mut mask = BitVec::new();

                for neighbor_node_state_id in neighbor_node_state_ids.iter() {
                    let mut is_permitted: bool = false;
                    for node_state_id_string in node_state_collection.node_state_ids.iter() {
                        let node_state_id: &str = node_state_id_string;
                        if node_state_id == *neighbor_node_state_id {
                            is_permitted = true;
                            break;
                        }
                    }
                    mask.push(is_permitted)
                }

                let possible_node_state_id: &str = &node_state_collection.node_state_id;
                node_state_mask_per_node_state_id.insert(possible_node_state_id, mask);
            }
            node_state_mask_per_node_state_id_per_neighbor_node_id.insert(neighbor_node_id, node_state_mask_per_node_state_id);
        }

        node_state_mask_per_node_state_id_per_neighbor_node_id
    }
}

pub struct CollapsedWaveFunction {
    // TODO
}

pub struct WaveFunction {
    nodes: Vec<Node>,
    node_state_collections: Vec<NodeStateCollection>,
}

impl WaveFunction {
    pub fn new(nodes: Vec<Node>, node_state_collections: Vec<NodeStateCollection>) -> Self {

        WaveFunction {
            nodes: nodes,
            node_state_collections: node_state_collections,
        }
    }
    pub fn validate(&self) -> Option<String> {
        let nodes_length: usize = self.nodes.len();
        let mut node_per_id: HashMap<&str, &Node> = HashMap::new();
        let mut node_ids: HashSet<&str> = HashSet::new();
        self.nodes.iter().for_each(|node: &Node| {
            node_per_id.insert(&node.id, node);
            node_ids.insert(&node.id);
        });
        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection> = HashMap::new();
        self.node_state_collections.iter().for_each(|node_state_collection| {
            node_state_collection_per_id.insert(&node_state_collection.id, node_state_collection);
        });

        let mut error_message = Option::None;

        // TODO collect all possible states
        // TODO ensure that references neighbors are actually nodes
        // TODO ensure that all nodes, for each neighbor, specify how to react to each possible state
        // TODO ensure that valid states for neighbors do not contain duplicate states
        
        let mut all_possible_node_state_ids: HashSet<&str> = HashSet::new();
        for (node_id, node) in node_per_id.iter() {
            for (neighbor_node_id_string, node_state_collection_ids) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
                let neighbor_node_id: &str = neighbor_node_id_string;
                for node_state_collection_id_string in node_state_collection_ids {
                    let node_state_collection_id: &str = &node_state_collection_id_string;
                    let node_state_collection: &NodeStateCollection = node_state_collection_per_id.get(node_state_collection_id).expect("The permitted node state collection should exist for this id.");
                    let node_state_id: &str = &node_state_collection.node_state_id;

                    all_possible_node_state_ids.insert(node_state_id);
                    for node_state_id_string in node_state_collection.node_state_ids.iter() {
                        let node_state_id: &str = node_state_id_string;
                        all_possible_node_state_ids.insert(node_state_id);
                    }
                }
            }
        }
        let all_possible_node_state_ids_length: u32 = all_possible_node_state_ids.len().try_into().expect("The length should be castable to u32.");

        // ensure that references neighbors are actually nodes
        for (node_id, node) in node_per_id.iter() {
            for (neighbor_node_id_string, node_state_collection_ids) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
                let neighbor_node_id: &str = neighbor_node_id_string;
                if !node_ids.contains(neighbor_node_id) {
                    error_message = Some(format!("Neighbor node {neighbor_node_id} does not exist in main list of nodes."));
                    break;
                }
            }
            if error_message.is_some() {
                break;
            }
        }

        if error_message.is_none() {
            // ensure that every node, for each neighbor, accounts for all possible states
            for (node_id, node) in node_per_id.iter() {
                for (neighbor_node_id_string, node_state_collection_ids) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
                    let neighbor_node_id: &str = neighbor_node_id_string;
                    let mut node_state_ids: HashSet<&str> = HashSet::new();
                    let mut node_state_ids_length: u32 = 0;
                    for node_state_collection_id_string in node_state_collection_ids {
                        let node_state_collection_id: &str = &node_state_collection_id_string;
                        let node_state_collection: &NodeStateCollection = node_state_collection_per_id.get(node_state_collection_id).expect("The permitted node state collection should exist for this id.");
                        let node_state_id: &str = &node_state_collection.node_state_id;
                        if node_state_ids.contains(node_state_id) {
                            error_message = Some(format!("Found duplicate node state when node {node_id} references neighbor node {neighbor_node_id}."));
                            break;
                        }
                        else {
                            node_state_ids.insert(node_state_id);
                            node_state_ids_length += 1;
                        }
                    }
                    if error_message.is_some() {
                        break;
                    }
                    else {
                        if node_state_ids_length != all_possible_node_state_ids_length {
                            error_message = Some(format!("Missing at least one node state reference when node {node_id} references neighbor node {neighbor_node_id}."));
                            break;
                        }
                    }
                }
                if error_message.is_some() {
                    break;
                }
            }
        }

        return error_message;
    }
    pub fn collapse(&self) -> Result<CollapsedWaveFunction, String> {
        let nodes_length: usize = self.nodes.len();
        let mut node_per_id: HashMap<&str, &Node> = HashMap::new();
        self.nodes.iter().for_each(|node: &Node| {
            node_per_id.insert(&node.id, node);
        });
        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection> = HashMap::new();
        self.node_state_collections.iter().for_each(|node_state_collection| {
            node_state_collection_per_id.insert(&node_state_collection.id, node_state_collection);
        });

        let mut cell_per_neighbor_node_id_per_node_id: HashMap<&str, HashMap<&str, Cell<Option<&BitVec>>>> = HashMap::new();
        for node in self.nodes.iter() {
            let node_id: &str = &node.id;
            let mut cell_per_neighbor_node_id: HashMap<&str, Cell<Option<&BitVec>>> = HashMap::new();
            for neighbor_node_id_string in node.node_state_collection_ids_per_neighbor_node_id.keys() {
                let neighbor_node_id: &str = neighbor_node_id_string;

                let cell: Cell<Option<&BitVec>> = Cell::new(Option::None);
                
                cell_per_neighbor_node_id.insert(neighbor_node_id, cell);
            }
            cell_per_neighbor_node_id_per_node_id.insert(node_id, cell_per_neighbor_node_id);
        }

        let mut collapsable_node_index: usize = 0;
        let mut collapsable_nodes: Vec<CollapsableNode> = Vec::new();
        let mut collapsable_node_index_per_node_id: HashMap<&str, usize> = HashMap::new();
        for node in self.nodes.iter() {
            let collapsable_node = CollapsableNode::new(node, &node_per_id, &node_state_collection_per_id, &cell_per_neighbor_node_id_per_node_id);
            collapsable_nodes.push(collapsable_node);
            collapsable_node_index_per_node_id.insert(&node.id, collapsable_node_index.clone());
            collapsable_node_index = collapsable_node_index + 1;
        }

        // TODO use the provided cells in cell_per_neighbor_node_id_per_node_id during construction of CollapsableNode

        // attach the collapsable node masks to their neighbors
        for node in self.nodes.iter() {
            let mask_boxes = CollapsableNode::get_mask_box_references_for_node_id(&collapsable_nodes, &node.id);
            for collapsable_node in collapsable_nodes.iter() {
                if collapsable_node.id == node.id {
                    collapsable_node.set_applied_node_state_masks(mask_boxes);
                }
            }
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
        //          TODO alter reference to mask
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
        while !is_unable_to_collapse && current_collapsable_node_index != nodes_length {
            if is_sort_necessary {
                collapsable_nodes.sort_by(|a, b| {

                    let comparison: std::cmp::Ordering;
                    if let Some(a_chosen_from_sort_index) = a.current_chosen_from_sort_index {
                        if let Some(b_chosen_from_sort_index) = b.current_chosen_from_sort_index {
                            comparison = a_chosen_from_sort_index.cmp(&b_chosen_from_sort_index);
                        }
                        else {
                            comparison = std::cmp::Ordering::Less
                        }
                    }
                    else if b.current_chosen_from_sort_index.is_some() {
                        comparison = std::cmp::Ordering::Greater
                    }
                    else {
                        let a_possible_states_count = a.node_state_ids_length;
                        let b_possible_states_count = b.node_state_ids_length;

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
                });
            }

            // if the current collapsable node has a chosen state, invalidate the mask in the neighbors
            {
                let mut neighbor_node_ids: Vec<&str> = Vec::new();
                let mut current_collapsable_node_id_option: Option<&str> = Option::None;

                let current_collapsable_node = collapsable_nodes.get(current_collapsable_node_index).expect("The collapsable node index should be within range.");
                if current_collapsable_node.current_node_state_id_index.is_some() {
                    current_collapsable_node_id_option = Some(current_collapsable_node.id);
                    for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                        neighbor_node_ids.push(neighbor_node_id);
                    }
                }

                if let Some(current_collapsable_node_id) = current_collapsable_node_id_option {
                    for (neighbor_node_id, mask_box) in current_collapsable_node.current_node_state_mask_per_neighbor_node_id.iter() {
                        //mask_box.write(Option::None);
                    }
                }
            }

            // try to increment the state of the current collapsable node
            let is_incremented = collapsable_nodes.get_mut(current_collapsable_node_index).expect("The collapsable node should exist at this index.").try_increment_state_id_index();

            if is_incremented {

                let mut is_at_least_one_neighbor_fully_restricted = false;

                {
                    let mut collapsable_node_per_id: HashMap<&str, &CollapsableNode> = HashMap::new();
                    for collapsable_node in collapsable_nodes.iter() {
                        collapsable_node_per_id.insert(collapsable_node.id, collapsable_node);
                    }

                    let current_collapsable_node = collapsable_nodes.get(current_collapsable_node_index).expect("The collapsable node index should be within range.");

                    for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                        let collapsable_node = collapsable_node_per_id.get(neighbor_node_id).expect(&format!("The list of collapsable nodes originally provided should contain this neighbor node id {neighbor_node_id}"));
                        if collapsable_node.current_is_fully_restricted {
                            is_at_least_one_neighbor_fully_restricted = true;
                            break;
                        }
                    }
                }

                if is_at_least_one_neighbor_fully_restricted {
                    // this node needs to be attempted again
                }
                else {
                    current_collapsable_node_index = current_collapsable_node_index + 1;
                    is_sort_necessary = true;
                }
            }
            else {
                let original_collapsable_node_id = collapsable_nodes.get_mut(current_collapsable_node_index).expect("The index should still be within the range of collapsable nodes.").id;
                while !is_unable_to_collapse {
                    if current_collapsable_node_index == 0 {
                        is_unable_to_collapse = true;
                    }
                    else {
                        collapsable_nodes.get_mut(current_collapsable_node_index).expect("The node should exist at this index.").current_node_state_id_index = Option::None;
                        current_collapsable_node_index = current_collapsable_node_index - 1;
                        for neighbor_node_id in collapsable_nodes.get_mut(current_collapsable_node_index).expect("The node index should exist in the range of collapsable nodes since the earlier if-statement would trigger leaving the while loop.").neighbor_node_ids.iter() {
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