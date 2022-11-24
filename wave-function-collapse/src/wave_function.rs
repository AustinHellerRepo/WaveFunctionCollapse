use std::{collections::{HashMap, HashSet, VecDeque}, cell::{RefCell}, rc::Rc, fmt::Display, hash::Hash};
use serde::{Deserialize, Serialize};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use bitvec::prelude::*;
use uuid::Uuid;
use log::debug;
extern crate pretty_env_logger;

mod indexed_view;
use self::indexed_view::IndexedView;
mod probability_collection;
use self::probability_collection::ProbabilityCollection;
mod probability_tree;
use self::probability_tree::ProbabilityTree;
mod probability_container;
use self::probability_container::ProbabilityContainer;
mod tests;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub node_state_probability_per_node_state_id: HashMap<String, f32>,
    pub node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>>,
    pub node_state_ids: Vec<String>
}

impl Node {
    pub fn new(id: String, node_state_probability_per_node_state_id: HashMap<String, f32>, node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>>) -> Self {
        let mut node_state_ids: Vec<String> = Vec::new();
        for node_state_id in node_state_probability_per_node_state_id.keys() {
            node_state_ids.push(node_state_id.clone());
        }
        Node {
            id: id,
            node_state_probability_per_node_state_id: node_state_probability_per_node_state_id,
            node_state_collection_ids_per_neighbor_node_id: node_state_collection_ids_per_neighbor_node_id,
            node_state_ids: node_state_ids
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeStateCollection {
    pub id: String,
    pub node_state_id: String,
    pub node_state_ids: Vec<String>
}

impl NodeStateCollection {
    pub fn new(id: String, node_state_id: String, node_state_ids: Vec<String>) -> Self {
        NodeStateCollection {
            id: id,
            node_state_id: node_state_id,
            node_state_ids: node_state_ids
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct CollapsedNodeState {
    pub node_id: String,
    pub node_state_id: Option<String>
}

#[derive(Debug)]
struct CollapsableNode<'a> {
    // the node id that this collapsable node refers to
    id: &'a str,
    // this nodes list of neighbor node ids
    neighbor_node_ids: Vec<&'a str>,
    // the full list of possible node states, masked by internal references to neighbor masks
    node_state_indexed_view: IndexedView<&'a str>,
    // the mapped view that this node's neighbors will have a reference to and pull their masks from
    mask_per_neighbor_per_state: HashMap<&'a str, HashMap<&'a str, BitVec>>,
    // the index of traversed nodes based on the sorted vector of nodes as they are chosen for state determination
    current_chosen_from_sort_index: Option<usize>
}

impl<'a> CollapsableNode<'a> {
    #[time_graph::instrument]
    fn new(node: &'a Node, mask_per_neighbor_per_state: HashMap<&'a str, HashMap<&'a str, BitVec>>, node_state_indexed_view: IndexedView<&'a str>) -> CollapsableNode<'a> {
        // get the neighbors for this node
        let mut neighbor_node_ids: Vec<&str> = Vec::new();

        for neighbor_node_id_string in node.node_state_collection_ids_per_neighbor_node_id.keys() {
            let neighbor_node_id: &str = neighbor_node_id_string;
            neighbor_node_ids.push(neighbor_node_id);
        }
        neighbor_node_ids.sort();

        CollapsableNode {
            id: &node.id,
            neighbor_node_ids: neighbor_node_ids,
            node_state_indexed_view: node_state_indexed_view,
            mask_per_neighbor_per_state: mask_per_neighbor_per_state,
            current_chosen_from_sort_index: None
        }
    }
    #[time_graph::instrument]
    fn randomize<R: Rng + ?Sized>(&mut self, random_instance: &mut R) {
        self.node_state_indexed_view.shuffle(random_instance);
    }
    #[time_graph::instrument]
    fn is_fully_restricted(&mut self) -> bool {
        self.node_state_indexed_view.is_fully_restricted() || self.node_state_indexed_view.is_current_state_restricted()
    }
    #[time_graph::instrument]
    fn get_ids(collapsable_nodes: &Vec<Rc<RefCell<Self>>>) -> String {
        let mut string_builder = string_builder::Builder::new(0);
        for collapsable_node in collapsable_nodes.iter() {
            let node_id: &str = collapsable_node.borrow().id;
            if string_builder.len() != 0 {
                string_builder.append(", ");
            }
            string_builder.append(node_id);
        }
        string_builder.string().unwrap()
    }
    #[time_graph::instrument]
    fn add_mask(&mut self, mask: &BitVec) {
        self.node_state_indexed_view.add_mask(mask);
    }
    #[time_graph::instrument]
    fn subtract_mask(&mut self, mask: &BitVec) {
        self.node_state_indexed_view.subtract_mask(mask);
    }
}

impl<'a> Display for CollapsableNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Serialize)]
pub struct CollapsedWaveFunction {
    pub node_state_per_node: HashMap<String, String>
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UncollapsedWaveFunction {
    pub node_state_per_node: HashMap<String, Option<String>>
}

impl Hash for UncollapsedWaveFunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for property in self.node_state_per_node.iter() {
            property.hash(state);
        }
    }
}

pub struct CollapsableWaveFunction<'a> {
    // represents a wave function with all of the necessary steps to collapse
    collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a>>>>,
    collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a>>>>,
    collapsable_nodes_length: usize,
    current_collapsable_node_index: usize
}

impl<'a> CollapsableWaveFunction<'a> {
    #[time_graph::instrument]
    fn new(collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a>>>>, collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a>>>>) -> Self {
        let collapsable_nodes_length: usize = collapsable_nodes.len();

        let mut collapsable_wave_function = CollapsableWaveFunction {
            collapsable_nodes: collapsable_nodes,
            collapsable_node_per_id: collapsable_node_per_id,
            collapsable_nodes_length: collapsable_nodes_length,
            current_collapsable_node_index: 0
        };

        collapsable_wave_function
    }
    #[time_graph::instrument]
    fn revert_existing_neighbor_masks(&mut self) {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        if let Some(current_possible_state) = current_collapsable_node.node_state_indexed_view.get() {
            // if there is a mask_per_neighbor for this node's current state
            if let Some(mask_per_neighbor) = current_collapsable_node.mask_per_neighbor_per_state.get(current_possible_state) {
                for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                    if mask_per_neighbor.contains_key(neighbor_node_id) {
                        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                        let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                        neighbor_collapsable_node.subtract_mask(mask);  // TODO make each node contain a memo structure such that the masks are not needed to revert to the previous state since subtractions happen in reverse order anyway
                    }
                }
            }
        }
    }
    #[time_graph::instrument]
    fn try_increment_current_collapsable_node_state(&mut self) -> CollapsedNodeState {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

        let is_successful = current_collapsable_node.node_state_indexed_view.try_move_next();
        let collapsed_node_state: CollapsedNodeState;
        if is_successful {
            current_collapsable_node.current_chosen_from_sort_index = Some(self.current_collapsable_node_index);
            collapsed_node_state = CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: Some(String::from(*current_collapsable_node.node_state_indexed_view.get().unwrap()))
            };
        }
        else {
            current_collapsable_node.current_chosen_from_sort_index = None;
            collapsed_node_state = CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: None
            };
        }

