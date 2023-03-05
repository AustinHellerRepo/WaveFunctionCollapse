use std::collections::VecDeque;
use std::fmt::{Debug};
use std::hash::Hash;
use std::{collections::HashMap};
use bitvec::prelude::*;
use rand::{Rng};
use crate::wave_function::probability_container::ProbabilityContainer;

/// This struct represents a stashed state of the IndexedView.
pub struct IndexedViewMaskState {
    mask_counter: Vec<u32>,
    is_restricted_at_index: BitVec
}

/// This struct represents a collection that can be incremented from an unstarted state to each sequential state provided. As masks are provided that either restrict or permit certain states, they will be skipped when performing try_move_next.
pub struct IndexedView<TNodeState: Clone + Eq + Hash + Debug> {
    // items are states of the node
    node_state_ids: Vec<TNodeState>,
    node_state_ratios: Vec<f32>,
    index_per_node_state_id: HashMap<TNodeState, usize>,
    node_state_ids_length: usize,
    index: Option<usize>,
    index_mapping: Vec<usize>,
    mask_counter: Vec<u32>,
    is_restricted_at_index: BitVec,
    is_mask_dirty: bool,
    is_fully_restricted: bool,
    previous_mask_counters: VecDeque<Vec<u32>>,
    previous_is_restricted_at_index: VecDeque<BitVec>,
    entropy: Option<f32>
}

