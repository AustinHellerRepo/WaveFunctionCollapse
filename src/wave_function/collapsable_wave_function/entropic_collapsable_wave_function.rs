use std::ops::{BitOr, BitOrAssign};
use std::collections::HashMap;
use std::hash::Hash;
use bitvec::vec::BitVec;
use indexmap::IndexMap;

use super::collapsable_wave_function::{CollapsableNode, CollapsableWaveFunction, CollapsedNodeState, CollapsedWaveFunction};

pub struct EntropicCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
    collapsable_nodes_length: usize,
    current_collapsable_node_index: usize,
    collapsed_nodes_total: usize,
    is_node_collapsed: BitVec,
    cached_mask_per_neighbor_index: IndexMap<usize, BitVec>,
    popped_neighbor_index: Option<usize>,
    popped_mask: Option<BitVec>,
    possible_states_from_popped_neighbor: Vec<&'a TNodeState>,
    great_neighbors_from_popped_neighbor: Vec<usize>,
    great_neighbors_from_popped_neighbor_length: usize,
    explored_great_neighbor_index: Option<usize>,
    collected_masks_for_each_possible_state_for_currently_explored_neighbor: Vec<BitVec>,
    calculated_flattened_mask: Option<BitVec>,
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
                let current_entropy_value = self.collapsable_nodes[index].node_state_indexed_view.entropy();
                if let Some(lowest_entropy_value) = lowest_entropy {
                    if current_entropy_value < lowest_entropy_value {
                        lowest_entropy = Some(current_entropy_value);
                        lowest_entropy_index = Some(index);
                    }
                } else {
                    lowest_entropy = Some(current_entropy_value);
                    lowest_entropy_index = Some(index);
                }
            }
        }
        self.current_collapsable_node_index = lowest_entropy_index.unwrap();
    }

    fn try_increment_current_collapsable_node_state(&mut self) -> CollapsedNodeState<TNodeState> {
        let current_collapsable_node = &mut self.collapsable_nodes[self.current_collapsable_node_index];

        let is_successful = current_collapsable_node.node_state_indexed_view.try_move_next();
        let collapsed_node_state: CollapsedNodeState<TNodeState>;
        if is_successful {
            current_collapsable_node.current_chosen_from_sort_index = Some(self.current_collapsable_node_index);
            collapsed_node_state = CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: Some((*current_collapsable_node.node_state_indexed_view.get().unwrap()).clone()),
            };
        } else {
            current_collapsable_node.current_chosen_from_sort_index = None;
            collapsed_node_state = CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: None,
            };
        }
        self.is_node_collapsed.set(self.current_collapsable_node_index, true);
        self.collapsed_nodes_total += 1;

        collapsed_node_state
    }

    fn cache_neighbor_node_and_mask_pairs(&mut self) {
        let masks: Vec<(usize, BitVec)> = {
            let node = &self.collapsable_nodes[self.current_collapsable_node_index];
            let mut m = Vec::new();
            if let Some(state) = node.node_state_indexed_view.get() {
                if let Some(mask_per_neighbor) = node.mask_per_neighbor_per_state.get(state) {
                    for &neighbor_index in &node.neighbor_node_indices {
                        if let Some(mask) = mask_per_neighbor.get(&neighbor_index) {
                            m.push((neighbor_index, mask.clone()));
                        }
                    }
                }
            }
            m
        };
        for (idx, mask) in masks {
            self.cached_mask_per_neighbor_index.insert(idx, mask);
        }
    }

    fn is_cached_neighbor_node_and_mask_pairs_empty(&self) -> bool {
        self.cached_mask_per_neighbor_index.is_empty()
    }

    fn pop_first_neighbor_node_and_mask(&mut self) {
        let (neighbor_index, mask) = self.cached_mask_per_neighbor_index.pop().unwrap();
        self.popped_neighbor_index = Some(neighbor_index);
        self.popped_mask = Some(mask);
        debug!("popped neighbor {:?} with mask {:?}", self.popped_neighbor_index, self.popped_mask);
    }

    fn try_apply_popped_mask_to_neighbor_node_and_collect_possible_states_and_great_neighbors(&mut self) -> bool {
        let popped_neighbor_index = self.popped_neighbor_index.unwrap();
        let mask = self.popped_mask.as_ref().unwrap();
        self.collapsable_nodes[popped_neighbor_index].node_state_indexed_view.add_mask(mask);
        if self.collapsable_nodes[popped_neighbor_index].is_fully_restricted() {
            debug!("is fully restricted after applying mask");
            false
        } else {
            self.possible_states_from_popped_neighbor = self.collapsable_nodes[popped_neighbor_index].node_state_indexed_view.get_possible_states();
            self.great_neighbors_from_popped_neighbor = self.collapsable_nodes[popped_neighbor_index].neighbor_node_indices.clone();
            self.great_neighbors_from_popped_neighbor_length = self.great_neighbors_from_popped_neighbor.len();
            debug!("is not fully restricted after applying mask");
            if self.collapsable_nodes[popped_neighbor_index].node_state_indexed_view.is_mask_restrictive(mask) {
                panic!("mask cannot be restrictive after just being added");
            }
            true
        }
    }

    fn prepare_to_explore_each_great_neighbor_of_popped_neighbor(&mut self) {
        self.explored_great_neighbor_index = None;
    }

    fn is_every_great_neighbor_explored(&self) -> bool {
        if let Some(index) = self.explored_great_neighbor_index {
            index + 1 == self.great_neighbors_from_popped_neighbor_length
        } else {
            self.great_neighbors_from_popped_neighbor_length == 0
        }
    }

    fn explore_next_great_neighbor_node(&mut self) {
        if let Some(index) = self.explored_great_neighbor_index {
            self.explored_great_neighbor_index = Some(index + 1);
        } else {
            self.explored_great_neighbor_index = Some(0);
        }
    }

    fn collect_masks_for_each_possible_state_of_popped_neighbor_for_currently_explored_great_neighbor(&mut self) {
        self.collected_masks_for_each_possible_state_for_currently_explored_neighbor.clear();
        let popped_neighbor_index = self.popped_neighbor_index.unwrap();
        let explored_great_neighbor_index = self.great_neighbors_from_popped_neighbor[self.explored_great_neighbor_index.unwrap()];
        let masks: Vec<BitVec> = {
            let node = &self.collapsable_nodes[popped_neighbor_index];
            let mut m = Vec::new();
            for possible_state in self.possible_states_from_popped_neighbor.iter() {
                if let Some(mask_per_neighbor) = node.mask_per_neighbor_per_state.get(possible_state) {
                    if let Some(mask) = mask_per_neighbor.get(&explored_great_neighbor_index) {
                        m.push(mask.clone());
                    }
                }
            }
            m
        };
        self.collected_masks_for_each_possible_state_for_currently_explored_neighbor = masks;
    }

    fn calculate_flattened_mask(&mut self) {
        if !self.collected_masks_for_each_possible_state_for_currently_explored_neighbor.is_empty() {
            let mut flattened_mask: Option<BitVec> = None;
            for mask in self.collected_masks_for_each_possible_state_for_currently_explored_neighbor.iter() {
                if let Some(ref mut fm) = flattened_mask {
                    fm.bitor_assign(mask);
                } else {
                    flattened_mask = Some(mask.clone());
                }
            }
            self.calculated_flattened_mask = flattened_mask;
        }
    }

    fn is_flattened_mask_restrictive_to_explored_neighbor(&self) -> bool {
        if let Some(flattened_mask_value) = self.calculated_flattened_mask.as_ref() {
            let explored_index = self.great_neighbors_from_popped_neighbor[self.explored_great_neighbor_index.unwrap()];
            let is_restrictive = self.collapsable_nodes[explored_index].node_state_indexed_view.is_mask_restrictive(flattened_mask_value);
            if is_restrictive {
                debug!("great neighbor {:?} would be restricted by {:?}", explored_index, flattened_mask_value);
            }
            is_restrictive
        } else {
            false
        }
    }

    fn append_explored_neighbor_and_flattened_mask_to_cache_of_neighbor_node_and_mask_pairs(&mut self) {
        let explored_index = self.great_neighbors_from_popped_neighbor[self.explored_great_neighbor_index.unwrap()];
        let mask_to_add = self.calculated_flattened_mask.as_ref().unwrap().clone();
        if let Some(existing) = self.cached_mask_per_neighbor_index.shift_remove(&explored_index) {
            self.cached_mask_per_neighbor_index.insert(explored_index, existing.clone().bitor(&mask_to_add));
        } else {
            self.cached_mask_per_neighbor_index.insert(explored_index, mask_to_add);
        }
        self.calculated_flattened_mask = None;
        debug!("pushed to back with length {:?}", self.cached_mask_per_neighbor_index.keys().len());
    }

    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut node_state_per_node_id: HashMap<String, TNodeState> = HashMap::new();
        for collapsable_node in self.collapsable_nodes.iter() {
            let node_state: TNodeState = (*collapsable_node.node_state_indexed_view.get().unwrap()).clone();
            let node_id: String = String::from(collapsable_node.id);
            debug!("established node {node_id} in state {:?}.", node_state);
            node_state_per_node_id.insert(node_id, node_state);
        }
        CollapsedWaveFunction {
            node_state_per_node_id,
        }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for EntropicCollapsableWaveFunction<'a, TNodeState> {
    fn new(
        collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
        _random_instance: fastrand::Rng,
    ) -> Self {
        let collapsable_nodes_length: usize = collapsable_nodes.len();
        let mut is_node_collapsed: BitVec = BitVec::new();
        for _ in 0..collapsable_nodes_length {
            is_node_collapsed.push(false);
        }
        EntropicCollapsableWaveFunction {
            collapsable_nodes,
            collapsable_nodes_length,
            current_collapsable_node_index: 0,
            collapsed_nodes_total: 0,
            is_node_collapsed,
            cached_mask_per_neighbor_index: IndexMap::new(),
            popped_neighbor_index: None,
            popped_mask: None,
            possible_states_from_popped_neighbor: Vec::new(),
            great_neighbors_from_popped_neighbor: Vec::new(),
            great_neighbors_from_popped_neighbor_length: 0,
            explored_great_neighbor_index: None,
            collected_masks_for_each_possible_state_for_currently_explored_neighbor: Vec::new(),
            calculated_flattened_mask: None,
        }
    }

    fn collapse_into_steps(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {
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
            } else {
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
                    } else {
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
                            } else {
                                debug!("is not restrictive");
                            }
                        }
                    }
                }
            }
        }

        Ok(collapsed_node_states)
    }

    fn collapse(&mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {
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
            } else {
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
                    } else {
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
                            } else {
                                debug!("is not restrictive");
                            }
                        }
                    }
                }
            }
        }

        if is_unable_to_collapse {
            Err(String::from("Cannot collapse wave function."))
        } else {
            let collapsed_wave_function = self.get_collapsed_wave_function();
            Ok(collapsed_wave_function)
        }
    }
}
