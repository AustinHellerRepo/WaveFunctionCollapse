use std::{collections::{HashMap, HashSet}, cell::{Cell, RefCell}, rc::Rc};
use serde::Deserialize;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use bitvec::prelude::*;

mod indexed_view;
use self::indexed_view::IndexedView;
mod mapped_view;
use self::mapped_view::MappedView;

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
    // all possible node states given what is permitted by this node's neighbors
    node_state_ids: Vec<&'a str>,
    // this nodes list of neighbor node ids
    neighbor_node_ids: Vec<&'a str>,
    // the full list of possible node states, masked by internal references to neighbor masks
    node_state_indexed_view: IndexedView<&'a str, &'a str, &'a str>,
    // the length of the node_state_ids object
    node_state_ids_length: usize,
    // the mapped view that this node's neighbors will have a reference to and pull their masks from
    neighbor_mask_mapped_view: Rc<RefCell<MappedView<&'a str, &'a str, BitVec>>>,
    // the index of traversed nodes based on the sorted vector of nodes as they are chosen for state determination
    current_chosen_from_sort_index: Option<u32>,
    // if the collapsable node is fully restricted and is a simplification of current_is_valid_per_node_state_id being all false
    current_is_fully_restricted: bool,
    // a random sort value for adding randomness to the process between runs (if randomized)
    random_sort_index: u32
}

impl<'a> CollapsableNode<'a> {
    fn new(node: &'a Node, node_per_id: &'a HashMap<&'a str, &'a Node>, node_state_collection_per_id: &'a HashMap<&'a str, &'a NodeStateCollection>, neighbor_mask_mapped_view: Rc<RefCell<MappedView<&'a str, &'a str, BitVec>>>, node_state_indexed_view: IndexedView<&'a str, &'a str, &'a str>) -> CollapsableNode<'a> {
        // get the neighbors for this node
        let mut neighbor_node_ids: Vec<&str> = Vec::new();
        // contains the possible node states and is None so that only the first neighbor will supply the possible values while the others are checked to ensure that they consistent
        let mut node_state_ids: Vec<&str> = node.get_possible_node_states(node_state_collection_per_id).expect("The node should be able to provide its possible node states if it knows how its states related to its neighbors.");
        // get the possible node states; pulled from the possible states and how they impact the node's neighbors
        let mut node_state_ids_length: usize = node_state_ids.len();

        for neighbor_node_id_string in node.node_state_collection_ids_per_neighbor_node_id.keys() {
            let neighbor_node_id: &str = neighbor_node_id_string;
            neighbor_node_ids.push(neighbor_node_id);
        }

        let node_id: &str = &node.id;

        CollapsableNode {
            id: &node.id,
            node_state_ids: node_state_ids,
            node_state_ids_length: node_state_ids_length,
            neighbor_node_ids: neighbor_node_ids,
            node_state_indexed_view: node_state_indexed_view,
            neighbor_mask_mapped_view: neighbor_mask_mapped_view,
            current_chosen_from_sort_index: None,
            current_is_fully_restricted: (node_state_ids_length == 0),
            random_sort_index: 0
        }
    }
    fn randomize(&mut self, seed: u64) {
        let mut random_instance = ChaCha8Rng::seed_from_u64(seed);
        self.neighbor_node_ids.shuffle(&mut random_instance);
        self.node_state_ids.shuffle(&mut random_instance);
        self.random_sort_index = random_instance.next_u32();
    }
    fn try_increment_state_id_index(&mut self) -> bool {

        if self.node_state_indexed_view.try_move_next() {
            let current_possible_state: &str = self.node_state_indexed_view.get();
            self.neighbor_mask_mapped_view.borrow_mut().orient(current_possible_state);
            
            return true;
        }

        return false;
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

pub struct CollapsableWaveFunction<'a> {
    // TODO represents a wave function that only has "collapse" as the main function
    collapsable_nodes: Vec<CollapsableNode<'a>>,
    current_collapsable_node_index: usize,
    is_sort_necessary: bool
}

