use std::{collections::{HashMap, HashSet}, cell::{RefCell}, rc::Rc, hash::Hash, fs::File, io::BufReader};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use bitvec::prelude::*;
use log::debug;
extern crate pretty_env_logger;
mod indexed_view;
use crate::wave_function::collapsable_wave_function::collapsable_wave_function::CollapsableNode;
use self::{indexed_view::IndexedView, collapsable_wave_function::collapsable_wave_function::{CollapsableWaveFunction}};
mod probability_collection;
mod probability_tree;
mod probability_container;
pub mod collapsable_wave_function;
mod tests;

/// This struct makes for housing convenient utility functions.
pub struct NodeStateProbability;

impl NodeStateProbability {
    pub fn get_equal_probability(node_state_ids: Vec<String>) -> HashMap<String, f32> {
        let mut node_state_probability_per_node_state_id: HashMap<String, f32> = HashMap::new();

        for node_state_id in node_state_ids.into_iter() {
            node_state_probability_per_node_state_id.insert(node_state_id, 1.0);
        }

        node_state_probability_per_node_state_id
    }
}

/// This is a node in the graph of the wave function. It can be in any of the provided node states, trying to achieve the cooresponding probability, connected to other nodes as described by the node state collections.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub id: String,
    pub node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>>,
    pub node_state_ids: Vec<TNodeState>,
    pub node_state_probabilities: Vec<f32>
}

impl<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> Node<TNodeState> {
    pub fn new(id: String, node_state_probability_per_node_state_id: HashMap<TNodeState, f32>, node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>>) -> Self {
        let mut node_state_ids: Vec<TNodeState> = Vec::new();
        let mut node_state_probabilities: Vec<f32> = Vec::new();
        for (node_state_id, node_state_probability) in node_state_probability_per_node_state_id.iter() {
            node_state_ids.push(node_state_id.clone());
            node_state_probabilities.push(*node_state_probability);
        }
        
        // sort the node_state_ids and node_state_probabilities
        let mut sort_permutation = permutation::sort(&node_state_ids);
        sort_permutation.apply_slice_in_place(&mut node_state_ids);
        sort_permutation.apply_slice_in_place(&mut node_state_probabilities);

        Node {
            id: id,
            node_state_collection_ids_per_neighbor_node_id: node_state_collection_ids_per_neighbor_node_id,
            node_state_ids: node_state_ids,
            node_state_probabilities: node_state_probabilities
        }
    }
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_neighbor_node_ids(&self) -> Vec<String> {
        let mut neighbor_node_ids: Vec<String> = Vec::new();
        for (neighbor_node_id, _) in self.node_state_collection_ids_per_neighbor_node_id.iter() {
            neighbor_node_ids.push(neighbor_node_id.clone());
        }
        neighbor_node_ids
    }
}

/// This struct represents a relationship between the state of one "original" node to another "neighbor" node, permitting only those node states for the connected neighbor if the original node is in the specific state. This defines the constraints between nodes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeStateCollection<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub id: String,
    pub node_state_id: TNodeState,
    pub node_state_ids: Vec<TNodeState>
}

impl<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> NodeStateCollection<TNodeState> {
    pub fn new(id: String, node_state_id: TNodeState, node_state_ids: Vec<TNodeState>) -> Self {
        NodeStateCollection {
            id: id,
            node_state_id: node_state_id,
            node_state_ids: node_state_ids
        }
    }
}

/// This struct represents the uncollapsed definition of nodes and their relationships to other nodes.
#[derive(Serialize, Clone, Deserialize)]
pub struct WaveFunction<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    nodes: Vec<Node<TNodeState>>,
    node_state_collections: Vec<NodeStateCollection<TNodeState>>
}

impl<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord + Serialize + DeserializeOwned> WaveFunction<TNodeState> {
    pub fn new(nodes: Vec<Node<TNodeState>>, node_state_collections: Vec<NodeStateCollection<TNodeState>>) -> Self {
        WaveFunction {
            nodes: nodes,
            node_state_collections: node_state_collections
        }
    }

    pub fn get_nodes(&self) -> Vec<Node<TNodeState>> {
        self.nodes.clone()
    }

    pub fn get_node_state_collections(&self) -> Vec<NodeStateCollection<TNodeState>> {
        self.node_state_collections.clone()
    }

