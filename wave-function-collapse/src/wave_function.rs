use std::{collections::{HashMap, HashSet, BTreeSet, VecDeque}, cell::{Cell, RefCell}, rc::Rc, fmt::Display, hash::Hash};
use serde::{Deserialize, Serialize};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use bitvec::prelude::*;
use uuid::Uuid;
use log::debug;
extern crate pretty_env_logger;

mod indexed_view;
use self::indexed_view::IndexedView;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub node_state_ids: Vec<String>,
    pub node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>>
}

impl Node {
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

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct NodeState {
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
    current_chosen_from_sort_index: Option<usize>,
    // a random sort value for adding randomness to the process between runs (if randomized)
    random_sort_index: u32
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
            current_chosen_from_sort_index: None,
            random_sort_index: 0
        }
    }
    #[time_graph::instrument]
    fn randomize<R: Rng + ?Sized>(&mut self, random_instance: &mut R) {
        self.node_state_indexed_view.shuffle(random_instance);
        self.random_sort_index = random_instance.next_u32();
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
    fn try_increment_current_collapsable_node_state(&mut self) -> NodeState {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

        let is_successful = current_collapsable_node.node_state_indexed_view.try_move_next();
        let node_state: NodeState;
        if is_successful {
            current_collapsable_node.current_chosen_from_sort_index = Some(self.current_collapsable_node_index);
            node_state = NodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: Some(String::from(*current_collapsable_node.node_state_indexed_view.get().unwrap()))
            };
        }
        else {
            current_collapsable_node.current_chosen_from_sort_index = None;
            node_state = NodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: None
            };
        }

        node_state
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
    fn sort_collapsable_nodes(&mut self) {
        //let current_collapsable_nodes_display = CollapsableNode::get_ids(&self.collapsable_nodes);
        //debug!("current sort order: {current_collapsable_nodes_display}.");

        // sort by most neighbors
        {
            self.collapsable_nodes.sort_by(|a, b| {

                let a_collapsed_node = a.borrow();
                let b_collapsed_node = b.borrow();

                a_collapsed_node.random_sort_index.cmp(&b_collapsed_node.random_sort_index)
            });

            let current_collapsable_nodes_display = CollapsableNode::get_ids(&self.collapsable_nodes);
            debug!("after random_sort_index sort order: {current_collapsable_nodes_display}.");

            self.collapsable_nodes.sort_by(|a, b| {

                let a_collapsed_node = a.borrow();
                let b_collapsed_node = b.borrow();

                a_collapsed_node.neighbor_node_ids.len().cmp(&b_collapsed_node.neighbor_node_ids.len())
            });

            let current_collapsable_nodes_display = CollapsableNode::get_ids(&self.collapsable_nodes);
            debug!("after neighbor_node_ids sort order: {current_collapsable_nodes_display}.");

            let mut found_neighbor_node_ids: HashSet<&str> = HashSet::new();
            let mut searching_neighbor_node_ids: VecDeque<&str> = VecDeque::new();
            searching_neighbor_node_ids.push_back(&self.collapsable_nodes.first().unwrap().borrow().id);

            let mut choice_index: usize = 0;
            while !searching_neighbor_node_ids.is_empty() {
                let searching_neighbor_node_id = searching_neighbor_node_ids.pop_front().unwrap();
                debug!("searching: {:?}", searching_neighbor_node_ids);
                for finding_index in choice_index..self.collapsable_nodes_length {
                    let finding_collapsable_node_id: &str;
                    {
                        let wrapped_finding_collapsable_node = self.collapsable_nodes.get(finding_index).unwrap();
                        let finding_collapsable_node = wrapped_finding_collapsable_node.borrow();
                        finding_collapsable_node_id = finding_collapsable_node.id;
                    }
                    
                    if searching_neighbor_node_id == finding_collapsable_node_id {
                        debug!("found at {finding_index} and moving to {choice_index}.");
                        if choice_index != finding_index {
                            self.collapsable_nodes.swap(choice_index, finding_index);
                        }
                        choice_index += 1;
                        found_neighbor_node_ids.insert(finding_collapsable_node_id);
                        let wrapped_neighbor = self.collapsable_node_per_id.get(finding_collapsable_node_id).unwrap();
                        let neighbor = wrapped_neighbor.borrow();
                        for neighbors_neighbor_node_id in neighbor.neighbor_node_ids.iter() {
                            if !found_neighbor_node_ids.contains(neighbors_neighbor_node_id) {
                                debug!("adding potential neighbor: {neighbors_neighbor_node_id}");
                                searching_neighbor_node_ids.push_back(*neighbors_neighbor_node_id);
                            }
                        }
                        break;
                    }
                }
            }

            /*
            while choice_index + 1 < self.collapsable_nodes_length {
                let mut neighbor_indexes: Vec<usize> = Vec::new();
                {
                    let wrapper_choice_node = self.collapsable_nodes.get(choice_index).unwrap();
                    let choice_node = wrapper_choice_node.borrow();
                    for other_index in (choice_index + 1)..self.collapsable_nodes_length {
                        let wrapper_other_node = self.collapsable_nodes.get(other_index).unwrap();
                        let other_node = wrapper_other_node.borrow();

                        if choice_node.neighbor_node_ids.contains(&other_node.id) {
                            neighbor_indexes.push(other_index);
                        }
                    }
                }
                for (choice_index_offset, neighbor_index) in neighbor_indexes.iter().enumerate() {
                    let neighbor_node = self.collapsable_nodes.remove(*neighbor_index);
                    self.collapsable_nodes.insert(choice_index + choice_index_offset + 1, neighbor_node);
                }
                if neighbor_indexes.len() == 0 {
                    choice_index += 1;
                }
                else {
                    choice_index += neighbor_indexes.len();
                }
            }
            */
        }

        // sort by restriction ratio
        /*{
            let mut lowest_sort_criteria: f32 = f32::MAX;
            let mut lowest_sort_criteria_index = self.current_collapsable_node_index;
            for collapsable_node_index in self.current_collapsable_node_index..self.collapsable_nodes_length {
                let wrapped_collapsable_node = self.collapsable_nodes.get(collapsable_node_index).unwrap();
                let collapsable_node = wrapped_collapsable_node.borrow();
                let collapsable_node_sort_criteria = collapsable_node.restriction_ratio;
                if collapsable_node_sort_criteria < lowest_sort_criteria {
                    lowest_sort_criteria = collapsable_node_sort_criteria;
                    lowest_sort_criteria_index = collapsable_node_index;
                }
            }

            if lowest_sort_criteria_index != self.current_collapsable_node_index {
                self.collapsable_nodes.swap(lowest_sort_criteria_index, self.current_collapsable_node_index);
            }
        }*/

        // sort by random sort index
        /*{
            let mut lowest_number_of_possible_states: u32 = u32::MAX;
            let mut lowest_number_of_possible_states_index = self.current_collapsable_node_index;
            for collapsable_node_index in self.current_collapsable_node_index..self.collapsable_nodes_length {
                let wrapped_collapsable_node = self.collapsable_nodes.get(collapsable_node_index).unwrap();
                let collapsable_node = wrapped_collapsable_node.borrow();
                let collapsable_node_random_sort_index = collapsable_node.random_sort_index;
                if collapsable_node_random_sort_index < lowest_number_of_possible_states {
                    lowest_number_of_possible_states = collapsable_node_random_sort_index;
                    lowest_number_of_possible_states_index = collapsable_node_index;
                }
            }

            if lowest_number_of_possible_states_index != self.current_collapsable_node_index {
                self.collapsable_nodes.swap(lowest_number_of_possible_states_index, self.current_collapsable_node_index);
            }
        }*/

        // sort by if selected, then restriction ratio, then random
        /*
        self.collapsable_nodes.sort_unstable_by(|a, b| {

            let a_collapsed_node = a.borrow();
            let b_collapsed_node = b.borrow();

            let a_node_id: &str = a_collapsed_node.id;
            let b_node_id: &str = b_collapsed_node.id;

            let comparison: std::cmp::Ordering;
            if let Some(a_chosen_from_sort_index) = a_collapsed_node.current_chosen_from_sort_index {
                if let Some(b_chosen_from_sort_index) = b_collapsed_node.current_chosen_from_sort_index {
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
            else if b_collapsed_node.current_chosen_from_sort_index.is_some() {
                debug!("node {a_node_id} is greater than node {b_node_id} since the former has not yet been chosen.");
                comparison = std::cmp::Ordering::Greater;
            }
            else {
                debug!("determining restriction ratio for node {a_node_id}.");
                let a_restriction_ratio = a_collapsed_node.restriction_ratio;
                debug!("determined restriction ratio for node {a_node_id} as {a_restriction_ratio}.");
                debug!("determining restriction ratio for node {b_node_id}.");
                let b_restriction_ratio = b_collapsed_node.restriction_ratio;
                debug!("determined restriction ratio for node {b_node_id} as {b_restriction_ratio}.");

                if b_restriction_ratio < a_restriction_ratio {
                    debug!("node {a_node_id} is greater than node {b_node_id} after comparing restriction ratios {a_restriction_ratio} to {b_restriction_ratio}.");
                    comparison = std::cmp::Ordering::Greater;
                }
                else if b_restriction_ratio == a_restriction_ratio {

                    let a_random_sort_index = a_collapsed_node.random_sort_index;
                    let b_random_sort_index = b_collapsed_node.random_sort_index;

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
        */

        //let next_collapsable_nodes_display = CollapsableNode::get_ids(&self.collapsable_nodes);
        //debug!("next sort order: {next_collapsable_nodes_display}.");
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
    node_state_collections: Vec<NodeStateCollection>,
    is_sort_required: bool
}

