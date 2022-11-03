use std::{collections::{HashMap, HashSet, BTreeSet}, cell::{Cell, RefCell}, rc::Rc, fmt::Display};
use serde::{Deserialize, Serialize};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use bitvec::prelude::*;
use uuid::Uuid;
use log::debug;
extern crate pretty_env_logger;

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
    // the full list of possible node states, masked by internal references to neighbor masks
    node_state_indexed_view: IndexedView<&'a str, &'a str, &'a str>,
    // the mapped view that this node's neighbors will have a reference to and pull their masks from
    neighbor_mask_mapped_view: Rc<RefCell<MappedView<&'a str, &'a str, BitVec>>>,
    // the index of traversed nodes based on the sorted vector of nodes as they are chosen for state determination
    current_chosen_from_sort_index: Option<usize>,
    // a random sort value for adding randomness to the process between runs (if randomized)
    random_sort_index: u32
}

impl<'a> CollapsableNode<'a> {
    fn new(node: &'a Node, neighbor_mask_mapped_view: Rc<RefCell<MappedView<&'a str, &'a str, BitVec>>>, node_state_indexed_view: IndexedView<&'a str, &'a str, &'a str>) -> CollapsableNode<'a> {
        // get the neighbors for this node
        let mut neighbor_node_ids: Vec<&str> = Vec::new();

        for neighbor_node_id_string in node.node_state_collection_ids_per_neighbor_node_id.keys() {
            let neighbor_node_id: &str = neighbor_node_id_string;
            neighbor_node_ids.push(neighbor_node_id);
        }

        CollapsableNode {
            id: &node.id,
            neighbor_node_ids: neighbor_node_ids,
            node_state_indexed_view: node_state_indexed_view,
            neighbor_mask_mapped_view: neighbor_mask_mapped_view,
            current_chosen_from_sort_index: None,
            random_sort_index: 0
        }
    }
    fn randomize<R: Rng + ?Sized>(&mut self, random_instance: &mut R) {
        self.neighbor_node_ids.shuffle(random_instance);
        self.node_state_indexed_view.shuffle(random_instance);
        self.random_sort_index = random_instance.next_u32();
    }
    fn is_fully_restricted(&self) -> bool {
        self.node_state_indexed_view.is_fully_restricted_or_current_state_is_restricted()
    }
    fn get_restriction_ratio(&self) -> f32 {
        if self.node_state_indexed_view.is_in_some_state() {
            1.0
        }
        else {
            self.node_state_indexed_view.get_restriction_ratio()
        }
    }
    fn get_ids(collapsable_nodes: &Vec<Self>) -> String {
        let mut string_builder = string_builder::Builder::new(0);
        for collapsable_node in collapsable_nodes.iter() {
            let node_id: &str = collapsable_node.id;
            if string_builder.len() != 0 {
                string_builder.append(", ");
            }
            string_builder.append(node_id);
        }
        string_builder.string().unwrap()
    }
}

