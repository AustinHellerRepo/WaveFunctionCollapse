use std::{collections::{HashMap, HashSet}, cell::{Cell, RefCell}, rc::Rc};
use serde::{Deserialize, Serialize};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use bitvec::prelude::*;
use uuid::Uuid;

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
    fn get_possible_node_states(&self, node_state_collection_per_id: &'a HashMap<&'a str, &'a NodeStateCollection>) -> Result<Vec<&'a str>, String> {
        
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
                possible_node_state_ids_option = Some(possible_node_state_ids_vector);
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

        CollapsableNode {
            id: &node.id,
            node_state_ids: node_state_ids,
            node_state_ids_length: node_state_ids_length,
            neighbor_node_ids: neighbor_node_ids,
            node_state_indexed_view: node_state_indexed_view,
            neighbor_mask_mapped_view: neighbor_mask_mapped_view,
            current_chosen_from_sort_index: None,
            random_sort_index: 0
        }
    }
    fn randomize(&mut self, seed: u64) {
        let mut random_instance = ChaCha8Rng::seed_from_u64(seed);
        self.neighbor_node_ids.shuffle(&mut random_instance);
        self.node_state_ids.shuffle(&mut random_instance);
        self.random_sort_index = random_instance.next_u32();
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
    fn is_neighbor_to(&self, collapsable_node: &CollapsableNode) -> bool {
        for neighbor_node_id in self.neighbor_node_ids.iter() {
            if *neighbor_node_id == collapsable_node.id {
                return true;
            }
        }
        return false;
    }
    fn is_fully_restricted(&self) -> bool {
        if self.node_state_indexed_view.is_in_some_state() {
            false
        }
        else {
            self.node_state_indexed_view.is_fully_restricted()
        }
    }
}

#[derive(Serialize)]
pub struct CollapsedWaveFunction {
    node_state_per_node: HashMap<String, String>
}

pub struct CollapsableWaveFunction<'a> {
    // represents a wave function with all of the necessary steps to collapse
    collapsable_nodes: Vec<CollapsableNode<'a>>,
    collapsable_nodes_length: usize,
    current_collapsable_node_index: usize
}

impl<'a> CollapsableWaveFunction<'a> {
    fn new(collapsable_nodes: Vec<CollapsableNode<'a>>) -> Self {
        let collapsable_nodes_length: usize = collapsable_nodes.len();
        CollapsableWaveFunction {
            collapsable_nodes: collapsable_nodes,
            collapsable_nodes_length: collapsable_nodes_length,
            current_collapsable_node_index: 0
        }
    }
    fn try_increment_current_collapsable_node_state(&mut self) -> bool {
        let current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
        current_collapsable_node.node_state_indexed_view.try_move_next()
    }
    fn alter_reference_to_current_collapsable_node_mask(&mut self) {
        let current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
        let current_possible_state: &str = current_collapsable_node.node_state_indexed_view.get();
        current_collapsable_node.neighbor_mask_mapped_view.borrow_mut().orient(current_possible_state);
    }
    fn is_at_least_one_neighbor_fully_restricted(&self) -> bool {
        let current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let mut is_at_least_one_neighbor_fully_restricted = false;

        {
            for collapsable_node in self.collapsable_nodes.iter() {
                let possible_neighbor_node_id: &str = collapsable_node.id;
                if current_collapsable_node.neighbor_node_ids.contains(&possible_neighbor_node_id) {
                    if collapsable_node.is_fully_restricted() {
                        is_at_least_one_neighbor_fully_restricted = true;
                        break;
                    }
                }
            }
        }

        is_at_least_one_neighbor_fully_restricted
    }
    fn move_to_next_collapsable_node(&mut self) {
        self.current_collapsable_node_index += 1;
    }
    fn is_fully_collapsed(&self) -> bool {
        self.current_collapsable_node_index == self.collapsable_nodes_length
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
                    comparison = a.random_sort_index.cmp(&b.random_sort_index);
                }
                else {
                    comparison = std::cmp::Ordering::Less
                }
            }
            comparison
        });
    }
    fn get_current_collapsable_node_id(&self) -> &str {
        let current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).expect("The collapsable node index should be within range.");
        current_collapsable_node.id
    }
    fn is_current_collapsable_node_the_first_node(&self) -> bool {
        self.current_collapsable_node_index == 0
    }
    fn reset_current_collapsable_node_state(&mut self) {
        let current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node index should be within range.");
        current_collapsable_node.node_state_indexed_view.reset();
    }
    fn move_to_previous_collapsable_node(&mut self) {
        self.current_collapsable_node_index -= 1;
    }
    fn try_move_to_previous_collapsable_node_neighbor(&mut self) -> bool {
        let original_collapsable_node_id = self.collapsable_nodes.get(self.current_collapsable_node_index).expect("The collapsable node index should be within range.").id;
        while self.current_collapsable_node_index != 0 {
            self.current_collapsable_node_index -= 1;
            if self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap().neighbor_node_ids.contains(&original_collapsable_node_id) {
                break;
            }
        }
        self.current_collapsable_node_index != 0
    }
    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction {
        let mut node_state_per_node: HashMap<String, String> = HashMap::new();
        for collapsable_node in self.collapsable_nodes.iter() {
            let node_state: String = String::from(*collapsable_node.node_state_indexed_view.get());
            let node: String = String::from(collapsable_node.id);
            node_state_per_node.insert(node, node_state);
        }
        CollapsedWaveFunction { node_state_per_node: node_state_per_node }
    }
}

