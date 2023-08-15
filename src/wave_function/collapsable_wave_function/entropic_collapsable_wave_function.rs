use std::ops::{BitOr, BitOrAssign};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use bitvec::vec::BitVec;
use indexmap::IndexMap;

use super::collapsable_wave_function::{CollapsableNode, CollapsableWaveFunction, CollapsedNodeState, CollapsedWaveFunction};

pub struct EntropicCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_nodes_length: usize,
    current_collapsable_node_index: usize,
    collapsed_nodes_total: usize,
    is_node_collapsed: BitVec,
    cached_mask_per_neighbor_node_id: IndexMap<String, BitVec>,
    popped_neighbor_node_id: Option<String>,
    popped_mask: Option<BitVec>,
    possible_states_from_popped_neighbor: Vec<&'a TNodeState>,
    great_neighbors_from_popped_neighbor: Vec<&'a str>,
    great_neighbors_from_popped_neighbor_length: usize,
    explored_great_neighbor_node_index: Option<usize>,
    collected_masks_for_each_possible_state_for_currently_explored_neighbor: Vec<BitVec>,
    calculated_flattened_mask: Option<BitVec>,
    node_state_type: PhantomData<TNodeState>
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> EntropicCollapsableWaveFunction<'a, TNodeState> {
    fn is_fully_collapsed(&self) -> bool {
        self.collapsable_nodes_length == self.collapsed_nodes_total
    }
    fn set_current_collapsable_node_to_least_entropic_collapsable_node(&mut self) {
        let mut lowest_entropy: Option<f32> = None;
        let mut lowest_entropy_index: Option<usize> = None;
        for index in 0..self.collapsable_nodes_length {
            if !self.is_node_collapsed[index] {
                let wrapped_collapsable_node = self.collapsable_nodes.get(index).unwrap();
                let mut collapsable_node = wrapped_collapsable_node.borrow_mut();
                if let Some(lowest_entropy_value) = lowest_entropy {
                    let current_entropy_value = collapsable_node.node_state_indexed_view.entropy();
                    if current_entropy_value < lowest_entropy_value {
                        lowest_entropy = Some(current_entropy_value);
                        lowest_entropy_index = Some(index);
                    }
                }
                else {
                    lowest_entropy = Some(collapsable_node.node_state_indexed_view.entropy());
                    lowest_entropy_index = Some(index);
                }
            }
        }
        self.current_collapsable_node_index = lowest_entropy_index.unwrap();
    }
    fn try_increment_current_collapsable_node_state(&mut self) -> CollapsedNodeState<TNodeState> {

        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

        let is_successful = current_collapsable_node.node_state_indexed_view.try_move_next();
        let collapsed_node_state: CollapsedNodeState<TNodeState>;
        if is_successful {
            current_collapsable_node.current_chosen_from_sort_index = Some(self.current_collapsable_node_index);
            collapsed_node_state = CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: Some((*current_collapsable_node.node_state_indexed_view.get().unwrap()).clone())
            };
        }
        else {
            current_collapsable_node.current_chosen_from_sort_index = None;
            collapsed_node_state = CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: None
            };
        }
        self.is_node_collapsed.set(self.current_collapsable_node_index, true);
        self.collapsed_nodes_total += 1;

        collapsed_node_state
    }
    fn cache_neighbor_node_and_mask_pairs(&mut self) {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        let current_possible_state = current_collapsable_node.node_state_indexed_view.get().unwrap();
        let neighbor_node_ids: &Vec<&str> = &current_collapsable_node.neighbor_node_ids;
        let mask_per_neighbor_per_state: &HashMap<&TNodeState, HashMap<&str, BitVec>> = &current_collapsable_node.mask_per_neighbor_per_state;
        if let Some(mask_per_neighbor) = mask_per_neighbor_per_state.get(current_possible_state) {
            for neighbor_node_id in neighbor_node_ids.iter() {
                if mask_per_neighbor.contains_key(neighbor_node_id) {
                    let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                    self.cached_mask_per_neighbor_node_id.insert(String::from(*neighbor_node_id), mask.clone());
                }
            }
        }
    }
    fn is_cached_neighbor_node_and_mask_pairs_empty(&self) -> bool {
        self.cached_mask_per_neighbor_node_id.is_empty()
    }
    fn pop_first_neighbor_node_and_mask(&mut self) {
        let (neighbor_node_id, mask) = self.cached_mask_per_neighbor_node_id.pop().unwrap();
        self.popped_neighbor_node_id = Some(neighbor_node_id.to_owned());
        self.popped_mask = Some(mask);
        debug!("popped neighbor {:?} with mask {:?}", self.popped_neighbor_node_id, self.popped_mask);
    }
    fn try_apply_popped_mask_to_neighbor_node_and_collect_possible_states_and_great_neighbors(&mut self) -> bool {
        let popped_neighbor_node_id = self.popped_neighbor_node_id.as_ref().unwrap();
        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(popped_neighbor_node_id.as_str()).unwrap();
        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
        let mask = self.popped_mask.as_ref().unwrap();
        neighbor_collapsable_node.node_state_indexed_view.add_mask(mask);
        if neighbor_collapsable_node.is_fully_restricted() {
            debug!("is fully restricted after applying mask");
            false
        }
        else {
            self.possible_states_from_popped_neighbor = neighbor_collapsable_node.node_state_indexed_view.get_possible_states();
            self.great_neighbors_from_popped_neighbor = neighbor_collapsable_node.neighbor_node_ids.clone();
            self.great_neighbors_from_popped_neighbor_length = self.great_neighbors_from_popped_neighbor.len();
            debug!("is not fully restricted after applying mask");
            if neighbor_collapsable_node.node_state_indexed_view.is_mask_restrictive(mask) {
                panic!("mask cannot be restrictive after just being added");
            }
            true
        }
    }
    fn prepare_to_explore_each_great_neighbor_of_popped_neighbor(&mut self) {
        self.explored_great_neighbor_node_index = None;
    }
    fn is_every_great_neighbor_explored(&self) -> bool {
        if let Some(index) = self.explored_great_neighbor_node_index {
            index + 1 == self.great_neighbors_from_popped_neighbor_length
        }
        else {
            self.great_neighbors_from_popped_neighbor_length == 0
        }
    }
    fn explore_next_great_neighbor_node(&mut self) {
        if let Some(index) = self.explored_great_neighbor_node_index {
            self.explored_great_neighbor_node_index = Some(index + 1);
        }
        else {
            self.explored_great_neighbor_node_index = Some(0);
        }
    }
    fn collect_masks_for_each_possible_state_of_popped_neighbor_for_currently_explored_great_neighbor(&mut self) {
        self.collected_masks_for_each_possible_state_for_currently_explored_neighbor.clear();
        let popped_neighbor_node_id: &str = self.popped_neighbor_node_id.as_ref().unwrap();
        let wrapped_popped_neighbor_collapsable_node = self.collapsable_node_per_id.get(popped_neighbor_node_id).unwrap();
        let popped_neighbor_collapsable_node = wrapped_popped_neighbor_collapsable_node.borrow();
        let explored_great_neighbor_node_id = self.great_neighbors_from_popped_neighbor[self.explored_great_neighbor_node_index.unwrap()];
        for possible_state in self.possible_states_from_popped_neighbor.iter() {
            if popped_neighbor_collapsable_node.mask_per_neighbor_per_state.contains_key(possible_state) {
                let mask_per_neighbor = popped_neighbor_collapsable_node.mask_per_neighbor_per_state.get(possible_state).unwrap();
                if mask_per_neighbor.contains_key(explored_great_neighbor_node_id) {
                    let mask = mask_per_neighbor.get(explored_great_neighbor_node_id).unwrap();
                    self.collected_masks_for_each_possible_state_for_currently_explored_neighbor.push(mask.clone());
                }
            }
        }
    }
    fn calculate_flattened_mask(&mut self) {
        // TODO compress "collect_masks_for_each_possible_state_of_popped_neighbor_for_currently_explored_great_neighbor", "calculate_flattened_mask", and "is_flattened_mask_restrictive_to_explored_neighbor" into one function
        if !self.collected_masks_for_each_possible_state_for_currently_explored_neighbor.is_empty() {
            let mut flattened_mask: Option<BitVec> = None;
            for mask in self.collected_masks_for_each_possible_state_for_currently_explored_neighbor.iter() {
                if let Some(flattened_mask_value) = flattened_mask {
                    let bitwise_or_mask = flattened_mask_value.bitor(mask);
                    flattened_mask = Some(bitwise_or_mask);
                }
                else {
                    flattened_mask = Some(mask.clone());
                }
            }
            self.calculated_flattened_mask = flattened_mask;
        }
    }
    fn is_flattened_mask_restrictive_to_explored_neighbor(&self) -> bool {
        if let Some(flattened_mask_value) = self.calculated_flattened_mask.as_ref() {
            let explored_great_neighbor_node_id = self.great_neighbors_from_popped_neighbor[self.explored_great_neighbor_node_index.unwrap()];
            let wrapped_explored_great_neighbor_collapsable_node = self.collapsable_node_per_id.get(explored_great_neighbor_node_id).unwrap();
            let explored_great_neighbor_collapsable_node = wrapped_explored_great_neighbor_collapsable_node.borrow();
            let is_restrictive = explored_great_neighbor_collapsable_node.node_state_indexed_view.is_mask_restrictive(flattened_mask_value);
            if is_restrictive {
                debug!("great neighbor {:?} would be restricted by {:?}", explored_great_neighbor_node_id, flattened_mask_value);
            }
            is_restrictive
        }
        else {
            false
        }
    }
    fn append_explored_neighbor_and_flattened_mask_to_cache_of_neighbor_node_and_mask_pairs(&mut self) {
        let explored_great_neighbor_node_id = String::from(self.great_neighbors_from_popped_neighbor[self.explored_great_neighbor_node_index.unwrap()]);
        if self.cached_mask_per_neighbor_node_id.contains_key(&explored_great_neighbor_node_id) {
            let mut existing_mask = self.cached_mask_per_neighbor_node_id.remove(&explored_great_neighbor_node_id).unwrap();
            existing_mask.bitor_assign(self.calculated_flattened_mask.as_ref().unwrap());
            self.cached_mask_per_neighbor_node_id.insert(explored_great_neighbor_node_id, existing_mask);
        }
        else {
            self.cached_mask_per_neighbor_node_id.insert(explored_great_neighbor_node_id, self.calculated_flattened_mask.as_ref().unwrap().clone());
        }
        self.calculated_flattened_mask = None;
        debug!("pushed to back with length {:?}", self.cached_mask_per_neighbor_node_id.keys().len());
    }
    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut node_state_per_node: HashMap<String, TNodeState> = HashMap::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            let node_state: TNodeState = (*collapsable_node.node_state_indexed_view.get().unwrap()).clone();
            let node: String = String::from(collapsable_node.id);
            debug!("established node {node} in state {:?}.", node_state);
            node_state_per_node.insert(node, node_state);
        }
        CollapsedWaveFunction {
            node_state_per_node
        }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for EntropicCollapsableWaveFunction<'a, TNodeState> {
    fn new(collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>, collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>) -> Self {
        let collapsable_nodes_length: usize = collapsable_nodes.len();
        let mut is_node_collapsed: BitVec = BitVec::new();
        for _ in 0..collapsable_nodes_length {
            is_node_collapsed.push(false);
        }
        EntropicCollapsableWaveFunction {
            collapsable_nodes,
            collapsable_node_per_id,
            collapsable_nodes_length,
            current_collapsable_node_index: 0,
            collapsed_nodes_total: 0,
            is_node_collapsed,
            cached_mask_per_neighbor_node_id: IndexMap::new(),
            popped_neighbor_node_id: None,
            popped_mask: None,
            possible_states_from_popped_neighbor: Vec::new(),
            great_neighbors_from_popped_neighbor: Vec::new(),
            great_neighbors_from_popped_neighbor_length: 0,
            explored_great_neighbor_node_index: None,
            collected_masks_for_each_possible_state_for_currently_explored_neighbor: Vec::new(),
            calculated_flattened_mask: None,
            node_state_type: PhantomData
        }
    }
    fn collapse_into_steps(&'a mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {

        // while not yet fully collapsed and is still able to collapse
        //      find least entropic node not yet collapsed
        //      try to choose next state
        //      if unsuccessful in choosing next state
        //          set unable to collapse wave function
        //      else
        //          cache neighbor node ids and altering masks respectively
        //          while at least one pair of collapsable node id and mask still exists in the pair cache and is still able to collapse
        //              pop first pair
        //              try to apply mask to collapsable node
        //              if unsuccessful in applying mask
        //                  set unable to collapse wave function
        //              else
        //                  collect states that are possible for this collapsable node
        //                  for each neighbor of this collapsable node
        //                      collect the masks for each state from the prior collected states for this neighbor
        //                      perform a bitwise OR over all of the masks
        //                      if the bitwised mask would be newly restrictive to this neighbor
        //                          append this neighbor node id and bitwise mask respectively to the pair cache

        let mut collapsed_node_states: Vec<CollapsedNodeState<TNodeState>> = Vec::new();
        let mut is_unable_to_collapse = false;
        debug!("starting main while loop");
        while !self.is_fully_collapsed() && !is_unable_to_collapse {
            debug!("finding least entropic collapsable node");
            self.set_current_collapsable_node_to_least_entropic_collapsable_node();
            debug!("try incrementing current collapsable node state");
            let collapsed_node_state = self.try_increment_current_collapsable_node_state();
            let is_successful: bool = collapsed_node_state.node_state_id.is_some();
            collapsed_node_states.push(collapsed_node_state);
            if !is_successful {
                debug!("failed to increment node");
                is_unable_to_collapse = true;
            }
            else {
                debug!("succeeded to increment node and caching pairs");
                self.cache_neighbor_node_and_mask_pairs();
                debug!("starting neighbor node and mask pairs while loop");
                while !self.is_cached_neighbor_node_and_mask_pairs_empty() {
                    debug!("popping first neighbor node and mask");
                    self.pop_first_neighbor_node_and_mask();
                    debug!("trying to apply popped mask to neighbor node (etc.)");
                    let is_successful = self.try_apply_popped_mask_to_neighbor_node_and_collect_possible_states_and_great_neighbors();
                    if !is_successful {
                        debug!("failed to apply popped mask");
                        is_unable_to_collapse = true;
                    }
                    else {
                        debug!("succeeded to apply popped mask and preparing to explore great neighbors");
                        self.prepare_to_explore_each_great_neighbor_of_popped_neighbor();
                        debug!("while not every great neighbor has been explored");
                        while !self.is_every_great_neighbor_explored() {
                            debug!("incrementing to next great neighbor node");
                            self.explore_next_great_neighbor_node();
                            debug!("collecting masks");
                            self.collect_masks_for_each_possible_state_of_popped_neighbor_for_currently_explored_great_neighbor();
                            debug!("calculate flattened mask");
                            self.calculate_flattened_mask();
                            let is_restrictive = self.is_flattened_mask_restrictive_to_explored_neighbor();
                            if is_restrictive {
                                debug!("is restrictive");
                                self.append_explored_neighbor_and_flattened_mask_to_cache_of_neighbor_node_and_mask_pairs();
                            }
                            else {
                                debug!("is not restrictive");
                            }
                        }
                    }
                }
            }
        }

        Ok(collapsed_node_states)
    }
    fn collapse(&'a mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {

        let mut is_unable_to_collapse = false;
        debug!("starting main while loop");
        while !self.is_fully_collapsed() && !is_unable_to_collapse {
            debug!("finding least entropic collapsable node");
            self.set_current_collapsable_node_to_least_entropic_collapsable_node();
            debug!("try incrementing current collapsable node state");
            let collapsed_node_state = self.try_increment_current_collapsable_node_state();
            let is_successful: bool = collapsed_node_state.node_state_id.is_some();
            if !is_successful {
                debug!("failed to increment node");
                is_unable_to_collapse = true;
            }
            else {
                debug!("succeeded to increment node and caching pairs");
                self.cache_neighbor_node_and_mask_pairs();
                debug!("starting neighbor node and mask pairs while loop");
                while !self.is_cached_neighbor_node_and_mask_pairs_empty() {
                    debug!("popping first neighbor node and mask");
                    self.pop_first_neighbor_node_and_mask();
                    debug!("trying to apply popped mask to neighbor node (etc.)");
                    let is_successful = self.try_apply_popped_mask_to_neighbor_node_and_collect_possible_states_and_great_neighbors();
                    if !is_successful {
                        debug!("failed to apply popped mask");
                        is_unable_to_collapse = true;
                    }
                    else {
                        debug!("succeeded to apply popped mask and preparing to explore great neighbors");
                        self.prepare_to_explore_each_great_neighbor_of_popped_neighbor();
                        debug!("while not every great neighbor has been explored");
                        while !self.is_every_great_neighbor_explored() {
                            debug!("incrementing to next great neighbor node");
                            self.explore_next_great_neighbor_node();
                            debug!("collecting masks");
                            self.collect_masks_for_each_possible_state_of_popped_neighbor_for_currently_explored_great_neighbor();
                            debug!("calculate flattened mask");
                            self.calculate_flattened_mask();
                            let is_restrictive = self.is_flattened_mask_restrictive_to_explored_neighbor();
                            if is_restrictive {
                                debug!("is restrictive");
                                self.append_explored_neighbor_and_flattened_mask_to_cache_of_neighbor_node_and_mask_pairs();
                            }
                            else {
                                debug!("is not restrictive");
                            }
                        }
                    }
                }
            }
        }

        if is_unable_to_collapse {
            Err(String::from("Cannot collapse wave function."))
        }
        else {
            let collapsed_wave_function = self.get_collapsed_wave_function();
            Ok(collapsed_wave_function)
        }
    }
}