impl<'a> Display for CollapsableNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
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
        let node_id: &str = current_collapsable_node.id;
        let current_state_id_option: Option<&&str> = current_collapsable_node.node_state_indexed_view.get();
        let current_state_id_display: String;
        if let Some(current_state_id) = current_state_id_option {
            current_state_id_display = String::from(*current_state_id);
        }
        else {
            current_state_id_display = String::from("None");
        }
        debug!("incrementing node {node_id} from {current_state_id_display}.");

        let is_successful = current_collapsable_node.node_state_indexed_view.try_move_next();
        if is_successful {
            current_collapsable_node.current_chosen_from_sort_index = Some(self.current_collapsable_node_index);
        }
        else {
            current_collapsable_node.current_chosen_from_sort_index = None;
        }

        let next_state_id_option: Option<&&str> = current_collapsable_node.node_state_indexed_view.get();
        let next_state_id_display: String;
        if let Some(next_state_id) = next_state_id_option {
            next_state_id_display = String::from(*next_state_id);
        }
        else {
            next_state_id_display = String::from("None");
        }
        debug!("incremented node {node_id} to {next_state_id_display}.");

        is_successful
    }
    fn alter_reference_to_current_collapsable_node_mask(&mut self) {
        let current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
        let current_possible_state: &str = current_collapsable_node.node_state_indexed_view.get().unwrap();
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
        let current_node_id: &str = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap().id;
        let current_collapsable_node_index: &usize = &self.current_collapsable_node_index;
        debug!("moving from {current_node_id} at index {current_collapsable_node_index}");

        self.current_collapsable_node_index += 1;

        let next_collapsable_node_index: &usize = &self.current_collapsable_node_index;
        if self.current_collapsable_node_index == self.collapsable_nodes_length {
            debug!("moved outside of bounds at index {next_collapsable_node_index}");
        }
        else {
            let next_node_id: &str = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap().id;
            debug!("moved to {next_node_id} at index {next_collapsable_node_index}");
        }
    }
    fn is_fully_collapsed(&self) -> bool {
        self.current_collapsable_node_index == self.collapsable_nodes_length
    }
    fn sort_collapsable_nodes(&mut self) {
        let current_collapsable_nodes_display = CollapsableNode::get_ids(&self.collapsable_nodes);
        debug!("current sort order: {current_collapsable_nodes_display}.");

        self.collapsable_nodes.sort_by(|a, b| {

            let a_node_id: &str = a.id;
            let b_node_id: &str = b.id;

            let comparison: std::cmp::Ordering;
            if let Some(a_chosen_from_sort_index) = a.current_chosen_from_sort_index {
                if let Some(b_chosen_from_sort_index) = b.current_chosen_from_sort_index {
                    comparison = a_chosen_from_sort_index.cmp(&b_chosen_from_sort_index);
                    match &comparison {
                        std::cmp::Ordering::Less => {
                            debug!("node {a_node_id} is less than node {b_node_id} after finding both have a chosen sort index.");
                        },
                        std::cmp::Ordering::Equal => {
                            debug!("node {a_node_id} are equal to node {b_node_id} after finding both have a chosen sort index.");
                        }
                        std::cmp::Ordering::Greater => {
                            debug!("node {a_node_id} is greater than node {b_node_id} after finding both have a chosen sort index.");
                        }
                    }
                }
                else {
                    debug!("node {a_node_id} is less than node {b_node_id} since the latter has not yet been chosen.");
                    comparison = std::cmp::Ordering::Less;
                }
            }
            else if b.current_chosen_from_sort_index.is_some() {
                debug!("node {a_node_id} is greater than node {b_node_id} since the former has not yet been chosen.");
                comparison = std::cmp::Ordering::Greater;
            }
            else {
                let a_restriction_ratio = a.get_restriction_ratio();
                let b_restriction_ratio = b.get_restriction_ratio();

                if b_restriction_ratio < a_restriction_ratio {
                    debug!("node {a_node_id} is greater than node {b_node_id} after comparing restriction ratios {a_restriction_ratio} to {b_restriction_ratio}.");
                    comparison = std::cmp::Ordering::Greater;
                }
                else if b_restriction_ratio == a_restriction_ratio {

                    let a_random_sort_index = a.random_sort_index;
                    let b_random_sort_index = b.random_sort_index;

                    comparison = a_random_sort_index.cmp(&b_random_sort_index);
                    match &comparison {
                        std::cmp::Ordering::Less => {
                            debug!("node {a_node_id} is less than node {b_node_id} after comparing random sort indexes {a_random_sort_index} to {b_random_sort_index}.");
                        },
                        std::cmp::Ordering::Equal => {
                            debug!("node {a_node_id} are equal to node {b_node_id} after comparing random sort indexes {a_random_sort_index} to {b_random_sort_index}.");
                        }
                        std::cmp::Ordering::Greater => {
                            debug!("node {a_node_id} is greater than node {b_node_id} after comparing random sort indexes {a_random_sort_index} to {b_random_sort_index}.");
                        }
                    }
                }
                else {
                    debug!("node {a_node_id} is less than node {b_node_id} after comparing restriction ratios {a_restriction_ratio} to {b_restriction_ratio}.");
                    comparison = std::cmp::Ordering::Less;
                }
            }
            comparison
        });

        let next_collapsable_nodes_display = CollapsableNode::get_ids(&self.collapsable_nodes);
        debug!("next sort order: {next_collapsable_nodes_display}.");
    }
    fn try_move_to_previous_collapsable_node_neighbor(&mut self) -> bool {
        let mut is_useful_neighbor_found: bool = false;

        // store the original node id in order to check if previously chosen nodes are a neighbor to this fully restricted node
        let original_collapsable_node_id = self.collapsable_nodes.get(self.current_collapsable_node_index).expect("The collapsable node index should be within range.").id;

        // if we're not already at the very root chosen node, then reset the current node and move back up the chain of chosen nodes
        while self.current_collapsable_node_index != 0 {
            self.collapsable_nodes.get_mut(self.current_collapsable_node_index).unwrap().node_state_indexed_view.reset();
            self.collapsable_nodes.get_mut(self.current_collapsable_node_index).unwrap().neighbor_mask_mapped_view.borrow_mut().reset();
            self.current_collapsable_node_index -= 1;
            if self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap().neighbor_node_ids.contains(&original_collapsable_node_id) {
                is_useful_neighbor_found = true;
            }
        }
        is_useful_neighbor_found
    }
    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction {
        let mut node_state_per_node: HashMap<String, String> = HashMap::new();
        for collapsable_node in self.collapsable_nodes.iter() {
            let node_state: String = String::from(*collapsable_node.node_state_indexed_view.get().unwrap());
            let node: String = String::from(collapsable_node.id);
            node_state_per_node.insert(node, node_state);
        }
        CollapsedWaveFunction { node_state_per_node: node_state_per_node }
    }
}