pub struct WaveFunction {
    nodes: Vec<Node>,
    node_state_collections: Vec<NodeStateCollection>,
}

impl WaveFunction {
    pub fn new(nodes: Vec<Node>, node_state_collections: Vec<NodeStateCollection>) -> Result<Self, String> {
        if nodes.len() < 2 {
            Err(String::from("There must be two or more nodes."))
        }
        else {
            Ok(WaveFunction {
                nodes: nodes,
                node_state_collections: node_state_collections,
            })
        }
    }
    pub fn validate(&self) -> Result<(), String> {
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

        if error_message.is_some() {
            Err(error_message.unwrap())
        }
        else {
            Ok(())
        }
    }
    pub fn collapse(&self) -> Result<CollapsedWaveFunction, String> {
        let mut node_per_id: HashMap<&str, &Node> = HashMap::new();
        self.nodes.iter().for_each(|node: &Node| {
            node_per_id.insert(&node.id, node);
        });
        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection> = HashMap::new();
        self.node_state_collections.iter().for_each(|node_state_collection| {
            node_state_collection_per_id.insert(&node_state_collection.id, node_state_collection);
        });

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
                let node = node_per_id.get(neighbor_node_id).expect("The neighbor should exist in the complete list of nodes.");
                let neighbor_node_state_ids = node.get_possible_node_states(&node_state_collection_per_id).expect("The neighbor node should have some node states.");

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
                masks.push(mask.clone());  // create another owner
                neighbor_mask_mapped_view_per_node_id.insert(neighbor_node_id, mask);
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

            let collapsable_node = CollapsableNode::new(node, &node_per_id, &node_state_collection_per_id, neighber_masked_mapped_view, node_state_indexed_view);
            collapsable_nodes.push(collapsable_node);
            collapsable_node_index_per_node_id.insert(&node.id, collapsable_node_index.clone());
            collapsable_node_index = collapsable_node_index + 1;
        }

        // TODO use the provided cells in cell_per_neighbor_node_id_per_node_id during construction of CollapsableNode

