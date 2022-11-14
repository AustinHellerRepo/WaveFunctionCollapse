use std::f32::consts::E;
use std::fmt::{Debug};
use std::{collections::HashMap};
use bitvec::prelude::*;
use rand::{seq::SliceRandom, Rng};

pub struct IndexedView<TNodeState> {
    // items are states of the node
    node_state_ids: Vec<TNodeState>,
    node_state_ids_length: usize,
    index: Option<usize>,
    index_mapping: HashMap<usize, usize>,
    mask_counter: Vec<u32>,
    current_restriction_total: u32,
    is_mask_dirty: bool
}

impl<TNodeState> IndexedView<TNodeState> {
    #[time_graph::instrument]
    pub fn new(node_state_ids: Vec<TNodeState>) -> Self {
        let node_state_ids_length: usize = node_state_ids.len();
        let mut index_mapping = HashMap::new();
        let mut mask_counter: Vec<u32> = Vec::new();
        for index in 0..node_state_ids_length {
            index_mapping.insert(index, index);
            mask_counter.push(0);
        }
        IndexedView {
            node_state_ids: node_state_ids,
            node_state_ids_length: node_state_ids_length,
            index: Option::None,
            index_mapping: index_mapping,
            mask_counter: mask_counter,
            current_restriction_total: 0,
            is_mask_dirty: false
        }
    }
    #[time_graph::instrument]
    pub fn shuffle<R: Rng + ?Sized>(&mut self, random_instance: &mut R) {
        if self.index.is_some() {
            panic!("Can only be shuffled prior to use.");
        }
        let mut shuffled_indexes: Vec<usize> = (0..self.node_state_ids_length).collect();
        shuffled_indexes.shuffle(random_instance);
        self.index_mapping.clear();
        for index in 0..self.node_state_ids_length {
            self.index_mapping.insert(index, shuffled_indexes[index]);
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
    fn is_unmasked_at_index(&self, index: usize) -> bool {
        //debug!("checking if unmasked at index {index} for node {mask_key}.");
        let mapped_index = self.index_mapping.get(&index).unwrap();
        self.mask_counter[*mapped_index] == 0
    }
    #[time_graph::instrument]
    pub fn get(&self) -> Option<&TNodeState> {
        let value: Option<&TNodeState>;
        if let Some(index) = self.index {
            if index == self.node_state_ids_length {
                value = None;
            }
            else {
                let mapped_index = self.index_mapping.get(&index).unwrap();
                value = self.node_state_ids.get(*mapped_index);
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
            self.current_restriction_total = 0;
            for index in 0..self.node_state_ids_length {
                if self.mask_counter[index] != 0 {
                    self.current_restriction_total += 1;
                }
            }
            self.is_mask_dirty = false;
        }
        self.current_restriction_total == (self.node_state_ids_length as u32)
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
                    self.is_mask_dirty = true;
                }
                //self.mask_counter[index] = self.mask_counter[index].checked_add(1).unwrap();  // TODO replace with unchecked version above
            }
            else {
                //debug!("not adding mask at {index}");
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
                    self.is_mask_dirty = true;
                }
                //self.mask_counter[index] = self.mask_counter[index].checked_sub(1).unwrap();  // TODO replace with unchecked version above
            }
            else {
                //debug!("not removing mask at {index}");
            }
        }
        //debug!("removed mask {:?} at current state {:?}.", mask, self.mask_counter);
    }
}

impl<TNodeState> Debug for IndexedView<TNodeState> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexedView with mask counter {:?}.", self.mask_counter)
    }
}