impl WaveFunction {
    pub fn new(nodes: Vec<Node>, node_state_collections: Vec<NodeStateCollection>) -> Self {
        
        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection> = HashMap::new();
        node_state_collections.iter().for_each(|node_state_collection| {
            node_state_collection_per_id.insert(&node_state_collection.id, node_state_collection);
        });

        WaveFunction {
            nodes: nodes,
            node_state_collections: node_state_collections,
            is_sort_required: false
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

    pub fn sort(&mut self) {
        self.is_sort_required = true;
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
        for node in self.nodes.iter() {
            let node_id: &str = &node.id;

            let node_state_indexed_view: IndexedView<&str> = node_state_indexed_view_per_node_id.remove(node_id).unwrap();
            let mask_per_neighbor_per_state = neighbor_mask_mapped_view_per_node_id.remove(node_id).unwrap();

            let mut collapsable_node = CollapsableNode::new(node, mask_per_neighbor_per_state, node_state_indexed_view);

            if let Some(seed) = random_seed {
                if random_instance.is_none() {
                    random_instance = Some(ChaCha8Rng::seed_from_u64(seed));
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
    pub fn collapse_into_steps(&self, random_seed: Option<u64>) -> Result<Vec<NodeState>, String> {
        let mut node_states: Vec<NodeState> = Vec::new();

        let mut collapsable_wave_function = self.get_collapsable_wave_function(random_seed);

        time_graph::spanned!("sort_collapsable_nodes (first)", {
            if self.is_sort_required {
                debug!("sorting initial list of collapsable nodes");
                collapsable_wave_function.sort_collapsable_nodes();
                debug!("sorted initial list of collapsable nodes");
            }
        });

        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !collapsable_wave_function.is_fully_collapsed() {
            time_graph::spanned!("is_increment_successful", {
                collapsable_wave_function.revert_existing_neighbor_masks();
            });
            debug!("incrementing node state");
            let node_state = collapsable_wave_function.try_increment_current_collapsable_node_state();
            let is_successful: bool = node_state.node_state_id.is_some();
            node_states.push(node_state);

            debug!("stored node state");
            if is_successful {
                debug!("incremented node state: {:?}", node_states.last());
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

        Ok(node_states)
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

        debug!("sorting initial list of collapsable nodes");
        time_graph::spanned!("sort_collapsable_nodes (first)", {
            if self.is_sort_required {
                collapsable_wave_function.sort_collapsable_nodes();
            }
        });
        debug!("sorted initial list of collapsable nodes");

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










































#[cfg(test)]
mod unit_tests {

    use super::*;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
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
        let validation_result = wave_function.validate();

        assert_eq!("Not all nodes connect together. At least one node must be able to traverse to all other nodes.", validation_result.err().unwrap());
    }

    #[test]
    fn one_node_no_states() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: Vec::new(),
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        let collapsed_wave_function_result = wave_function.collapse(None);

        assert_eq!("Cannot collapse wave function.", collapsed_wave_function_result.err().unwrap());
    }

    #[test]
    fn one_node_one_state() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let node_id: String = Uuid::new_v4().to_string();
        let node_state_id: String = Uuid::new_v4().to_string();

        nodes.push(Node { 
            id: node_id.clone(),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        let collapsed_wave_function = wave_function.collapse(None).unwrap();
        
        assert_eq!(1, collapsed_wave_function.node_state_per_node.keys().len());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&node_id).unwrap());
    }

    #[test]
    fn one_node_randomly_two_states() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let one_node_state_id: String = Uuid::new_v4().to_string();
        let two_node_state_id: String = Uuid::new_v4().to_string();
        let mut count_per_node_state_id: HashMap<&str, u32> = HashMap::new();
        count_per_node_state_id.insert(&one_node_state_id, 0);
        count_per_node_state_id.insert(&two_node_state_id, 0);

        let node_id: String = Uuid::new_v4().to_string();

        nodes.push(Node { 
            id: node_id.clone(),
            node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        
        let mut rng = rand::thread_rng();

        for _ in 0..100000 {
            let random_seed = Some(rng.next_u64());
            let collapsed_wave_function = wave_function.collapse(random_seed).unwrap();

            let node_state_id: &str = collapsed_wave_function.node_state_per_node.get(&node_id).unwrap();
            *count_per_node_state_id.get_mut(node_state_id).unwrap() += 1;
        }

        println!("count_per_node_state_id: {:?}", count_per_node_state_id);
        assert!(count_per_node_state_id.get(one_node_state_id.as_str()).unwrap() > &49000, "The first node state was less than expected.");
        assert!(count_per_node_state_id.get(two_node_state_id.as_str()).unwrap() > &49000, "The first node state was less than expected.");
    }

    #[test]
    fn two_nodes_without_neighbors() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection> = Vec::new();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![Uuid::new_v4().to_string()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![Uuid::new_v4().to_string()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let validation_result = wave_function.validate();
        assert_eq!("Not all nodes connect together. At least one node must be able to traverse to all other nodes.", validation_result.err().unwrap());
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor_restriction_ignored() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let unrestricted_node_state_id: String = String::from("unrestricted");
        let from_restrictive_node_state_id: String = String::from("from_restrictive");
        let to_restrictive_node_state_id: String = String::from("to_restrictive");

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![from_restrictive_node_state_id.clone(), unrestricted_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![unrestricted_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let restrictive_node_state_collection_id: String = Uuid::new_v4().to_string();
        let restrictive_node_state_collection = NodeStateCollection {
            id: restrictive_node_state_collection_id.clone(),
            node_state_id: from_restrictive_node_state_id.clone(),
            node_state_ids: vec![to_restrictive_node_state_id.clone()]
        };
        node_state_collections.push(restrictive_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(restrictive_node_state_collection_id.clone());

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);
            let collapsed_wave_function = collapsed_wave_function_result.unwrap();

            assert_eq!(&unrestricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
            assert_eq!(&unrestricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        }
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor_ordered() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let node_state_id: String = Uuid::new_v4().to_string();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let mut neighbor_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..1000 {
            neighbor_node_state_ids.push(Uuid::new_v4().to_string());
        }
        neighbor_node_state_ids.push(node_state_id.clone());

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: neighbor_node_state_ids,
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

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

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);
            let collapsed_wave_function = collapsed_wave_function_result.unwrap();

            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        }
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor_disordered() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let node_state_id: String = Uuid::new_v4().to_string();

        let mut neighbor_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..1 {
            neighbor_node_state_ids.push(Uuid::new_v4().to_string());
        }
        neighbor_node_state_ids.push(node_state_id.clone());

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: neighbor_node_state_ids,
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let first_node_id: String = nodes[1].id.clone();
        let second_node_id: String = nodes[0].id.clone();

        let same_node_state_collection_id: String = Uuid::new_v4().to_string();
        let same_node_state_collection = NodeStateCollection {
            id: same_node_state_collection_id.clone(),
            node_state_id: node_state_id.clone(),
            node_state_ids: vec![node_state_id.clone()]
        };
        node_state_collections.push(same_node_state_collection);

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);
            let collapsed_wave_function = collapsed_wave_function_result.unwrap();

            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        }
    }

    #[test]
    fn two_nodes_both_as_neighbors_only_ordered() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let node_state_id: String = String::from("state_A");

        nodes.push(Node { 
            id: String::from("node_1"),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let mut neighbor_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..1000 {
            neighbor_node_state_ids.push(Uuid::new_v4().to_string());
        }
        neighbor_node_state_ids.push(node_state_id.clone());

        nodes.push(Node { 
            id: String::from("node_2"),
            node_state_ids: neighbor_node_state_ids,
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

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

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        }
    }

    #[test]
    fn two_nodes_both_as_neighbors_only_disordered() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let node_state_id: String = String::from("state_A");

        let mut neighbor_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..1000 {
            neighbor_node_state_ids.push(Uuid::new_v4().to_string());
        }
        neighbor_node_state_ids.push(node_state_id.clone());

        nodes.push(Node { 
            id: String::from("node_2"),
            node_state_ids: neighbor_node_state_ids,
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        nodes.push(Node { 
            id: String::from("node_1"),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

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

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        }
    }

    #[test]
    fn two_nodes_both_as_neighbors_and_different_states() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let one_node_state_id: String = Uuid::new_v4().to_string();
        let two_node_state_id: String = Uuid::new_v4().to_string();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

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

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        }
    }

    #[test]
    fn two_nodes_both_as_neighbors_and_different_states_with_random_runs() {
        init();

        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let mut nodes: Vec<Node> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

            let one_node_state_id: String = Uuid::new_v4().to_string();
            let two_node_state_id: String = Uuid::new_v4().to_string();

            nodes.push(Node { 
                id: Uuid::new_v4().to_string(),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: Uuid::new_v4().to_string(),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });

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

            let mut wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            let random_seed = Some(rng.gen::<u64>());

            for is_sort_required in [false, true] {
                if is_sort_required {
                    wave_function.sort();
                }
                let collapsed_wave_function_result = wave_function.collapse(random_seed);

                if let Err(error_message) = collapsed_wave_function_result {
                    panic!("Error: {error_message}");
                }

                let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

                assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
            }
        }
    }

    #[test]
    fn two_nodes_both_as_neighbors_with_conflicting_state_requirements() {
        init();

        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let mut nodes: Vec<Node> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");
            let three_node_state_id: String = String::from("state_C");
            let four_node_state_id: String = String::from("state_D");

            nodes.push(Node { 
                id: String::from("node_1"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone(), three_node_state_id.clone(), four_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_2"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone(), three_node_state_id.clone(), four_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });

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

            let mut wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            let random_seed = Some(rng.gen::<u64>());

            for is_sort_required in [false, true] {
                if is_sort_required {
                    wave_function.sort();
                }
                let collapsed_wave_function_result = wave_function.collapse(random_seed);

                assert_eq!("Cannot collapse wave function.", collapsed_wave_function_result.err().unwrap());
            }
        }
    }

    #[test]
    fn three_nodes_as_neighbors_all_same_state() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let node_state_id: String = String::from("state_A");

        nodes.push(Node { 
            id: String::from("node_1"),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: String::from("node_2"),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: String::from("node_3"),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();
        let third_node_id: String = nodes[2].id.clone();

        let same_node_state_collection_id: String = String::from("nsc_1");
        let same_node_state_collection = NodeStateCollection {
            id: same_node_state_collection_id.clone(),
            node_state_id: node_state_id.clone(),
            node_state_ids: vec![node_state_id.clone()]
        };
        node_state_collections.push(same_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(same_node_state_collection_id.clone());

        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(same_node_state_collection_id.clone());

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
            assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
        }
    }

    #[test]
    fn three_nodes_as_dense_neighbors_all_different_states() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let first_node_state_id: String = String::from("state_A");
        let second_node_state_id: String = String::from("state_B");
        let third_node_state_id: String = String::from("state_C");

        nodes.push(Node { 
            id: String::from("node_1"),
            node_state_ids: vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: String::from("node_2"),
            node_state_ids: vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: String::from("node_3"),
            node_state_ids: vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();
        let third_node_id: String = nodes[2].id.clone();

        let all_but_first_node_state_collection_id: String = String::from("nsc_1");
        let all_but_first_node_state_collection = NodeStateCollection {
            id: all_but_first_node_state_collection_id.clone(),
            node_state_id: first_node_state_id.clone(),
            node_state_ids: vec![second_node_state_id.clone(), third_node_state_id.clone()]
        };
        node_state_collections.push(all_but_first_node_state_collection);

        let all_but_second_node_state_collection_id: String = String::from("nsc_2");
        let all_but_second_node_state_collection = NodeStateCollection {
            id: all_but_second_node_state_collection_id.clone(),
            node_state_id: second_node_state_id.clone(),
            node_state_ids: vec![first_node_state_id.clone(), third_node_state_id.clone()]
        };
        node_state_collections.push(all_but_second_node_state_collection);

        let all_but_third_node_state_collection_id: String = String::from("nsc_3");
        let all_but_third_node_state_collection = NodeStateCollection {
            id: all_but_third_node_state_collection_id.clone(),
            node_state_id: third_node_state_id.clone(),
            node_state_ids: vec![first_node_state_id.clone(), second_node_state_id.clone()]
        };
        node_state_collections.push(all_but_third_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            debug!("collapsed_wave_function.node_state_per_node: {:?}", collapsed_wave_function.node_state_per_node);

            assert_eq!(&first_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
            assert_eq!(&second_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
            assert_eq!(&third_node_state_id, collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
        }
    }

    #[test]
    fn three_nodes_as_dense_neighbors_randomly_all_different_states() {
        init();

        time_graph::enable_data_collection(true);

        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            
            let mut nodes: Vec<Node> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

            let first_node_state_id: String = String::from("state_A");
            let second_node_state_id: String = String::from("state_B");
            let third_node_state_id: String = String::from("state_C");

            nodes.push(Node { 
                id: String::from("node_1"),
                node_state_ids: vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_2"),
                node_state_ids: vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_3"),
                node_state_ids: vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });

            let first_node_id: String = nodes[0].id.clone();
            let second_node_id: String = nodes[1].id.clone();
            let third_node_id: String = nodes[2].id.clone();

            let all_but_first_node_state_collection_id: String = String::from("nsc_1");
            let all_but_first_node_state_collection = NodeStateCollection {
                id: all_but_first_node_state_collection_id.clone(),
                node_state_id: first_node_state_id.clone(),
                node_state_ids: vec![second_node_state_id.clone(), third_node_state_id.clone()]
            };
            node_state_collections.push(all_but_first_node_state_collection);

            let all_but_second_node_state_collection_id: String = String::from("nsc_2");
            let all_but_second_node_state_collection = NodeStateCollection {
                id: all_but_second_node_state_collection_id.clone(),
                node_state_id: second_node_state_id.clone(),
                node_state_ids: vec![first_node_state_id.clone(), third_node_state_id.clone()]
            };
            node_state_collections.push(all_but_second_node_state_collection);

            let all_but_third_node_state_collection_id: String = String::from("nsc_3");
            let all_but_third_node_state_collection = NodeStateCollection {
                id: all_but_third_node_state_collection_id.clone(),
                node_state_id: third_node_state_id.clone(),
                node_state_ids: vec![first_node_state_id.clone(), second_node_state_id.clone()]
            };
            node_state_collections.push(all_but_third_node_state_collection);

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

            nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

            nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

            let mut wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            let random_seed = rng.next_u64();

            for is_sort_required in [false, true] {
                if is_sort_required {
                    wave_function.sort();
                }
                let collapsed_wave_function_result = wave_function.collapse(Some(random_seed));

                if let Err(error_message) = collapsed_wave_function_result {
                    panic!("Error: {error_message}");
                }

                let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

                let first_node_state_id = collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap();
                let second_node_state_id = collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap();
                let third_node_state_id = collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap();
                assert_ne!(second_node_state_id, first_node_state_id);
                assert_ne!(third_node_state_id, first_node_state_id);
                assert_ne!(first_node_state_id, second_node_state_id);
                assert_ne!(third_node_state_id, second_node_state_id);
                assert_ne!(first_node_state_id, third_node_state_id);
                assert_ne!(second_node_state_id, third_node_state_id);
            }
        }
    }
    
    #[test]
    fn many_nodes_as_dense_neighbors_all_different_states() {
        //init();
        time_graph::enable_data_collection(true);

        let nodes_total = 50;

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_ids: Vec<String> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();
        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_collection_ids: Vec<String> = Vec::new();

        time_graph::spanned!("creating test data", {

            for _ in 0..nodes_total {
                node_state_ids.push(Uuid::new_v4().to_string());
            }

            for index in 0..nodes_total {
                let node_id: String = Uuid::new_v4().to_string();
                node_ids.push(node_id.clone());
                nodes.push(Node { 
                    id: node_id,
                    node_state_ids: node_state_ids.clone(),
                    node_state_collection_ids_per_neighbor_node_id: HashMap::new()
                });
            }

            for node_state_id in node_state_ids.iter() {
                let mut other_node_state_ids: Vec<String> = Vec::new();
                for other_node_state_id in node_state_ids.iter() {
                    if node_state_id != other_node_state_id {
                        other_node_state_ids.push(other_node_state_id.clone());
                    }
                }
                
                let node_state_collection_id: String = Uuid::new_v4().to_string();
                node_state_collection_ids.push(node_state_collection_id.clone());
                node_state_collections.push(NodeStateCollection {
                    id: node_state_collection_id,
                    node_state_id: node_state_id.clone(),
                    node_state_ids: other_node_state_ids
                });
            }

            // tie nodes to their neighbors
            for node in nodes.iter_mut() {
                for other_node_id in node_ids.iter() {
                    if *other_node_id != node.id {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                    }
                }
            }
        });

        let mut wave_function: WaveFunction;

        time_graph::spanned!("creating wave function", {
            wave_function = WaveFunction::new(nodes, node_state_collections);
        });

        time_graph::spanned!("validating wave function", {
            wave_function.validate().unwrap();
        });

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
        
            let collapsed_wave_function_result: Result<CollapsedWaveFunction, String>;

            time_graph::spanned!("collapsing wave function", {
                collapsed_wave_function_result = wave_function.collapse(None);
            });

            time_graph::spanned!("check results", {

                if let Err(error_message) = collapsed_wave_function_result {
                    panic!("Error: {error_message}");
                }

                let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

                // check that no nodes have the same state
                for (first_index, (first_node, first_node_state)) in collapsed_wave_function.node_state_per_node.iter().enumerate() {
                    for (second_index, (second_node, second_node_state)) in collapsed_wave_function.node_state_per_node.iter().enumerate() {
                        if first_index == second_index {
                            assert_eq!(first_node, second_node);
                            assert_eq!(first_node_state, second_node_state);
                        }
                        else {
                            assert_ne!(first_node, second_node);
                            assert_ne!(first_node_state, second_node_state);
                        }
                    }
                }
            });
        }

        println!("{}", time_graph::get_full_graph().as_dot());
    }

    #[test]
    fn many_nodes_as_dense_neighbors_randomly_all_different_states() {
        //init();
        time_graph::enable_data_collection(true);

        let mut rng = rand::thread_rng();

        for _ in 0..10 {

            let nodes_total = 20;

            let mut nodes: Vec<Node> = Vec::new();
            let mut node_ids: Vec<String> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();
            let mut node_state_ids: Vec<String> = Vec::new();
            let mut node_state_collection_ids: Vec<String> = Vec::new();

            time_graph::spanned!("creating test data", {

                for _ in 0..nodes_total {
                    node_state_ids.push(Uuid::new_v4().to_string());
                }

                for index in 0..nodes_total {
                    let node_id: String = Uuid::new_v4().to_string();
                    node_ids.push(node_id.clone());
                    nodes.push(Node { 
                        id: node_id,
                        node_state_ids: node_state_ids.clone(),
                        node_state_collection_ids_per_neighbor_node_id: HashMap::new()
                    });
                }

                for node_state_id in node_state_ids.iter() {
                    let mut other_node_state_ids: Vec<String> = Vec::new();
                    for other_node_state_id in node_state_ids.iter() {
                        if node_state_id != other_node_state_id {
                            other_node_state_ids.push(other_node_state_id.clone());
                        }
                    }
                    
                    let node_state_collection_id: String = Uuid::new_v4().to_string();
                    node_state_collection_ids.push(node_state_collection_id.clone());
                    node_state_collections.push(NodeStateCollection {
                        id: node_state_collection_id,
                        node_state_id: node_state_id.clone(),
                        node_state_ids: other_node_state_ids
                    });
                }

                // tie nodes to their neighbors
                for node in nodes.iter_mut() {
                    for other_node_id in node_ids.iter() {
                        if *other_node_id != node.id {
                            node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                        }
                    }
                }
            });

            let mut wave_function: WaveFunction;

            time_graph::spanned!("creating wave function", {
                wave_function = WaveFunction::new(nodes, node_state_collections);
            });

            time_graph::spanned!("validating wave function", {
                wave_function.validate().unwrap();
            });

            for is_sort_required in [false, true] {
                if is_sort_required {
                    wave_function.sort();
                }

                let collapsed_wave_function_result: Result<CollapsedWaveFunction, String>;
                
                time_graph::spanned!("collapsing wave function", {
                    let random_seed = rng.next_u64();
                    collapsed_wave_function_result = wave_function.collapse(Some(random_seed));
                });

                time_graph::spanned!("check results", {

                    if let Err(error_message) = collapsed_wave_function_result {
                        panic!("Error: {error_message}");
                    }

                    let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

                    let mut all_node_state_ids: Vec<String> = Vec::new();
                    for (node_state_id, node_id) in std::iter::zip(&node_state_ids, &node_ids) {
                        if !all_node_state_ids.contains(&node_state_id) {
                            all_node_state_ids.push(node_state_id.clone());
                        }
                    }

                    assert_eq!(nodes_total, all_node_state_ids.len());
                });
            }
        }

        println!("{}", time_graph::get_full_graph().as_dot());
    }

    #[test]
    fn many_nodes_as_3D_grid_all_different_states() {
        init();
        time_graph::enable_data_collection(true);

        let nodes_height = 25;
        let nodes_width = 25;
        let nodes_depth = 25;
        let nodes_total = nodes_height * nodes_width * nodes_depth;
        let node_states_total = 12;

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_ids: Vec<String> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();
        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_collection_ids: Vec<String> = Vec::new();

        time_graph::spanned!("creating test data", {

            for _ in 0..node_states_total {
                node_state_ids.push(Uuid::new_v4().to_string());
            }

            for _ in 0..nodes_total {
                let node_id: String = Uuid::new_v4().to_string();
                node_ids.push(node_id.clone());
                nodes.push(Node { 
                    id: node_id,
                    node_state_ids: node_state_ids.clone(),
                    node_state_collection_ids_per_neighbor_node_id: HashMap::new()
                });
            }

            for node_state_id in node_state_ids.iter() {
                let mut other_node_state_ids: Vec<String> = Vec::new();
                for other_node_state_id in node_state_ids.iter() {
                    if node_state_id != other_node_state_id {
                        other_node_state_ids.push(other_node_state_id.clone());
                    }
                }
                
                let node_state_collection_id: String = Uuid::new_v4().to_string();
                node_state_collection_ids.push(node_state_collection_id.clone());
                node_state_collections.push(NodeStateCollection {
                    id: node_state_collection_id,
                    node_state_id: node_state_id.clone(),
                    node_state_ids: other_node_state_ids
                });
            }

            // tie nodes to their neighbors
            for (node_index, node) in std::iter::zip(0..nodes_total, nodes.iter_mut()) {
                let node_x: i32 = node_index % nodes_width;
                let node_y: i32 = (node_index / nodes_width) % nodes_height;
                let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
                let mut neighbors_total = 0;
                //debug!("processing node {node_x}, {node_y}, {node_z}.");
                for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                    let other_node_x: i32 = other_node_index % nodes_width;
                    let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                    let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                    if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                        //debug!("found neighbor at {other_node_x}, {other_node_y}, {other_node_z}.");
                        node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                        neighbors_total += 1;
                    }
                }
                //debug!("neighbors: {neighbors_total}.");
            }
        });

        let mut wave_function: WaveFunction;

        time_graph::spanned!("creating wave function", {
            wave_function = WaveFunction::new(nodes, node_state_collections);
        });

        time_graph::spanned!("validating wave function", {
            wave_function.validate().unwrap();
        });
        

        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }

            let collapsed_wave_function_result: Result<CollapsedWaveFunction, String>;
            
            time_graph::spanned!("collapsing wave function", {
                collapsed_wave_function_result = wave_function.collapse(None);
            });

            time_graph::spanned!("check results", {

                if let Err(error_message) = collapsed_wave_function_result {
                    println!("{}", time_graph::get_full_graph().as_dot());
                    panic!("Error: {error_message}");
                }

                let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

                // check that none of the neighbors match the same state
                for (node_index, node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                    let node_x: i32 = node_index % nodes_width;
                    let node_y: i32 = (node_index / nodes_width) % nodes_height;
                    let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
                    for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                        let other_node_x: i32 = other_node_index % nodes_width;
                        let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                        let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                        if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                            assert_ne!(collapsed_wave_function.node_state_per_node.get(node_id), collapsed_wave_function.node_state_per_node.get(other_node_id));
                        }
                    }
                }
            });
        }

        println!("{}", time_graph::get_full_graph().as_dot());
    }

    #[test]
    fn many_nodes_as_3D_grid_randomly_all_different_states_getting_collapsed_function() {
        init();
        time_graph::enable_data_collection(true);

        let mut rng = rand::thread_rng();
        let random_seed = Some(15177947778026677005);
        //let random_seed = None;

        let max_runs = 1;

        for index in 0..max_runs {

            //let random_seed = Some(rng.next_u64());

            let nodes_height = 4;
            let nodes_width = 4;
            let nodes_depth = 4;
            let nodes_total = nodes_height * nodes_width * nodes_depth;
            let node_states_total = 8;

            let mut nodes: Vec<Node> = Vec::new();
            let mut node_ids: Vec<String> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();
            let mut node_state_ids: Vec<String> = Vec::new();
            let mut node_state_collection_ids: Vec<String> = Vec::new();

            time_graph::spanned!("creating test data", {

                for _ in 0..node_states_total {
                    node_state_ids.push(Uuid::new_v4().to_string());
                }

                for _ in 0..nodes_total {
                    let node_id: String = Uuid::new_v4().to_string();
                    node_ids.push(node_id.clone());
                    nodes.push(Node { 
                        id: node_id,
                        node_state_ids: node_state_ids.clone(),
                        node_state_collection_ids_per_neighbor_node_id: HashMap::new()
                    });
                }

                for node_state_id in node_state_ids.iter() {
                    let mut other_node_state_ids: Vec<String> = Vec::new();
                    for other_node_state_id in node_state_ids.iter() {
                        if node_state_id != other_node_state_id {
                            other_node_state_ids.push(other_node_state_id.clone());
                        }
                    }
                    
                    let node_state_collection_id: String = Uuid::new_v4().to_string();
                    node_state_collection_ids.push(node_state_collection_id.clone());
                    node_state_collections.push(NodeStateCollection {
                        id: node_state_collection_id,
                        node_state_id: node_state_id.clone(),
                        node_state_ids: other_node_state_ids
                    });
                }

                // tie nodes to their neighbors
                for (node_index, node) in std::iter::zip(0..nodes_total, nodes.iter_mut()) {
                    let node_x: i32 = node_index % nodes_width;
                    let node_y: i32 = (node_index / nodes_width) % nodes_height;
                    let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
                    let mut neighbors_total = 0;
                    //debug!("processing node {node_x}, {node_y}, {node_z}.");
                    for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                        let other_node_x: i32 = other_node_index % nodes_width;
                        let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                        let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                        if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                            //debug!("found neighbor at {other_node_x}, {other_node_y}, {other_node_z}.");
                            node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                            neighbors_total += 1;
                        }
                    }
                    //debug!("neighbors: {neighbors_total}.");
                }
            });

            let mut wave_function: WaveFunction;

            time_graph::spanned!("creating wave function", {
                wave_function = WaveFunction::new(nodes, node_state_collections);
            });

            time_graph::spanned!("validating wave function", {
                wave_function.validate().unwrap();
            });

            wave_function.sort();

            let collapsed_wave_function_result: Result<CollapsedWaveFunction, String>;
            
            time_graph::spanned!("collapsing wave function", {
                //let random_seed = Some(rng.gen::<u64>());  // TODO uncomment after fixing
                collapsed_wave_function_result = wave_function.collapse(random_seed);
            });

            if index + 1 == max_runs {
                println!("{}", time_graph::get_full_graph().as_dot());
            }

            if let Err(error_message) = collapsed_wave_function_result {
                println!("tried random seed: {:?}.", random_seed);
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            for (node_index, node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                let node_x: i32 = node_index % nodes_width;
                let node_y: i32 = (node_index / nodes_width) % nodes_height;
                let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
                for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                    let other_node_x: i32 = other_node_index % nodes_width;
                    let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                    let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                    if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                        assert_ne!(collapsed_wave_function.node_state_per_node.get(node_id), collapsed_wave_function.node_state_per_node.get(other_node_id));
                    }
                }
            }
        }
    }

    #[test]
    fn many_nodes_as_3D_grid_randomly_all_different_states_uncollapsed_wave_functions() {
        init();
        time_graph::enable_data_collection(true);

        let mut rng = rand::thread_rng();

        for _ in 0..1 {

            //let random_seed = Some(rng.next_u64());
            let random_seed = Some(3137775564618414013);

            //let random_seed = None;

            let size = 9;

            let nodes_height = size;
            let nodes_width = size;
            let nodes_depth = size;
            let nodes_total = nodes_height * nodes_width * nodes_depth;
            let node_states_total = 8;

            let mut nodes: Vec<Node> = Vec::new();
            let mut node_ids: Vec<String> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();
            let mut node_state_ids: Vec<String> = Vec::new();
            let mut node_state_collection_ids: Vec<String> = Vec::new();

            time_graph::spanned!("creating test data", {

                for index in 0..node_states_total {
                    let node_state_id: String = format!("{}{}", index, Uuid::new_v4());
                    node_state_ids.push(node_state_id);
                }

                for index in 0..nodes_total {
                    let node_id: String = format!("{}{}", index, Uuid::new_v4());
                    node_ids.push(node_id.clone());
                    nodes.push(Node { 
                        id: node_id,
                        node_state_ids: node_state_ids.clone(),
                        node_state_collection_ids_per_neighbor_node_id: HashMap::new()
                    });
                }

                for node_state_id in node_state_ids.iter() {
                    let mut other_node_state_ids: Vec<String> = Vec::new();
                    for other_node_state_id in node_state_ids.iter() {
                        if node_state_id != other_node_state_id {
                            other_node_state_ids.push(other_node_state_id.clone());
                        }
                    }
                    
                    let node_state_collection_id: String = format!("{}{}", node_state_id, Uuid::new_v4().to_string());
                    node_state_collection_ids.push(node_state_collection_id.clone());
                    node_state_collections.push(NodeStateCollection {
                        id: node_state_collection_id,
                        node_state_id: node_state_id.clone(),
                        node_state_ids: other_node_state_ids
                    });
                }

                // tie nodes to their neighbors
                for (node_index, node) in std::iter::zip(0..nodes_total, nodes.iter_mut()) {
                    let node_x: i32 = node_index % nodes_width;
                    let node_y: i32 = (node_index / nodes_width) % nodes_height;
                    let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
                    let mut neighbors_total = 0;
                    //debug!("processing node {node_x}, {node_y}, {node_z}.");
                    for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                        let other_node_x: i32 = other_node_index % nodes_width;
                        let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                        let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                        if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                            //debug!("found neighbor at {other_node_x}, {other_node_y}, {other_node_z}.");
                            node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                            neighbors_total += 1;
                        }
                    }
                    //debug!("neighbors: {neighbors_total}.");
                }
            });

            let mut wave_function: WaveFunction;

            time_graph::spanned!("creating wave function", {
                wave_function = WaveFunction::new(nodes, node_state_collections);
            });

            time_graph::spanned!("validating wave function", {
                wave_function.validate().unwrap();
            });

            wave_function.sort();

            let node_states_result: Result<Vec<NodeState>, String>;
            
            time_graph::spanned!("collapsing wave function", {
                //let random_seed = Some(rng.gen::<u64>());  // TODO uncomment after fixing
                node_states_result = wave_function.collapse_into_steps(random_seed);
            });

            if let Err(error_message) = node_states_result {
                println!("{}", time_graph::get_full_graph().as_dot());
                println!("tried random seed: {:?}.", random_seed);
                panic!("Error: {error_message}");
            }

            let node_states = node_states_result.ok().unwrap();

            // TODO assert something about the uncollapsed wave functions
            //println!("States: {:?}", node_states);
            println!("Found {:?} node states.", node_states.len());
            println!("tried random seed: {:?}.", random_seed);
        }

        println!("{}", time_graph::get_full_graph().as_dot());
    }

    #[test]
    fn write_and_read_wave_function_from_tempfile() {
        init();

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let node_state_id: String = Uuid::new_v4().to_string();

        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: Uuid::new_v4().to_string(),
            node_state_ids: vec![node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

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
        wave_function.validate().unwrap();

        let file = tempfile::NamedTempFile::new().unwrap();
        let file_path: &str = file.path().to_str().unwrap();
        debug!("Saving wave function to {:?}", file_path);
        wave_function.save_to_file(file_path);

        let loaded_wave_function = WaveFunction::load_from_file(file_path);
        loaded_wave_function.validate().unwrap();

        file.close().unwrap();

        let collapsed_wave_function = wave_function.collapse(None).unwrap();
        let loaded_collapsed_wave_function = loaded_wave_function.collapse(None).unwrap();

        assert_eq!(collapsed_wave_function.node_state_per_node, loaded_collapsed_wave_function.node_state_per_node);
    }

    #[test]
    fn four_nodes_as_square_neighbors_randomly() {
        init();

        let mut rng = rand::thread_rng();

        for _ in 0..1000 {

            let random_seed = Some(rng.next_u64());

            let mut nodes: Vec<Node> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");

            nodes.push(Node { 
                id: String::from("node_1"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_2"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_3"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_4"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });

            let one_forces_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let one_forces_two_node_state_collection = NodeStateCollection {
                id: one_forces_two_node_state_collection_id.clone(),
                node_state_id: one_node_state_id.clone(),
                node_state_ids: vec![two_node_state_id.clone()]
            };
            node_state_collections.push(one_forces_two_node_state_collection);

            let two_forces_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let two_forces_one_node_state_collection = NodeStateCollection {
                id: two_forces_one_node_state_collection_id.clone(),
                node_state_id: two_node_state_id.clone(),
                node_state_ids: vec![one_node_state_id.clone()]
            };
            node_state_collections.push(two_forces_one_node_state_collection);

            let possible_node_ids: Vec<&str> = vec!["node_1", "node_2", "node_3", "node_4"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                    if node_index != other_node_index && node_index % 2 != other_node_index % 2 {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                    }
                }
            }

            let mut wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            
            for is_sort_required in [false, true] {
                if is_sort_required {
                    wave_function.sort();
                }
                let collapsed_wave_function_result = wave_function.collapse(random_seed);

                if let Err(error_message) = collapsed_wave_function_result {
                    panic!("Error: {error_message}");
                }

                let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
            }
        }
    }

    #[test]
    fn four_nodes_as_square_neighbors_in_cycle_alone() {
        init();

        let mut rng = rand::thread_rng();

        for _ in 0..100 {

            let random_seed = Some(rng.next_u64());

            let mut nodes: Vec<Node> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");

            nodes.push(Node { 
                id: String::from("node_1"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_2"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_3"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_4"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });

            let one_forces_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let one_forces_two_node_state_collection = NodeStateCollection {
                id: one_forces_two_node_state_collection_id.clone(),
                node_state_id: one_node_state_id.clone(),
                node_state_ids: vec![two_node_state_id.clone()]
            };
            node_state_collections.push(one_forces_two_node_state_collection);

            let two_forces_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let two_forces_one_node_state_collection = NodeStateCollection {
                id: two_forces_one_node_state_collection_id.clone(),
                node_state_id: two_node_state_id.clone(),
                node_state_ids: vec![one_node_state_id.clone()]
            };
            node_state_collections.push(two_forces_one_node_state_collection);

            let possible_node_ids: Vec<&str> = vec!["node_1", "node_2", "node_3", "node_4"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                    if (node_index + 1) % 4 == other_node_index {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                    }
                }
            }

            let mut wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            
            for is_sort_required in [false, true] {
                if is_sort_required {
                    wave_function.sort();
                }
                let collapsed_wave_function_result = wave_function.collapse(random_seed);

                if let Err(error_message) = collapsed_wave_function_result {
                    panic!("Error: {error_message}");
                }

                let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
            }
        }
    }

    #[test]
    fn four_nodes_as_square_neighbors_in_cycle_affects_another_square() {
        init();

        let mut rng = rand::thread_rng();

        for _ in 0..100 {

            let random_seed = Some(rng.next_u64());

            let mut nodes: Vec<Node> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");

            let one_forces_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let one_forces_two_node_state_collection = NodeStateCollection {
                id: one_forces_two_node_state_collection_id.clone(),
                node_state_id: one_node_state_id.clone(),
                node_state_ids: vec![two_node_state_id.clone()]
            };
            node_state_collections.push(one_forces_two_node_state_collection);

            let two_forces_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let two_forces_one_node_state_collection = NodeStateCollection {
                id: two_forces_one_node_state_collection_id.clone(),
                node_state_id: two_node_state_id.clone(),
                node_state_ids: vec![one_node_state_id.clone()]
            };
            node_state_collections.push(two_forces_one_node_state_collection);

            nodes.push(Node { 
                id: String::from("node_1a"),
                node_state_ids: vec![two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_2a"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_3a"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_4a"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });

            let possible_node_ids: Vec<&str> = vec!["node_1a", "node_2a", "node_3a", "node_4a"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                    if (node_index + 1) % 4 == other_node_index {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                    }
                }
            }

            nodes.push(Node { 
                id: String::from("node_1b"),
                node_state_ids: vec![two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_2b"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_3b"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });
            nodes.push(Node { 
                id: String::from("node_4b"),
                node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
                node_state_collection_ids_per_neighbor_node_id: HashMap::new()
            });

            let possible_node_ids: Vec<&str> = vec!["node_1b", "node_2b", "node_3b", "node_4b"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                if node_index > 3 {
                    for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                        if (node_index + 1) % 4 == other_node_index {
                            node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                        }
                    }
                }
            }

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(String::from("node_1b"), vec![one_forces_two_node_state_collection_id]);

            let mut wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            
            for is_sort_required in [false, true] {
                if is_sort_required {
                    wave_function.sort();
                }
                let collapsed_wave_function_result = wave_function.collapse(random_seed);

                if let Err(error_message) = collapsed_wave_function_result {
                    panic!("Error: {error_message}");
                }

                let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
                assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
                assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
            }
        }
    }

    #[test]
    fn four_nodes_that_would_skip_over_nonneighbor() {
        init();

        // TODO add randomization

        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let one_node_id: String = String::from("node_1");
        let two_node_id: String = String::from("node_2");
        let three_node_id: String = String::from("node_3");
        let four_node_id: String = String::from("node_4");
        
        let one_node_state_id: String = String::from("state_A");
        let two_node_state_id: String = String::from("state_B");

        nodes.push(Node { 
            id: one_node_id.clone(),
            node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: two_node_id.clone(),
            node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: three_node_id.clone(),
            node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });
        nodes.push(Node { 
            id: four_node_id.clone(),
            node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()],
            node_state_collection_ids_per_neighbor_node_id: HashMap::new()
        });

        let one_node_state_id: String = String::from("state_A");
        let two_node_state_id: String = String::from("state_B");

        let one_permits_one_and_two_node_state_collection_id: String = Uuid::new_v4().to_string();
        let one_permits_one_and_two_node_state_collection = NodeStateCollection {
            id: one_permits_one_and_two_node_state_collection_id.clone(),
            node_state_id: one_node_state_id.clone(),
            node_state_ids: vec![one_node_state_id.clone(), two_node_state_id.clone()]
        };
        node_state_collections.push(one_permits_one_and_two_node_state_collection);

        let two_permits_none_node_state_collection_id: String = Uuid::new_v4().to_string();
        let two_permits_none_node_state_collection = NodeStateCollection {
            id: two_permits_none_node_state_collection_id.clone(),
            node_state_id: two_node_state_id.clone(),
            node_state_ids: vec![]
        };
        node_state_collections.push(two_permits_none_node_state_collection);

        let two_permits_one_node_state_collection_id: String = Uuid::new_v4().to_string();
        let two_permits_one_node_state_collection = NodeStateCollection {
            id: two_permits_one_node_state_collection_id.clone(),
            node_state_id: two_node_state_id.clone(),
            node_state_ids: vec![one_node_state_id.clone()]
        };
        node_state_collections.push(two_permits_one_node_state_collection);

        let one_permits_two_node_state_collection_id: String = Uuid::new_v4().to_string();
        let one_permits_two_node_state_collection = NodeStateCollection {
            id: one_permits_two_node_state_collection_id.clone(),
            node_state_id: one_node_state_id.clone(),
            node_state_ids: vec![two_node_state_id.clone()]
        };
        node_state_collections.push(one_permits_two_node_state_collection);

        let one_permits_one_node_state_collection_id: String = Uuid::new_v4().to_string();
        let one_permits_one_node_state_collection = NodeStateCollection {
            id: one_permits_one_node_state_collection_id.clone(),
            node_state_id: one_node_state_id.clone(),
            node_state_ids: vec![one_node_state_id.clone()]
        };
        node_state_collections.push(one_permits_one_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(two_node_id.clone(), vec![one_permits_one_and_two_node_state_collection_id.clone(), two_permits_none_node_state_collection_id.clone()]);
        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(three_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(one_node_id.clone(), vec![one_permits_one_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(four_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(one_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(four_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[3].node_state_collection_ids_per_neighbor_node_id.insert(two_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[3].node_state_collection_ids_per_neighbor_node_id.insert(three_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);

        let mut wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        
        for is_sort_required in [false, true] {
            if is_sort_required {
                wave_function.sort();
            }
            let collapsed_wave_function_result = wave_function.collapse(None);

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_eq!(&one_node_state_id, collapsed_wave_function.node_state_per_node.get(&one_node_id).unwrap());
            assert_eq!(&two_node_state_id, collapsed_wave_function.node_state_per_node.get(&two_node_id).unwrap());
            assert_eq!(&two_node_state_id, collapsed_wave_function.node_state_per_node.get(&three_node_id).unwrap());
            assert_eq!(&one_node_state_id, collapsed_wave_function.node_state_per_node.get(&four_node_id).unwrap());
        }
    }
}