impl<'a> CollapsableWaveFunction<'a> {
    fn new(collapsable_nodes: Vec<CollapsableNode<'a>>) -> Self {
        CollapsableWaveFunction {
            collapsable_nodes: collapsable_nodes,
            current_collapsable_node_index: 0,
            is_sort_necessary: true
        }
    }
    fn is_sort_necessary(&self) -> bool {
        self.is_sort_necessary
    }
    fn sort_collapsable_nodes(&mut self) {
        self.collapsable_nodes.sort_by(|a, b| {

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
    fn is_current_collapsable_node_in_some_state(&self) -> bool {
        let current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).expect("The collapsable node index should be within range.");
        current_collapsable_node.node_state_indexed_view.is_in_some_state()
    }
    fn retract_current_collapsable_node_state_restrictions_from_neighbors(&self) {

    }
    fn try_increment_current_collapsable_node_state(&mut self) -> bool {
        self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.").try_increment_state_id_index()
    }
    fn try_move_to_next_collapsable_node(&mut self) -> bool {
        // TODO return false if the current node's state is too restrictive on at least one neighbor
        // TODO set sort necessary (if applicable)

        let mut is_at_least_one_neighbor_fully_restricted = false;

        {
            let mut collapsable_node_per_id: HashMap<&str, &CollapsableNode> = HashMap::new();
            for collapsable_node in self.collapsable_nodes.iter() {
                collapsable_node_per_id.insert(collapsable_node.id, collapsable_node);
            }

            let current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).expect("The collapsable node index should be within range.");

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
            self.is_sort_necessary = false;
        }
        else {
            self.current_collapsable_node_index += 1;
            if self.current_collapsable_node_index == self.collapsable_nodes.len() {
                return false;
            }
            self.is_sort_necessary = true;
        }
        return true;
    }
    fn reset_current_collapsable_node_state(&self) {

    }
    fn try_move_to_previous_collapsable_node(&mut self) -> bool {
        // TODO return false if previous turns out to be index 0
        // TODO set sort unnecessary

        self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The node should exist at this index.").node_state_indexed_view.reset();
        self.current_collapsable_node_index -= 1;
        self.current_collapsable_node_index != 0
    }
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

        // for each neighbor node
        //      for each possible state for this node
        //          create a mutable bit vector
        //          for each possible node state for the neighbor node
        //              get if the neighbor node state is permitted by this node's possible node state
        //              push the boolean into bit vector
        //          push bit vector into hashmap of mask per node state per neighbor node

        let mut neighbor_mask_mapped_view_per_node_id: HashMap<&str, Rc<RefCell<MappedView<&str, &str, BitVec>>>> = HashMap::new();

        for node in self.nodes.iter() {

            let mut neighbor_mask_mapped_view: MappedView<&str, &str, BitVec> = MappedView::new();

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

                neighbor_mask_mapped_view.insert_individual(neighbor_node_id, node_state_mask_per_node_state_id);
            }

            neighbor_mask_mapped_view_per_node_id.insert(&node.id, Rc::new(RefCell::new(neighbor_mask_mapped_view)));
        }

        let mut node_state_indexed_view_per_node_id: HashMap<&str, IndexedView<&str, &str, &str>> = HashMap::new();

        for node in self.nodes.iter() {
            let node_id: &str = &node.id;
            let mut masks: Vec<Rc<RefCell<MappedView<&str, &str, BitVec>>>> = Vec::new();
            for (neighbor_node_id_string, node_state_collection_ids) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
                let neighbor_node_id: &str = neighbor_node_id_string;
                let mask: Rc<RefCell<MappedView<&str, &str, BitVec>>> = neighbor_mask_mapped_view_per_node_id.remove(neighbor_node_id).unwrap();
                masks.push(mask);
            }