    pub fn validate(&self) -> Result<(), String> {
        let nodes_length: usize = self.nodes.len();

        let mut node_per_id: HashMap<&str, &Node<TNodeState>> = HashMap::new();
        let mut node_ids: HashSet<&str> = HashSet::new();
        self.nodes.iter().for_each(|node: &Node<TNodeState>| {
            node_per_id.insert(&node.id, node);
            node_ids.insert(&node.id);
        });
        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection<TNodeState>> = HashMap::new();
        self.node_state_collections.iter().for_each(|node_state_collection| {
            node_state_collection_per_id.insert(&node_state_collection.id, node_state_collection);
        });

        let mut error_message = Option::None;
        
        // ensure that references neighbors are actually nodes
        for (_, node) in node_per_id.iter() {
            for (neighbor_node_id_string, _) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
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
            let mut at_least_one_node_connects_to_all_other_nodes: bool = false;
            for node in self.nodes.iter() {
                // ensure that all nodes connect to all other nodes
                let mut all_traversed_node_ids: HashSet<&str> = HashSet::new();
                let mut potential_node_ids: Vec<&str> = Vec::new();

                potential_node_ids.push(&node.id);

                while !potential_node_ids.is_empty() {
                    let node_id = potential_node_ids.pop().unwrap();
                    let node = node_per_id.get(node_id).unwrap();
                    for neighbor_node_id_string in node.node_state_collection_ids_per_neighbor_node_id.keys() {
                        let neighbor_node_id: &str = neighbor_node_id_string;
                        if !all_traversed_node_ids.contains(neighbor_node_id) && !potential_node_ids.contains(&neighbor_node_id) {
                            potential_node_ids.push(neighbor_node_id);
                        }
                    }
                    all_traversed_node_ids.insert(node_id);
                }

                let all_traversed_node_ids_length = all_traversed_node_ids.len();
                if all_traversed_node_ids_length == nodes_length {
                    at_least_one_node_connects_to_all_other_nodes = true;
                    break;
                }
            }

            if !at_least_one_node_connects_to_all_other_nodes {
                error_message = Some(format!("Not all nodes connect together. At least one node must be able to traverse to all other nodes."));
            }

            if error_message.is_none() {
                // TODO add more vaidation when needed
            }
        }

        if error_message.is_some() {
            Err(error_message.unwrap())
        }
        else {
            Ok(())
        }
    }

    pub fn get_collapsable_wave_function<'a, TCollapsableWaveFunction: CollapsableWaveFunction<'a, TNodeState>>(&'a self, random_seed: Option<u64>) -> TCollapsableWaveFunction {
        let mut node_per_id: HashMap<&str, &Node<TNodeState>> = HashMap::new();
        self.nodes.iter().for_each(|node: &Node<TNodeState>| {
            node_per_id.insert(&node.id, node);
        });
        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection<TNodeState>> = HashMap::new();
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

        // neighbor_mask_mapped_view_per_node_id is equivalent to mask_per_child_neighbor_per_state_per_node
        let mut neighbor_mask_mapped_view_per_node_id: HashMap<&str, HashMap<&TNodeState, HashMap<&str, BitVec>>> = HashMap::new();

        // create, per parent neighbor, a mask for each node (as child of parent neighbor)
        let mut mask_per_parent_state_per_parent_neighbor_per_node: HashMap<&str, HashMap<&str, HashMap<&TNodeState, BitVec>>> = HashMap::new();

        // for each node
        for child_node in self.nodes.iter() {

            let mut mask_per_parent_state_per_parent_neighbor: HashMap<&str, HashMap<&TNodeState, BitVec>> = HashMap::new();

            // look for each parent neighbor node
            for parent_neighbor_node in self.nodes.iter() {
                // if you find that this is a parent neighbor node
                if parent_neighbor_node.node_state_collection_ids_per_neighbor_node_id.contains_key(&child_node.id) {

                    debug!("constructing mask for {:?}'s child node {:?}.", parent_neighbor_node.id, child_node.id);

                    let mut mask_per_parent_state: HashMap<&TNodeState, BitVec> = HashMap::new();

                    // get the node state collections that this parent neighbor node forces upon this node
                    let node_state_collection_ids: &Vec<String> = parent_neighbor_node.node_state_collection_ids_per_neighbor_node_id.get(&child_node.id).unwrap();
                    for node_state_collection_id in node_state_collection_ids.iter() {
                        let node_state_collection_id: &str = node_state_collection_id;
                        let node_state_collection = node_state_collection_per_id.get(node_state_collection_id).unwrap();
                        // construct a mask for this parent neighbor's node state collection and node state for this child node
                        let mut mask: BitVec = BitVec::new();
                        for node_state_id in child_node.node_state_ids.iter() {
                            // if the node state for the child is permitted by the parent neighbor node state collection
                            if node_state_collection.node_state_ids.contains(node_state_id) {
                                mask.push(true);
                            }
                            else {
                                mask.push(false);
                            }
                        }
                        // store the mask for this child node
                        mask_per_parent_state.insert(&node_state_collection.node_state_id, mask);
                    }

                    mask_per_parent_state_per_parent_neighbor.insert(&parent_neighbor_node.id, mask_per_parent_state);
                }
            }

            mask_per_parent_state_per_parent_neighbor_per_node.insert(&child_node.id, mask_per_parent_state_per_parent_neighbor);
        }

        // fill the neighbor_mask_mapped_view_per_node_id now that all masks have been constructed
        // neighbor_mask_mapped_view_per_node_id is equivalent to mask_per_child_neighbor_per_state_per_node
        for node in self.nodes.iter() {

            // for this node, find all child neighbors
            let node_id: &str = &node.id;

            let mut mask_per_neighbor_per_state: HashMap<&TNodeState, HashMap<&str, BitVec>> = HashMap::new();

            for (neighbor_node_id, _) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
                let neighbor_node_id: &str = neighbor_node_id;

                // get the inverse hashmap of this node to its child neighbor
                let mask_per_parent_state_per_parent_neighbor = mask_per_parent_state_per_parent_neighbor_per_node.get(neighbor_node_id).unwrap();
                let mask_per_parent_state = mask_per_parent_state_per_parent_neighbor.get(node_id).unwrap();

                for (node_state_id, mask) in mask_per_parent_state.iter() {
                    if !mask_per_neighbor_per_state.contains_key(node_state_id) {
                        mask_per_neighbor_per_state.insert(node_state_id, HashMap::new());
                    }
                    mask_per_neighbor_per_state.get_mut(node_state_id).unwrap().insert(neighbor_node_id, mask.clone());
                }
            }

            neighbor_mask_mapped_view_per_node_id.insert(node_id, mask_per_neighbor_per_state);
        }