impl<TNodeState: Clone + Eq + Hash + Debug> IndexedView<TNodeState> {
    pub fn new(node_state_ids: Vec<TNodeState>, node_state_ratios: Vec<f32>) -> Self {
        let node_state_ids_length: usize = node_state_ids.len();
        let mut index_per_node_state_id: HashMap<TNodeState, usize> = HashMap::new();
        let mut index_mapping = Vec::new();
        let mut mask_counter: Vec<u32> = Vec::new();
        let mut is_restricted_at_index: BitVec = BitVec::new();
        for (index, node_state_id) in node_state_ids.iter().enumerate() {
            index_per_node_state_id.insert(node_state_id.clone(), index);
            index_mapping.push(index);
            mask_counter.push(0);
            is_restricted_at_index.push(false);
        }
        IndexedView {
            node_state_ids,
            node_state_ratios,
            index_per_node_state_id,
            node_state_ids_length,
            index: Option::None,
            index_mapping,
            mask_counter,
            is_restricted_at_index,
            is_mask_dirty: true,
            is_fully_restricted: false,
            previous_mask_counters: VecDeque::new(),
            previous_is_restricted_at_index: VecDeque::new(),
            entropy: None
        }
    }
    pub fn shuffle<R: Rng + ?Sized>(&mut self, random_instance: &mut R) {
        if self.index.is_some() {
            panic!("Can only be shuffled prior to use.");
        }

        self.index_mapping.clear();
        let mut probability_container = ProbabilityContainer::default();
        for (node_state_id, ratio) in std::iter::zip(self.node_state_ids.iter(), self.node_state_ratios.iter()) {
            probability_container.push(node_state_id, *ratio);
        }

        for _ in 0..self.node_state_ids_length {
            let node_state_id = probability_container.pop_random(random_instance).unwrap();
            self.index_mapping.push(*self.index_per_node_state_id.get(node_state_id).unwrap());
        }

        debug!("randomized index mapping to {:?}.", self.index_mapping);
    }
    pub fn try_move_next(&mut self) -> bool {
        let mut is_unmasked = false;
        let mut next_index: usize;

        if let Some(index) = self.index {
            debug!("trying to get next state starting with {index} and ending prior to {}.", self.node_state_ids_length);
        }
        else {
            debug!("trying to get next state starting with None and ending prior to {}.", self.node_state_ids_length);
        }

        while self.index.is_none() || (self.index.unwrap() < self.node_state_ids_length && !is_unmasked) {
            if let Some(index) = self.index {
                next_index = index + 1;
            }
            else {
                next_index = 0;
            }

            debug!("incrementing index to {next_index}.");

            self.index = Some(next_index);

            if next_index != self.node_state_ids_length {
                is_unmasked = self.is_unmasked_at_index(next_index);
            }
        }
        self.index.unwrap() != self.node_state_ids_length
    }
    pub fn move_next(&mut self) {
        let mut next_index: usize;
        if let Some(index) = self.index {
            next_index = index + 1;
            if next_index == self.node_state_ids_length {
                next_index = 0;
            }
        }
        else {
            next_index = 0;
        }
        self.index = Some(next_index);
    }
    pub fn try_move_next_cycle(&mut self, terminal_node_state: &TNodeState) -> bool {
        let mut is_unmasked = false;
        let mut next_index: usize;

        if let Some(index) = self.index {
            debug!("trying to get next state while cycling starting with {index} and cycling at {}.", self.node_state_ids_length);
        }
        else {
            debug!("trying to get next state while cycling starting with None and cycling at {}.", self.node_state_ids_length);
        }

        let terminal_node_state_index: usize = *self.index_per_node_state_id.get(terminal_node_state).unwrap();

        let mut is_incremented_at_least_once: bool = false;
        let mut is_current_state_terminal_node_state: bool = false;
        while !is_incremented_at_least_once || (!is_current_state_terminal_node_state && !is_unmasked) {
            is_incremented_at_least_once = true;

            if let Some(index) = self.index {
                next_index = index + 1;
                if next_index == self.node_state_ids_length {
                    next_index = 0;
                }
            }
            else {
                next_index = 0;
            }

            debug!("incrementing or cycled index to {next_index}.");

            self.index = Some(next_index);

            if next_index == terminal_node_state_index {
                is_current_state_terminal_node_state = true;
            }
            else {
                is_unmasked = self.is_unmasked_at_index(next_index);
            }
        }
        is_unmasked
    }
    pub fn move_next_cycle(&mut self) {
        let mut next_index: usize;

        if let Some(index) = self.index {
            debug!("trying to get next state while cycling starting with {index} and cycling at {}.", self.node_state_ids_length);
        }
        else {
            debug!("trying to get next state while cycling starting with None and cycling at {}.", self.node_state_ids_length);
        }

        if let Some(index) = self.index {
            next_index = index + 1;
            if next_index == self.node_state_ids_length {
                next_index = 0;
            }
        }
        else {
            next_index = 0;
        }

        debug!("incrementing or cycled index to {next_index}.");

        self.index = Some(next_index);
    }
    fn is_unmasked_at_index(&self, index: usize) -> bool {
        //debug!("checking if unmasked at index {index} for node {mask_key}.");
        let mapped_index = self.index_mapping[index];
        !self.is_restricted_at_index[mapped_index]
    }
    pub fn is_mask_restrictive_to_current_state(&self, mask: &BitVec) -> bool {
        if let Some(index) = self.index {
            let mapped_index = self.index_mapping[index];
            
            /*if is_restrictive {
                debug!("mask is restrictive at index {:?} after mapping to index {:?} for mask {:?}", index, mapped_index, mask);
            }
            else {
                debug!("mask is not restrictive at index {:?} after mapping to index {:?} for mask {:?}", index, mapped_index, mask);
            }*/
            !mask[mapped_index]
        }
        else {
            debug!("was not restrictive because not currently in a state");
            false
        }
    }
    pub fn get(&self) -> Option<&TNodeState> {
        let value: Option<&TNodeState>;
        if let Some(index) = self.index {
            if index == self.node_state_ids_length {
                // TODO determine when this is the case; why not set self.index to None?
                value = None;
            }
            else {
                let mapped_index = self.index_mapping[index];
                value = self.node_state_ids.get(mapped_index);
            }
        }
        else {
            value = None;
        }
        value
    }
    pub fn reset(&mut self) {
        self.index = Option::None;
        // NOTE: the mask_counter should not be fully reverted to ensure that the neighbor restrictions are still being considered
    }
    pub fn is_current_state_restricted(&self) -> bool {
        let is_restricted: bool;
        if let Some(index) = self.index {
            is_restricted = !self.is_unmasked_at_index(index);
        }
        else {
            is_restricted = false;
        }
        is_restricted
    }
    pub fn is_fully_restricted(&mut self) -> bool {
        if self.is_mask_dirty {
            self.is_fully_restricted = self.is_restricted_at_index.count_ones() == self.node_state_ids_length;
            self.is_mask_dirty = false;
        }
        self.is_fully_restricted
    }
    pub fn add_mask(&mut self, mask: &BitVec) {
        //debug!("adding mask {:?} at current state {:?}.", mask, self.mask_counter);
        for index in 0..self.node_state_ids_length {
            if !mask[index] {
                //debug!("adding mask at {index}");
                let next_mask_counter = self.mask_counter[index] + 1;
                self.mask_counter[index] = next_mask_counter;
                if next_mask_counter == 1 {
                    self.is_restricted_at_index.set(index, true);
                    self.is_mask_dirty = true;
                    self.entropy = None;
                }
            }
        }
        //debug!("added mask {:?} at current state {:?}.", mask, self.mask_counter);
    }
    pub fn subtract_mask(&mut self, mask: &BitVec) {
        //debug!("removing mask {:?} at current state {:?}.", mask, self.mask_counter);
        for index in 0..self.node_state_ids_length {
            if !mask[index] {
                //debug!("removing mask at {index}");
                let next_mask_counter = self.mask_counter[index] - 1;
                self.mask_counter[index] = next_mask_counter;
                if next_mask_counter == 0 {
                    self.is_restricted_at_index.set(index, false);
                    self.is_mask_dirty = true;
                    self.entropy = None;
                }
            }
        }
        //debug!("removed mask {:?} at current state {:?}.", mask, self.mask_counter);
    }
    pub fn forward_mask(&mut self, mask: &BitVec) {
        self.previous_mask_counters.push_back(self.mask_counter.clone());
        self.previous_is_restricted_at_index.push_back(self.is_restricted_at_index.clone());
        self.add_mask(mask);
    }
    pub fn reverse_mask(&mut self) {
        //debug!("removing mask {:?} at current state {:?}.", mask, self.mask_counter);
        self.mask_counter = self.previous_mask_counters.pop_back().unwrap();
        self.is_restricted_at_index = self.previous_is_restricted_at_index.pop_back().unwrap();
        self.is_fully_restricted = false;  // any movement backwards is to a non-restricted state
        self.entropy = None;
        //debug!("removed mask {:?} at current state {:?}.", mask, self.mask_counter);
    }
    /// This function will return if the provided mask would change the restrictions of this indexed view
    pub fn is_mask_restrictive(&self, mask: &BitVec) -> bool {
        let mut is_at_least_one_bit_updated = false;
        for index in 0..self.node_state_ids_length {
            if !mask[index] && !self.is_restricted_at_index[index] {
                is_at_least_one_bit_updated = true;
                break;
            }
        }
        is_at_least_one_bit_updated
    }
    pub fn stash_mask_state(&mut self) -> IndexedViewMaskState {
        let indexed_view_mask_state = IndexedViewMaskState {
            mask_counter: self.mask_counter.clone(),
            is_restricted_at_index: self.is_restricted_at_index.clone()
        };
        for index in 0..self.node_state_ids_length {
            self.mask_counter[index] = 0;
            self.is_restricted_at_index.set(index, false);
        }
        self.is_mask_dirty = true;
        indexed_view_mask_state
    }
    pub fn unstash_mask_state(&mut self, mask_state: &mut IndexedViewMaskState) {
        for index in 0..self.node_state_ids_length {
            self.mask_counter[index] += mask_state.mask_counter[index];
            let is_restricted_at_index: bool = self.is_restricted_at_index[index];
            self.is_restricted_at_index.set(index, is_restricted_at_index || mask_state.is_restricted_at_index[index]);

            mask_state.mask_counter[index] = 0;
            mask_state.is_restricted_at_index.set(index, false);
        }
        self.is_mask_dirty = true;
    }
    pub fn is_fully_unmasked(&self) -> bool {
        let mut is_masked = false;
        for index in 0..self.node_state_ids_length {
            if self.mask_counter[index] != 0 {
                is_masked = true;
                break;
            }
        }
        !is_masked
    }
    pub fn get_mask_density(&self) -> u32 {
        let mut mask_density = 0;
        for index in 0..self.node_state_ids_length {
            mask_density += self.mask_counter[index];
        }
        mask_density
    }
    pub fn entropy(&mut self) -> f32 {
        if self.entropy.is_none() {
            let mut weights_total: f32 = 0.0;
            let mut weights_times_log_weights_total: f32 = 0.0;
            for index in 0..self.node_state_ids_length {
                if !self.is_restricted_at_index[index] {
                    let weight = self.node_state_ratios[index];
                    let log_weight = weight.ln();
                    weights_total += weight;
                    weights_times_log_weights_total += weight * log_weight;
                }
            }
            self.entropy = Some(weights_total.ln() - weights_times_log_weights_total / weights_total);
        }
        self.entropy.unwrap()
    }
    pub fn get_possible_states(&self) -> Vec<TNodeState> {
        let mut possible_states: Vec<TNodeState> = Vec::new();
        if let Some(index) = self.index {
            let mapped_index = self.index_mapping[index];
            let node_state = self.node_state_ids.get(mapped_index).unwrap();
            possible_states.push(node_state.clone());
        }
        else {
            for index in 0..self.node_state_ids_length {
                if !self.is_restricted_at_index[index] {
                    let node_state = self.node_state_ids.get(index).unwrap();
                    possible_states.push(node_state.clone());
                }
            }
        }
        possible_states
    }
}

impl<TNodeState: Eq + Hash + Clone + std::fmt::Debug> Debug for IndexedView<TNodeState> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexedView with mask counter {:?}.", self.mask_counter)
    }
}