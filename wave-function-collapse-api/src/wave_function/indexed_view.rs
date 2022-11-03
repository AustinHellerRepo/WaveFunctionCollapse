use std::cell::RefCell;
use std::rc::Rc;
use std::{cell::Cell, collections::HashMap};
use bitvec::prelude::*;
use rand::{seq::SliceRandom, Rng};
use super::mapped_view::MappedView;
use std::cmp::Eq;
use std::hash::Hash;

pub struct IndexedView<TNodeState, TViewKey: Eq + Hash, TKey: Eq + Hash + Copy> {
    // items are states of the node
    node_state_ids: Vec<TNodeState>,
    node_state_ids_length: usize,
    index: Option<usize>,
    masks: Vec<Rc<RefCell<MappedView<TViewKey, TKey, BitVec>>>>,
    masks_key: TKey,
    index_mapping: HashMap<usize, usize>
}

impl<TNodeState, TViewKey: Eq + Hash, TKey: Eq + Hash + Copy> IndexedView<TNodeState, TViewKey, TKey> {
    pub fn new(node_state_ids: Vec<TNodeState>, masks: Vec<Rc<RefCell<MappedView<TViewKey, TKey, BitVec>>>>, masks_key: TKey) -> Self {
        let node_state_ids_length: usize = node_state_ids.len();
        let mut index_mapping = HashMap::new();
        for index in 0..node_state_ids_length {
            index_mapping.insert(index, index);
        }
        IndexedView {
            node_state_ids: node_state_ids,
            node_state_ids_length: node_state_ids_length,
            index: Option::None,
            masks: masks,
            masks_key: masks_key,
            index_mapping: index_mapping
        }
    }
    pub fn shuffle<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        if self.index.is_some() {
            panic!("Can only be shuffled prior to use.");
        }
        let mut shuffled_values: Vec<usize> = (0..self.node_state_ids_length).collect();
        shuffled_values.shuffle(rng);
        for index in 0..self.node_state_ids_length {
            self.index_mapping.insert(index, shuffled_values[index]);
        }
    }
    pub fn try_move_next(&mut self) -> bool {
        let mut is_unmasked = false;
        let mut next_index: usize;
        while self.index.is_none() || (self.index.unwrap() < self.node_state_ids_length && !is_unmasked) {
            if let Some(index) = self.index {
                next_index = index + 1;
            }
            else {
                next_index = 0;
            }
            is_unmasked = self.is_unmasked_at_index(next_index);
            self.index = Some(next_index);
        }
        self.index.unwrap() != self.node_state_ids_length
    }
    pub fn try_move_previous(&mut self) -> bool {
        let mut is_unmasked = false;
        let mut current_index: usize;
        let mut next_index: usize;
        while !self.index.is_none() && !is_unmasked {
            current_index = self.index.unwrap();
            if current_index == 0 {
                self.index = Option::None;
            }
            else {
                next_index = current_index - 1;
                is_unmasked = self.is_unmasked_at_index(next_index);
                self.index = Some(next_index);
            }
        }
        self.index.is_some()
    }
    fn is_unmasked_at_index(&self, index: usize) -> bool {
        for mask_mapped_view in self.masks.iter() {
            if let Some(mask) = mask_mapped_view.as_ref().borrow().get(&self.masks_key) {
                if !mask[index] {
                    return false;
                }
            }
        }
        return true;
    }
    pub fn get(&self) -> Option<&TNodeState> {
        let value: Option<&TNodeState>;
        if let Some(index) = self.index {
            value = self.node_state_ids.get(index);
        }
        else {
            value = None;
        }
        value
    }
    pub fn is_in_some_state(&self) -> bool {
        self.index.is_some()
    }
    pub fn reset(&mut self) {
        self.index = Option::None;
    }
    pub fn is_fully_restricted(&self) -> bool {
        let mut is_at_least_one_node_state_possible: bool = false;
        for index in 0..self.node_state_ids_length {
            if self.is_unmasked_at_index(index) {
                is_at_least_one_node_state_possible = true;
                break;
            }
        }
        !is_at_least_one_node_state_possible
    }
    pub fn get_restriction_ratio(&self) -> f32 {
        let mut masked_bits_total: u32 = 0;
        for index in 0..self.node_state_ids_length {
            if !self.is_unmasked_at_index(index) {
                masked_bits_total += 1;
            }
        }
        (masked_bits_total as f32) / (self.node_state_ids_length as f32)
    }
}