        // set sort necessary
        // set error message as None
        // while
        //          no error message
        //          and
        //          the collapsable node index is less than the total number of collapsable nodes
        //
        // REMOVE      if current collapsable node has Some state id index
        // REMOVE          inform neighbors that this state id is now available again (if applicable)
        //
        //      try to increment the current collapsable node state id index (maybe just going from None to Some(0))
        //
        //      if succeeded to increment
        //          alter reference to mask (via orient function)
        //          if not at least one neighbor no longer having any valid states (or better said if all neighbors have at least one valid state)
        //              increment current collapsable node index
        //              if node index is not outside of the bounds
        //                  sort by (1) chosen from sorted collapsable nodes vector index (in order to maintain the chosen order) and then (2) least possible states being first (in order to adjust the next possible nodes to pick the most restricted nodes first)
        //      else (then we need to try a different state for the most recent parent that has the current node as a neighbor)
        //          set is neighbor found to false
        //          cache the current collapsable node id
        //          while
        //                  not yet errored
        //                  and
        //                  not found neighbor
        //
        //              if current collapsable node index is the first node (then the nodes have been exhausted)
        //                  set error message
        //              else
        //                  set current collapsable node's state id index to None (via reset function)
        //                  decrement current collapsale node index
        //                  if one of the newly current collapsable node's neighbors is the original collapsable node
        //                      set found neighbor to true

        let mut collapsable_wave_function = CollapsableWaveFunction::new(collapsable_nodes);

        let mut is_unable_to_collapse = false;
        while !is_unable_to_collapse && !collapsable_wave_function.is_fully_collapsed() {
            if collapsable_wave_function.try_increment_current_collapsable_node_state() {
                collapsable_wave_function.alter_reference_to_current_collapsable_node_mask();
                if !collapsable_wave_function.is_at_least_one_neighbor_fully_restricted() {
                    collapsable_wave_function.move_to_next_collapsable_node();
                    if !collapsable_wave_function.is_fully_collapsed() {
                        collapsable_wave_function.sort_collapsable_nodes();
                    }
                }
            }
            else {
                if !collapsable_wave_function.try_move_to_previous_collapsable_node_neighbor() {
                    is_unable_to_collapse = true;
                }
            }
        }

        if is_unable_to_collapse {
            Err(String::from("Cannot collapse wave function."))
        }
        else {
            let collapsed_wave_function = collapsable_wave_function.get_collapsed_wave_function();
            Ok(collapsed_wave_function)
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use std::hash::Hash;

    use super::*;

    #[test]
    fn initialize() {
        let nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();
        let wave_function = WaveFunction::new(nodes, node_state_collections);
    }

    #[test]
    fn no_nodes() {
        let nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();
        let wave_function_result = WaveFunction::new(nodes, node_state_collections);

        assert_eq!("There must be two or more nodes.", wave_function_result.err().unwrap());
    }

    #[test]
    fn one_node() {
        let mut nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let wave_function_result = WaveFunction::new(nodes, node_state_collections);

        assert_eq!("There must be two or more nodes.", wave_function_result.err().unwrap());
    }

    #[test]
    fn two_nodes_without_neighbors() {
        let mut nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let wave_function_result = WaveFunction::new(nodes, node_state_collections);

        assert_eq!("There must be two or more nodes.", wave_function_result.err().unwrap());
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor() {
        let mut nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let node_id: String = nodes[1].id.clone();
        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(node_id.clone(), Vec::new());
        let node_state_id: String = Uuid::new_v4().to_string();
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&node_id).unwrap().push(node_state_id);

        let wave_function_result = WaveFunction::new(nodes, node_state_collections);

        assert_eq!("There must be two or more nodes.", wave_function_result.err().unwrap());
    }

    #[test]
    fn two_nodes_both_as_neighbors() {
        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let node_state_id: String = Uuid::new_v4().to_string();
        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let same_node_state_collection_id: String = Uuid::new_v4().to_string();
        let same_node_state_collection = NodeStateCollection {
            id: same_node_state_collection_id.clone(),
            node_state_id: node_state_id.clone(),
            node_state_ids: vec![node_state_id.clone()]
        };
        node_state_collections.push(same_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(same_node_state_collection_id.clone());

        let wave_function_result = WaveFunction::new(nodes, node_state_collections);

        let wave_function = wave_function_result.ok().unwrap();

        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.collapse();

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());

    }
}