pub struct WaveFunction {
    nodes: Vec<Node>,
    node_state_collections: Vec<NodeStateCollection>,
    all_possible_node_state_ids: HashSet<String>
}

impl WaveFunction {
    pub fn new(nodes: Vec<Node>, node_state_collections: Vec<NodeStateCollection>) -> Self {
        
        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection> = HashMap::new();
        node_state_collections.iter().for_each(|node_state_collection| {
            node_state_collection_per_id.insert(&node_state_collection.id, node_state_collection);
        });

        let mut all_possible_node_state_ids: HashSet<String> = HashSet::new();
        for node in nodes.iter() {
            let node_id: &str = &node.id;
            for (neighbor_node_id_string, node_state_collection_ids) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
                let neighbor_node_id: &str = neighbor_node_id_string;

                debug!("Node {node_id} has neighbor {neighbor_node_id}.");

                for node_state_collection_id_string in node_state_collection_ids {
                    let node_state_collection_id: &str = &node_state_collection_id_string;
                    let node_state_collection: &NodeStateCollection = node_state_collection_per_id.get(node_state_collection_id).expect("The permitted node state collection should exist for this id. Verify that all node state collections are being provided.");
                    let node_state_id: &str = &node_state_collection.node_state_id;

                    all_possible_node_state_ids.insert(String::from(node_state_id));
                    for node_state_id_string in node_state_collection.node_state_ids.iter() {
                        let node_state_id: &str = node_state_id_string;
                        all_possible_node_state_ids.insert(String::from(node_state_id));
                    }
                }
            }
        }