        let mut node_state_indexed_view_per_node_id: HashMap<&str, IndexedView<&TNodeState>> = HashMap::new();

        // store all of the masks that my neighbors will be orienting so that this node can check for restrictions
        for node in self.nodes.iter() {
            let node_id: &str = &node.id;

            //debug!("storing for node {node_id} restrictive masks into node state indexed view.");

            let referenced_node_state_ids: Vec<&TNodeState> = node.node_state_ids.iter().collect();
            let cloned_node_state_probabilities: Vec<f32> = node.node_state_probabilities.clone();

            let node_state_indexed_view = IndexedView::new(referenced_node_state_ids, cloned_node_state_probabilities);
            //debug!("stored for node {node_id} node state indexed view {:?}", node_state_indexed_view);
            node_state_indexed_view_per_node_id.insert(node_id, node_state_indexed_view);
        }

        let mut collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<TNodeState>>>> = Vec::new();
        let mut collapsable_node_per_id: HashMap<&str, Rc<RefCell<CollapsableNode<TNodeState>>>> = HashMap::new();
        // contains the mask to apply to the neighbor when this node is in a specific state
        for (node_index, node) in self.nodes.iter().enumerate() {
            let node_id: &str = &node.id;

            let node_state_indexed_view: IndexedView<&TNodeState> = node_state_indexed_view_per_node_id.remove(node_id).unwrap();
            let mask_per_neighbor_per_state = neighbor_mask_mapped_view_per_node_id.remove(node_id).unwrap();

            let mut collapsable_node = CollapsableNode::new(&node.id, &node.node_state_collection_ids_per_neighbor_node_id, mask_per_neighbor_per_state, node_state_indexed_view);

            if let Some(seed) = random_seed {
                let seed_offset: u64 = node_index as u64;
                let mut random_instance = ChaCha8Rng::seed_from_u64(seed + seed_offset);
                collapsable_node.randomize(&mut random_instance);
            }

            collapsable_nodes.push(Rc::new(RefCell::new(collapsable_node)));
        }

        for wrapped_collapsable_node in collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            collapsable_node_per_id.insert(&collapsable_node.id, wrapped_collapsable_node.clone());
        }

        for wrapped_collapsable_node in collapsable_nodes.iter() {
            let mut collapsable_node = wrapped_collapsable_node.borrow_mut();
            let collapsable_node_id: &str = collapsable_node.id;

            if mask_per_parent_state_per_parent_neighbor_per_node.contains_key(collapsable_node_id) {
                let mask_per_parent_state_per_parent_neighbor = mask_per_parent_state_per_parent_neighbor_per_node.get(collapsable_node_id).unwrap();
                for parent_neighbor_node_id in mask_per_parent_state_per_parent_neighbor.keys() {
                    collapsable_node.parent_neighbor_node_ids.push(parent_neighbor_node_id);
                }
            }
        }

        TCollapsableWaveFunction::new(collapsable_nodes, collapsable_node_per_id)
    }

    pub fn save_to_file(&self, file_path: &str) {
        let serialized_self = serde_json::to_string(self).unwrap();
        std::fs::write(file_path, serialized_self).unwrap();
    }

    pub fn load_from_file(file_path: &str) -> Self {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let deserialized_self: WaveFunction<TNodeState> = serde_json::from_reader(reader).unwrap();
        deserialized_self
    }
}