        collapsed_node_state
    }
    #[time_graph::instrument]
    fn alter_reference_to_current_collapsable_node_mask(&mut self) {
        let neighbor_node_ids: &Vec<&str>;
        let mask_per_neighbor_per_state: &HashMap<&str, HashMap<&str, BitVec>>;
        let wrapped_current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        if let Some(current_possible_state) = current_collapsable_node.node_state_indexed_view.get() {
            neighbor_node_ids = &current_collapsable_node.neighbor_node_ids;
            mask_per_neighbor_per_state = &current_collapsable_node.mask_per_neighbor_per_state;
            if let Some(mask_per_neighbor) = mask_per_neighbor_per_state.get(current_possible_state) {
                for neighbor_node_id in neighbor_node_ids.iter() {
                    if mask_per_neighbor.contains_key(neighbor_node_id) {
                        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                        //debug!("looking for mask from parent {:?} to child {:?}.", current_collapsable_node.id, neighbor_node_id);
                        //debug!("mask_per_neighbor: {:?}", mask_per_neighbor);
                        let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                        neighbor_collapsable_node.add_mask(mask);
                    }
                }
            }
        }
    }
    #[time_graph::instrument]
    fn is_at_least_one_neighbor_fully_restricted(&self) -> bool {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        let mut is_at_least_one_neighbor_fully_restricted = false;

        {
            for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                if self.collapsable_node_per_id.get(neighbor_node_id).unwrap().borrow_mut().is_fully_restricted() {
                    is_at_least_one_neighbor_fully_restricted = true;
                    break;
                }
            }
        }

        is_at_least_one_neighbor_fully_restricted
    }
    #[time_graph::instrument]
    fn move_to_next_collapsable_node(&mut self) {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let current_node_id: &str = wrapped_current_collapsable_node.borrow().id;
        let current_collapsable_node_index: &usize = &self.current_collapsable_node_index;
        debug!("moving from {current_node_id} at index {current_collapsable_node_index}");

        self.current_collapsable_node_index += 1;

        let next_collapsable_node_index: &usize = &self.current_collapsable_node_index;
        if self.current_collapsable_node_index == self.collapsable_nodes_length {
            debug!("moved outside of bounds at index {next_collapsable_node_index}");
        }
        else {
            let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
            let next_node_id: &str = wrapped_current_collapsable_node.borrow().id;
            debug!("moved to {next_node_id} at index {next_collapsable_node_index}");
        }
    }
    #[time_graph::instrument]
    fn is_fully_collapsed(&self) -> bool {
        self.current_collapsable_node_index == self.collapsable_nodes_length
    }
    #[time_graph::instrument]
    fn try_move_to_previous_collapsable_node_neighbor(&mut self) {

        let neighbor_node_ids: &Vec<&str>;
        let mask_per_neighbor_per_state: &HashMap<&str, HashMap<&str, BitVec>>;
        let wrapped_current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
        let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();
        if let Some(current_possible_state) = current_collapsable_node.node_state_indexed_view.get() {
            neighbor_node_ids = &current_collapsable_node.neighbor_node_ids;
            mask_per_neighbor_per_state = &current_collapsable_node.mask_per_neighbor_per_state;
            let mask_per_neighbor = mask_per_neighbor_per_state.get(current_possible_state).unwrap();
            for neighbor_node_id in neighbor_node_ids.iter() {
                let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                neighbor_collapsable_node.subtract_mask(mask);
            }
        }

        // reset the node state index for the current node
        current_collapsable_node.node_state_indexed_view.reset();
        // reset chosen index within collapsable node
        current_collapsable_node.current_chosen_from_sort_index = None;
        
        // move to the previously chosen node
        if self.current_collapsable_node_index != 0 {
            self.current_collapsable_node_index -= 1;
        }
            
    }
    #[time_graph::instrument]
    fn is_fully_reset(&self) -> bool {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        self.current_collapsable_node_index == 0 && current_collapsable_node.current_chosen_from_sort_index.is_none()
    }
    fn get_uncollapsed_wave_function(&self) -> UncollapsedWaveFunction {
        let mut node_state_per_node: HashMap<String, Option<String>> = HashMap::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            let node_state_id_option: Option<String>;
            if let Some(node_state_id) = collapsable_node.node_state_indexed_view.get() {
                node_state_id_option = Some(String::from(*node_state_id));
            }
            else {
                node_state_id_option = None;
            }
            let node: String = String::from(collapsable_node.id);
            debug!("established node {node} in state {:?}.", node_state_id_option);
            node_state_per_node.insert(node, node_state_id_option);
        }
        UncollapsedWaveFunction {
            node_state_per_node: node_state_per_node
        }
    }
    #[time_graph::instrument]
    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction {
        let mut node_state_per_node: HashMap<String, String> = HashMap::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            let node_state: String = String::from(*collapsable_node.node_state_indexed_view.get().unwrap());
            let node: String = String::from(collapsable_node.id);
            debug!("established node {node} in state {node_state}.");
            node_state_per_node.insert(node, node_state);
        }
        CollapsedWaveFunction {
            node_state_per_node: node_state_per_node
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WaveFunction {
    nodes: Vec<Node>,
    node_state_collections: Vec<NodeStateCollection>
}

impl WaveFunction {
    pub fn new(nodes: Vec<Node>, node_state_collections: Vec<NodeStateCollection>) -> Self {
        WaveFunction {
            nodes: nodes,
            node_state_collections: node_state_collections
        }
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }

    pub fn get_node_state_collections(&self) -> Vec<NodeStateCollection> {
        self.node_state_collections.clone()
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

    pub fn optimize(&mut self) {
        //let current_collapsable_nodes_display = CollapsableNode::get_ids(&self.collapsable_nodes);
        //debug!("current sort order: {current_collapsable_nodes_display}.");

        // sort by most neighbors
        {
            self.nodes.sort_by(|a, b| {

                let a_neighbors_length = a.get_neighbor_node_ids().len();
                let b_neighbors_length = b.get_neighbor_node_ids().len();

                a_neighbors_length.cmp(&b_neighbors_length)
            });

            let mut next_node_index_per_node_id: HashMap<String, usize> = HashMap::new();

            {
                let mut found_neighbor_node_ids: HashSet<String> = HashSet::new();
                let mut searching_neighbor_node_ids: VecDeque<String> = VecDeque::new();
                searching_neighbor_node_ids.push_back(self.nodes.first().unwrap().id.clone());

                let mut node_per_id: HashMap<String, &Node> = HashMap::new();
                for node in self.nodes.iter() {
                    node_per_id.insert(node.id.clone(), node);
                }

                let mut node_index_per_id: HashMap<&str, usize> = HashMap::new();
                for (node_index, node) in self.nodes.iter().enumerate() {
                    node_index_per_id.insert(&node.id, node_index);
                }

                let mut next_node_index: usize = 0;
                while !searching_neighbor_node_ids.is_empty() {
                    let searching_neighbor_node_id = searching_neighbor_node_ids.pop_front().unwrap();
                    debug!("searching: {:?}", searching_neighbor_node_ids);

                    next_node_index_per_node_id.insert(searching_neighbor_node_id.clone(), next_node_index);
                    next_node_index += 1;

                    found_neighbor_node_ids.insert(searching_neighbor_node_id.clone());
                    let neighbor = node_per_id.get(&searching_neighbor_node_id).unwrap();
                    for neighbors_neighbor_node_id in neighbor.node_state_collection_ids_per_neighbor_node_id.keys() {
                        let neighbors_neighbor_node_id: &str = neighbors_neighbor_node_id;
                        if !found_neighbor_node_ids.contains(neighbors_neighbor_node_id) {
                            debug!("adding potential neighbor: {neighbors_neighbor_node_id}");
                            searching_neighbor_node_ids.push_back(String::from(neighbors_neighbor_node_id));
                        }
                    }
                }
            }
            
            self.nodes.sort_by(|a, b| {

                let a_node_index = next_node_index_per_node_id.get(&a.id);
                let b_node_index = next_node_index_per_node_id.get(&b.id);

                a_node_index.cmp(&b_node_index)
            });
        }

        //let next_collapsable_nodes_display = CollapsableNode::get_ids(&self.collapsable_nodes);
        //debug!("next sort order: {next_collapsable_nodes_display}.");
    }

    #[time_graph::instrument]
    fn get_collapsable_wave_function(&self, random_seed: Option<u64>) -> CollapsableWaveFunction {
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

        // neighbor_mask_mapped_view_per_node_id is equivalent to mask_per_child_neighbor_per_state_per_node
        let mut neighbor_mask_mapped_view_per_node_id: HashMap<&str, HashMap<&str, HashMap<&str, BitVec>>> = HashMap::new();

        time_graph::spanned!("creating masks for nodes", {

            // create, per parent neighbor, a mask for each node (as child of parent neighbor)
            let mut mask_per_parent_state_per_parent_neighbor_per_node: HashMap<&str, HashMap<&str, HashMap<&str, BitVec>>> = HashMap::new();
            // for each node
            for child_node in self.nodes.iter() {

                let mut mask_per_parent_state_per_parent_neighbor: HashMap<&str, HashMap<&str, BitVec>> = HashMap::new();

                // look for each parent neighbor node
                for parent_neighbor_node in self.nodes.iter() {
                    // if you find that this is a parent neighbor node
                    if parent_neighbor_node.node_state_collection_ids_per_neighbor_node_id.contains_key(&child_node.id) {

                        debug!("constructing mask for {:?}'s child node {:?}.", parent_neighbor_node.id, child_node.id);

                        let mut mask_per_parent_state: HashMap<&str, BitVec> = HashMap::new();

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

                let mut mask_per_neighbor_per_state: HashMap<&str, HashMap<&str, BitVec>> = HashMap::new();

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
        });

        let mut node_state_indexed_view_per_node_id: HashMap<&str, IndexedView<&str>> = HashMap::new();

        time_graph::spanned!("storing masks into neighbors", {

            // store all of the masks that my neighbors will be orienting so that this node can check for restrictions
            for node in self.nodes.iter() {
                let node_id: &str = &node.id;

                //debug!("storing for node {node_id} restrictive masks into node state indexed view.");

                let mut node_state_ids: Vec<&str> = Vec::new();
                for node_state_id_string in node.node_state_ids.iter() {
                    let node_state_id: &str = node_state_id_string;
                    node_state_ids.push(node_state_id);
                }

                let node_state_indexed_view = IndexedView::new(node_state_ids);
                //debug!("stored for node {node_id} node state indexed view {:?}", node_state_indexed_view);
                node_state_indexed_view_per_node_id.insert(node_id, node_state_indexed_view);
            }
        });

        let mut random_instance: Option<ChaCha8Rng> = None;

        let mut collapsable_nodes: Vec<Rc<RefCell<CollapsableNode>>> = Vec::new();
        let mut collapsable_node_per_id: HashMap<&str, Rc<RefCell<CollapsableNode>>> = HashMap::new();
        // contains the mask to apply to the neighbor when this node is in a specific state
        for (node_index, node) in self.nodes.iter().enumerate() {
            let node_id: &str = &node.id;

            let node_state_indexed_view: IndexedView<&str> = node_state_indexed_view_per_node_id.remove(node_id).unwrap();
            let mask_per_neighbor_per_state = neighbor_mask_mapped_view_per_node_id.remove(node_id).unwrap();

            let mut collapsable_node = CollapsableNode::new(node, mask_per_neighbor_per_state, node_state_indexed_view);

            if let Some(seed) = random_seed {
                if random_instance.is_none() {
                    let seed_offset: u64 = node_index as u64;
                    random_instance = Some(ChaCha8Rng::seed_from_u64(seed + seed_offset));
                }
                collapsable_node.randomize(random_instance.as_mut().unwrap());
            }

            collapsable_nodes.push(Rc::new(RefCell::new(collapsable_node)));
        }

        for wrapped_collapsable_node in collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            collapsable_node_per_id.insert(collapsable_node.id.clone(), wrapped_collapsable_node.clone());
        }

        CollapsableWaveFunction::new(collapsable_nodes, collapsable_node_per_id)
    }

    #[time_graph::instrument]
    pub fn collapse_into_steps(&self, random_seed: Option<u64>) -> Result<Vec<CollapsedNodeState>, String> {
        let mut collapsed_node_states: Vec<CollapsedNodeState> = Vec::new();

        let mut collapsable_wave_function = self.get_collapsable_wave_function(random_seed);

        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !collapsable_wave_function.is_fully_collapsed() {
            time_graph::spanned!("is_increment_successful", {
                collapsable_wave_function.revert_existing_neighbor_masks();
            });
            debug!("incrementing node state");
            let collapsed_node_state = collapsable_wave_function.try_increment_current_collapsable_node_state();
            let is_successful: bool = collapsed_node_state.node_state_id.is_some();
            collapsed_node_states.push(collapsed_node_state);

            debug!("stored node state");
            if is_successful {
                debug!("incremented node state: {:?}", collapsed_node_states.last());
                collapsable_wave_function.alter_reference_to_current_collapsable_node_mask();
                debug!("altered reference");
                if !collapsable_wave_function.is_at_least_one_neighbor_fully_restricted() {
                    debug!("all neighbors have at least one valid state");
                    collapsable_wave_function.move_to_next_collapsable_node(); // this has the potential to move outside of the bounds and put the collapsable wave function in a state of being fully collapsed
                    debug!("moved to next collapsable node");
                    if !collapsable_wave_function.is_fully_collapsed() {
                        debug!("not yet fully collapsed");
                        //collapsable_wave_function.sort_collapsable_nodes();
                        //debug!("sorted nodes");
                    }
                }
                else {
                    debug!("at least one neighbor is fully restricted");
                }
            }
            else {
                debug!("failed to incremented node");
                collapsable_wave_function.try_move_to_previous_collapsable_node_neighbor();

                if collapsable_wave_function.is_fully_reset() {
                    debug!("moved back to first node and reset it");
                    is_unable_to_collapse = true;
                }
                else {
                    debug!("moved back to previous neighbor");
                    //collapsable_wave_function.alter_reference_to_current_collapsable_node_mask();
                    //debug!("stored uncollapsed_wave_function state");
                }
            }
        }
        debug!("finished while loop");

        Ok(collapsed_node_states)
    }

    #[time_graph::instrument]
    pub fn collapse(&self, random_seed: Option<u64>) -> Result<CollapsedWaveFunction, String> {

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

        let mut collapsable_wave_function = self.get_collapsable_wave_function(random_seed);

        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !collapsable_wave_function.is_fully_collapsed() {
            time_graph::spanned!("is_increment_successful", {
                collapsable_wave_function.revert_existing_neighbor_masks();
            });
            debug!("incrementing node state");
            let is_increment_successful: bool;
            time_graph::spanned!("is_increment_successful", {
                is_increment_successful = collapsable_wave_function.try_increment_current_collapsable_node_state().node_state_id.is_some();
            });
            if is_increment_successful {
                debug!("incremented node state");
                time_graph::spanned!("alter_reference_to_current_collapsable_node_mask", {
                    collapsable_wave_function.alter_reference_to_current_collapsable_node_mask();
                });
                debug!("altered reference");
                let is_at_least_one_neighbor_fully_restricted: bool;
                time_graph::spanned!("is_at_least_one_neighbor_fully_restricted", {
                    is_at_least_one_neighbor_fully_restricted = collapsable_wave_function.is_at_least_one_neighbor_fully_restricted();
                });
                if !is_at_least_one_neighbor_fully_restricted {
                    debug!("all neighbors have at least one valid state");
                    time_graph::spanned!("move_to_next_collapsable_node", {
                        collapsable_wave_function.move_to_next_collapsable_node();
                    });
                    debug!("moved to next collapsable node");
                    let is_fully_collapsed: bool;
                    time_graph::spanned!("is_fully_collapsed", {
                        is_fully_collapsed = collapsable_wave_function.is_fully_collapsed();
                    });
                    if !is_fully_collapsed {
                        debug!("not yet fully collapsed");
                        /*time_graph::spanned!("sort_collapsable_nodes (during)", {
                            collapsable_wave_function.sort_collapsable_nodes();
                        });
                        debug!("sorted nodes");*/
                    }
                }
                else {
                    debug!("at least one neighbor is fully restricted");
                }
            }
            else {
                debug!("failed to incremented node");
                time_graph::spanned!("try_move_to_previous_collapsable_node_neighbor", {
                    collapsable_wave_function.try_move_to_previous_collapsable_node_neighbor();
                });
                let is_fully_reset: bool;
                time_graph::spanned!("is_fully_reset", {
                    is_fully_reset = collapsable_wave_function.is_fully_reset();
                });
                if is_fully_reset {
                    debug!("moved back to first node");
                    is_unable_to_collapse = true;
                }
                else {
                    debug!("moved back to previous neighbor");
                }
            }
        }
        debug!("finished while loop");

        if is_unable_to_collapse {
            Err(String::from("Cannot collapse wave function."))
        }
        else {
            let collapsed_wave_function = collapsable_wave_function.get_collapsed_wave_function();
            Ok(collapsed_wave_function)
        }
    }

    #[time_graph::instrument]
    pub fn save_to_file(&self, file_path: &str) {
        let serialized_self = serde_json::to_string(self).unwrap();
        std::fs::write(file_path, serialized_self).unwrap();
    }

    #[time_graph::instrument]
    pub fn load_from_file(file_path: &str) -> Self {
        let serialized_self = std::fs::read_to_string(file_path).unwrap();
        let deserialized_self: WaveFunction = serde_json::from_str(&serialized_self).unwrap();
        deserialized_self
    }
}