            let node_state_ids: Vec<&str> = node.get_possible_node_states(&node_state_collection_per_id).unwrap();
            let node_state_indexed_view = IndexedView::new(node_state_ids, masks, node_id);
            node_state_indexed_view_per_node_id.insert(node_id, node_state_indexed_view);
        }

        let mut collapsable_node_index: usize = 0;
        let mut collapsable_nodes: Vec<CollapsableNode> = Vec::new();
        let mut collapsable_node_index_per_node_id: HashMap<&str, usize> = HashMap::new();
        // contains the mask to apply to the neighbor when this node is in a specific state
        for node in self.nodes.iter() {
            let node_id: &str = &node.id;

            let neighber_masked_mapped_view: Rc<RefCell<MappedView<&str, &str, BitVec>>> = neighbor_mask_mapped_view_per_node_id.remove(node_id).unwrap();
            let node_state_indexed_view: IndexedView<&str, &str, &str> = node_state_indexed_view_per_node_id.remove(node_id).unwrap();

            let node_state_mask_per_node_state_id_per_neighbor_node_id: HashMap<&str, HashMap<&str, BitVec>> = CollapsableNode::get_node_state_mask_per_node_state_id_per_neighbor_node_id(&node, &node_per_id, &node_state_collection_per_id);
            let collapsable_node = CollapsableNode::new(node, &node_per_id, &node_state_collection_per_id, neighber_masked_mapped_view, node_state_indexed_view);
            collapsable_nodes.push(collapsable_node);
            collapsable_node_index_per_node_id.insert(&node.id, collapsable_node_index.clone());
            collapsable_node_index = collapsable_node_index + 1;
        }

        // TODO use the provided cells in cell_per_neighbor_node_id_per_node_id during construction of CollapsableNode

        // set sort necessary
        // set error message as None
        // while no error message and the collapsable node index is less than the total number of collapsable nodes
        //
        //      if sort is necessary
        //          sort by (1) chosen from sorted collapsable nodes vector index (in order to maintain the chosen order) and then (2) least possible states being first (in order to adjust the next possible nodes to pick the most restricted nodes first)
        //
        //      if current collapsable node has Some state id index
        //          inform neighbors that this state id is now available again (if applicable)
        //
        //      try to increment the current collapsable node state id index (maybe just going from None to Some(0))
        //
        //      if succeeded to increment
        //          TODO alter reference to mask
        //          TODO react to neighbor no longer having any valid states
        //          increment current collapsable node index
        //          if node index is outside of the bounds
        //              break
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

        /*let mut current_collapsable_node_index: usize = 0;
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
                if current_collapsable_node.current_node_state_id_index.get().is_some() {
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
            let is_incremented = collapsable_nodes.get(current_collapsable_node_index).expect("The collapsable node should exist at this index.").try_increment_state_id_index();

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
                let original_collapsable_node_id = collapsable_nodes.get(current_collapsable_node_index).expect("The index should still be within the range of collapsable nodes.").id;
                while !is_unable_to_collapse {
                    if current_collapsable_node_index == 0 {
                        is_unable_to_collapse = true;
                    }
                    else {
                        collapsable_nodes.get(current_collapsable_node_index).expect("The node should exist at this index.").current_node_state_id_index.set(Option::None);
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
        }*/

        let mut collapsable_wave_function = CollapsableWaveFunction::new(collapsable_nodes);

        let mut is_unable_to_collapse = false;
        let mut is_fully_collapsed = false;
        while !is_unable_to_collapse && !is_fully_collapsed {
            if collapsable_wave_function.is_sort_necessary() {
                collapsable_wave_function.sort_collapsable_nodes();
            }
            if collapsable_wave_function.is_current_collapsable_node_in_some_state() {
                collapsable_wave_function.retract_current_collapsable_node_state_restrictions_from_neighbors();
            }
            let is_incremented = collapsable_wave_function.try_increment_current_collapsable_node_state();
        }

        Err(String::from("Not Implemented"))
    }
}