        WaveFunction {
            nodes: nodes,
            node_state_collections: node_state_collections,
            all_possible_node_state_ids: all_possible_node_state_ids
        }
    }
    fn validate(&self) -> Result<(), String> {
        let nodes_length: usize = self.nodes.len();

        if nodes_length < 2 {
            return Err(String::from("There must be two or more nodes."));
        }

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
        
        // collect all possible states for later usage
        let all_possible_node_state_ids_length: u32 = self.all_possible_node_state_ids.len().try_into().expect("The length should be castable to u32.");

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
                        // ensure that valid states for neighbors do not contain duplicate states
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
        }

        if error_message.is_some() {
            Err(error_message.unwrap())
        }
        else {
            Ok(())
        }
    }
    pub fn collapse(&self, random_seed: Option<u64>) -> Result<CollapsedWaveFunction, String> {
        let validation_result = self.validate();

        if let Err(error_message) = validation_result {
            return Err(error_message);
        }

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
            let node_id: &str = &node.id;

            let mut neighbor_mask_mapped_view: MappedView<&str, &str, BitVec> = MappedView::new();

            for (neighbor_node_id_string, node_state_collection_ids) in node.node_state_collection_ids_per_neighbor_node_id.iter() {
                let neighbor_node_id: &str = neighbor_node_id_string;

                debug!("creating neighbor mask for node {node_id} neighbor {neighbor_node_id}.");

                // determine the masks for this neighbor based on its possible node states
                let neighbor_node = node_per_id.get(neighbor_node_id).expect("The neighbor should exist in the complete list of nodes.");
                let neighbor_node_state_ids = &self.all_possible_node_state_ids;

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

            let mut node_state_ids: Vec<&str> = Vec::new();
            for node_state_id_string in self.all_possible_node_state_ids.iter() {
                let node_state_id: &str = node_state_id_string;
                node_state_ids.push(node_state_id);
            }
            let node_state_indexed_view = IndexedView::new(node_state_ids, masks, node_id);
            node_state_indexed_view_per_node_id.insert(node_id, node_state_indexed_view);
        }

        let mut random_instance: Option<ChaCha8Rng> = None;

        let mut collapsable_node_index: usize = 0;
        let mut collapsable_nodes: Vec<CollapsableNode> = Vec::new();
        let mut collapsable_node_index_per_node_id: HashMap<&str, usize> = HashMap::new();
        // contains the mask to apply to the neighbor when this node is in a specific state
        for node in self.nodes.iter() {
            let node_id: &str = &node.id;

            let neighber_masked_mapped_view: Rc<RefCell<MappedView<&str, &str, BitVec>>> = neighbor_mask_mapped_view_per_node_id.remove(node_id).unwrap();
            let node_state_indexed_view: IndexedView<&str, &str, &str> = node_state_indexed_view_per_node_id.remove(node_id).unwrap();

            let mut collapsable_node = CollapsableNode::new(node, neighber_masked_mapped_view, node_state_indexed_view);

            if let Some(seed) = random_seed {
                if random_instance.is_none() {
                    random_instance = Some(ChaCha8Rng::seed_from_u64(seed));
                }
                collapsable_node.randomize(random_instance.as_mut().unwrap());
            }

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

        debug!("sorting initial list of collapsable nodes");
        collapsable_wave_function.sort_collapsable_nodes();
        debug!("sorted initial list of collapsable nodes");

        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !collapsable_wave_function.is_fully_collapsed() {
            debug!("incrementing node state");
            if collapsable_wave_function.try_increment_current_collapsable_node_state() {
                debug!("incremented node state");
                collapsable_wave_function.alter_reference_to_current_collapsable_node_mask();
                debug!("altered reference");
                if !collapsable_wave_function.is_at_least_one_neighbor_fully_restricted() {
                    debug!("all neighbors have at least one valid state");
                    collapsable_wave_function.move_to_next_collapsable_node();
                    debug!("moved to next collapsable node");
                    if !collapsable_wave_function.is_fully_collapsed() {
                        debug!("not yet fully collapsed");
                        collapsable_wave_function.sort_collapsable_nodes();
                        debug!("sorted nodes");
                    }
                }
                else {
                    debug!("at least one neighbor is fully restricted");
                }
            }
            else {
                debug!("failed to incremented node");
                if !collapsable_wave_function.try_move_to_previous_collapsable_node_neighbor() {
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
}










































#[cfg(test)]
mod unit_tests {

    use super::*;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::try_init();
    }

    #[test]
    fn initialize() {
        init();

        let nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();
        let wave_function = WaveFunction::new(nodes, node_state_collections);
        debug!("Succeeded to initialize WaveFunction instance.");
    }

    #[test]
    fn no_nodes() {
        init();

        let nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();
        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let collapsed_wave_function_result = wave_function.collapse(None);
        assert_eq!("There must be two or more nodes.", collapsed_wave_function_result.err().unwrap());
    }

    #[test]
    fn one_node() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let collapsed_wave_function_result = wave_function.collapse(None);
        assert_eq!("There must be two or more nodes.", collapsed_wave_function_result.err().unwrap());
    }

    #[test]
    fn two_nodes_without_neighbors() {
        init();

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

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let collapsed_wave_function_result = wave_function.collapse(None);
        assert_eq!("Not all nodes connect together. At least one node must be able to traverse to all other nodes.", collapsed_wave_function_result.err().unwrap());
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor() {
        init();

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

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let collapsed_wave_function_result = wave_function.collapse(None);
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_both_as_neighbors() {
        init();

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

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let collapsed_wave_function_result = wave_function.collapse(None);

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());

    }

    #[test]
    fn two_nodes_both_as_neighbors_and_different_states() {
        init();

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

        let one_node_state_id: String = Uuid::new_v4().to_string();
        let two_node_state_id: String = Uuid::new_v4().to_string();
        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let if_one_not_two_node_state_collection_id: String = Uuid::new_v4().to_string();
        let if_one_not_two_node_state_collection = NodeStateCollection {
            id: if_one_not_two_node_state_collection_id.clone(),
            node_state_id: one_node_state_id.clone(),
            node_state_ids: vec![two_node_state_id.clone()]
        };
        node_state_collections.push(if_one_not_two_node_state_collection);

        let if_two_not_one_node_state_collection_id: String = Uuid::new_v4().to_string();
        let if_two_not_one_node_state_collection = NodeStateCollection {
            id: if_two_not_one_node_state_collection_id.clone(),
            node_state_id: two_node_state_id.clone(),
            node_state_ids: vec![one_node_state_id.clone()]
        };
        node_state_collections.push(if_two_not_one_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_one_not_two_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_two_not_one_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_one_not_two_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_two_not_one_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let collapsed_wave_function_result = wave_function.collapse(None);

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());

    }

    #[test]
    fn two_nodes_both_as_neighbors_and_different_states_with_random_runs() {
        init();

        let mut rng = rand::thread_rng();

        for _ in 0..10 {
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

            let one_node_state_id: String = Uuid::new_v4().to_string();
            let two_node_state_id: String = Uuid::new_v4().to_string();
            let first_node_id: String = nodes[0].id.clone();
            let second_node_id: String = nodes[1].id.clone();

            let if_one_not_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_one_not_two_node_state_collection = NodeStateCollection {
                id: if_one_not_two_node_state_collection_id.clone(),
                node_state_id: one_node_state_id.clone(),
                node_state_ids: vec![two_node_state_id.clone()]
            };
            node_state_collections.push(if_one_not_two_node_state_collection);

            let if_two_not_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_two_not_one_node_state_collection = NodeStateCollection {
                id: if_two_not_one_node_state_collection_id.clone(),
                node_state_id: two_node_state_id.clone(),
                node_state_ids: vec![one_node_state_id.clone()]
            };
            node_state_collections.push(if_two_not_one_node_state_collection);

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_one_not_two_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_two_not_one_node_state_collection_id.clone());

            nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_one_not_two_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_two_not_one_node_state_collection_id.clone());

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            let random_seed = Some(rng.gen::<u64>());
            let collapsed_wave_function_result = wave_function.collapse(random_seed);

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        }
    }

    #[test]
    fn two_nodes_both_as_neighbors_with_conflicting_state_requirements() {
        init();

        let mut rng = rand::thread_rng();

        for _ in 0..10 { // TODO increase number of trials
            let mut nodes: Vec<Node> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

            nodes.push(Node { 
                id: String::from("node_1"),
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_2"),
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");
            let three_node_state_id: String = String::from("state_C");
            let four_node_state_id: String = String::from("state_D");
            let first_node_id: String = nodes[0].id.clone();
            let second_node_id: String = nodes[1].id.clone();

            let if_one_then_three_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_one_then_three_node_state_collection = NodeStateCollection {
                id: if_one_then_three_node_state_collection_id.clone(),
                node_state_id: one_node_state_id.clone(),
                node_state_ids: vec![three_node_state_id.clone()]
            };
            node_state_collections.push(if_one_then_three_node_state_collection);

            let if_two_then_four_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_two_then_four_node_state_collection = NodeStateCollection {
                id: if_two_then_four_node_state_collection_id.clone(),
                node_state_id: two_node_state_id.clone(),
                node_state_ids: vec![four_node_state_id.clone()]
            };
            node_state_collections.push(if_two_then_four_node_state_collection);

            let if_three_then_no_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_three_then_no_node_state_collection = NodeStateCollection {
                id: if_three_then_no_node_state_collection_id.clone(),
                node_state_id: three_node_state_id.clone(),
                node_state_ids: Vec::new()
            };
            node_state_collections.push(if_three_then_no_node_state_collection);

            let if_four_then_no_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_four_then_no_node_state_collection = NodeStateCollection {
                id: if_four_then_no_node_state_collection_id.clone(),
                node_state_id: four_node_state_id.clone(),
                node_state_ids: Vec::new()
            };
            node_state_collections.push(if_four_then_no_node_state_collection);

            let if_three_then_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_three_then_two_node_state_collection = NodeStateCollection {
                id: if_three_then_two_node_state_collection_id.clone(),
                node_state_id: three_node_state_id.clone(),
                node_state_ids: vec![two_node_state_id.clone()]
            };
            node_state_collections.push(if_three_then_two_node_state_collection);

            let if_four_then_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_four_then_one_node_state_collection = NodeStateCollection {
                id: if_four_then_one_node_state_collection_id.clone(),
                node_state_id: four_node_state_id.clone(),
                node_state_ids: vec![one_node_state_id.clone()]
            };
            node_state_collections.push(if_four_then_one_node_state_collection);

            let if_one_then_no_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_one_then_no_node_state_collection = NodeStateCollection {
                id: if_one_then_no_node_state_collection_id.clone(),
                node_state_id: one_node_state_id.clone(),
                node_state_ids: Vec::new()
            };
            node_state_collections.push(if_one_then_no_node_state_collection);

            let if_two_then_no_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_two_then_no_node_state_collection = NodeStateCollection {
                id: if_two_then_no_node_state_collection_id.clone(),
                node_state_id: two_node_state_id.clone(),
                node_state_ids: Vec::new()
            };
            node_state_collections.push(if_two_then_no_node_state_collection);

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_one_then_three_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_two_then_four_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_three_then_no_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_four_then_no_node_state_collection_id.clone());

            nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_three_then_two_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_four_then_one_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_one_then_no_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_two_then_no_node_state_collection_id.clone());

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            let random_seed = Some(rng.gen::<u64>()); // TODO randomize
            let collapsed_wave_function_result = wave_function.collapse(random_seed);

            assert_eq!("Cannot collapse wave function.", collapsed_wave_function_result.err().unwrap());
        }
    }
}