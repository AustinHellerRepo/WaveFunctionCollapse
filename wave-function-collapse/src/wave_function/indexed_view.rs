use std::collections::VecDeque;
use std::f32::consts::E;
use std::fmt::{Debug};
use std::hash::Hash;
use std::{collections::HashMap};
use bitvec::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::wave_function::probability_container::ProbabilityContainer;

pub struct IndexedView<TNodeState: Clone + Eq + Hash + Debug> {
    // items are states of the node
    node_state_ids: Vec<TNodeState>,
    node_state_probabilities: Vec<f32>,
    index_per_node_state_id: HashMap<TNodeState, usize>,
    node_state_ids_length: usize,
    index: Option<usize>,
    index_mapping: Vec<usize>,
    mask_counter: Vec<u32>,
    is_restricted_at_index: BitVec,
    is_mask_dirty: bool,
    is_fully_restricted: bool,
    previous_mask_counters: VecDeque<Vec<u32>>,
    previous_is_restricted_at_index: VecDeque<BitVec>
}

impl<TNodeState: Clone + Eq + Hash + Debug> IndexedView<TNodeState> {
    #[time_graph::instrument]
    pub fn new(node_state_ids: Vec<TNodeState>, node_state_probabilities: Vec<f32>) -> Self {
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
            node_state_ids: node_state_ids,
            node_state_probabilities: node_state_probabilities,
            index_per_node_state_id: index_per_node_state_id,
            node_state_ids_length: node_state_ids_length,
            index: Option::None,
            index_mapping: index_mapping,
            mask_counter: mask_counter,
            is_restricted_at_index: is_restricted_at_index,
            is_mask_dirty: true,
            is_fully_restricted: false,
            previous_mask_counters: VecDeque::new(),
            previous_is_restricted_at_index: VecDeque::new()
        }
    }
    #[time_graph::instrument]
    pub fn shuffle<R: Rng + ?Sized>(&mut self, random_instance: &mut R) {
        if self.index.is_some() {
            panic!("Can only be shuffled prior to use.");
        }

        self.index_mapping.clear();
        let mut probability_container = ProbabilityContainer::default();
        for (node_state_id, probability) in std::iter::zip(self.node_state_ids.iter(), self.node_state_probabilities.iter()) {
            probability_container.push(node_state_id, *probability);
        }

        for _ in 0..self.node_state_ids_length {
            let node_state_id = probability_container.pop_random(random_instance).unwrap();
            self.index_mapping.push(*self.index_per_node_state_id.get(&node_state_id).unwrap());
        }

        debug!("randomized index mapping to {:?}.", self.index_mapping);
    }
    #[time_graph::instrument]
    pub fn try_move_next(&mut self) -> bool {
        let mut is_unmasked = false;
        let mut next_index: usize;

        let node_state_ids_length = &self.node_state_ids_length;
        if let Some(index) = self.index {
            debug!("trying to get next state starting with {index} and ending prior to {node_state_ids_length}.");
        }
        else {
            debug!("trying to get next state starting with None and ending prior to {node_state_ids_length}.");
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
    #[time_graph::instrument]
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
    #[time_graph::instrument]
    fn is_unmasked_at_index(&self, index: usize) -> bool {
        //debug!("checking if unmasked at index {index} for node {mask_key}.");
        let mapped_index = self.index_mapping[index];
        //self.mask_counter[*mapped_index] == 0
        !self.is_restricted_at_index[mapped_index]
    }
    #[time_graph::instrument]
    pub fn is_mask_restrictive_to_current_state(&self, mask: &BitVec) -> bool {
        if let Some(index) = self.index {
            let mapped_index = self.index_mapping[index];
            let is_restrictive = !mask[mapped_index];
            if is_restrictive {
                debug!("mask is restrictive at index {:?} after mapping to index {:?} for mask {:?}", index, mapped_index, mask);
            }
            else {
                debug!("mask is not restrictive at index {:?} after mapping to index {:?} for mask {:?}", index, mapped_index, mask);
            }
            is_restrictive
        }
        else {
            debug!("was not restrictive because not currently in a state");
            false
        }
    }
    #[time_graph::instrument]
    pub fn get(&self) -> Option<&TNodeState> {
        let value: Option<&TNodeState>;
        if let Some(index) = self.index {
            if index == self.node_state_ids_length {
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
    #[time_graph::instrument]
    pub fn reset(&mut self) {
        self.index = Option::None;
        // NOTE: the mask_counter should not be fully reverted to ensure that the neighbor restrictions are still being considered
    }
    #[time_graph::instrument]
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
    #[time_graph::instrument]
    pub fn is_fully_restricted(&mut self) -> bool {
        if self.is_mask_dirty {
            self.is_fully_restricted = self.is_restricted_at_index.count_ones() == self.node_state_ids_length;
            self.is_mask_dirty = false;
        }
        self.is_fully_restricted
    }
    #[time_graph::instrument]
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
                }
                //self.mask_counter[index] = self.mask_counter[index].checked_add(1).unwrap();  // TODO replace with unchecked version above
            }
        }
        //debug!("added mask {:?} at current state {:?}.", mask, self.mask_counter);
    }
    #[time_graph::instrument]
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
                }
                //self.mask_counter[index] = self.mask_counter[index].checked_sub(1).unwrap();  // TODO replace with unchecked version above
            }
        }
        //debug!("removed mask {:?} at current state {:?}.", mask, self.mask_counter);
    }
    #[time_graph::instrument]
    pub fn forward_mask(&mut self, mask: &BitVec) {
        self.previous_mask_counters.push_back(self.mask_counter.clone());
        self.previous_is_restricted_at_index.push_back(self.is_restricted_at_index.clone());
        self.add_mask(mask);
    }
    #[time_graph::instrument]
    pub fn reverse_mask(&mut self) {
        //debug!("removing mask {:?} at current state {:?}.", mask, self.mask_counter);
        self.mask_counter = self.previous_mask_counters.pop_back().unwrap();
        self.is_restricted_at_index = self.previous_is_restricted_at_index.pop_back().unwrap();
        self.is_fully_restricted = false;  // any movement backwards is to a non-restricted state
        //debug!("removed mask {:?} at current state {:?}.", mask, self.mask_counter);
    }
}

impl<TNodeState: Eq + Hash + Clone + std::fmt::Debug> Debug for IndexedView<TNodeState> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexedView with mask counter {:?}.", self.mask_counter)
    }
}