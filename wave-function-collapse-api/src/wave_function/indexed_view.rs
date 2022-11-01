use std::{cell::Cell, collections::HashMap};
use bitvec::prelude::*;
use rand::{seq::SliceRandom, Rng};
use super::mapped_view::MappedView;
use std::cmp::Eq;
use std::hash::Hash;

pub struct IndexedView<'a, TItem, TViewKey: Eq + Hash, TKey: Eq + Hash> {
    // items are states of the node
    items: Vec<TItem>,
    items_length: usize,
    index: Option<usize>,
    masks: Vec<&'a MappedView<TViewKey, TKey, BitVec>>,
    masks_key: TKey,
    index_mapping: HashMap<usize, usize>
}

impl<'a, TItem, TViewKey: Eq + Hash, TKey: Eq + Hash> IndexedView<'a, TItem, TViewKey, TKey> {
    pub fn new(items: Vec<TItem>, masks: Vec<&'a MappedView<TViewKey, TKey, BitVec>>, masks_key: TKey) -> Self {
        let items_length: usize = items.len();
        let mut index_mapping = HashMap::new();
        for index in 0..items_length {
            index_mapping.insert(index, index);
        }
        IndexedView {
            items: items,
            items_length: items_length,
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
        let mut shuffled_values: Vec<usize> = (0..self.items_length).collect();
        shuffled_values.shuffle(rng);
        for index in 0..self.items_length {
            self.index_mapping.insert(index, shuffled_values[index]);
        }
    }
    pub fn try_move_next(&mut self) -> bool {
        let mut is_unmasked = false;
        let mut next_index: usize;
        while self.index.is_none() || (self.index.unwrap() < self.items_length && !is_unmasked) {
            if let Some(index) = self.index {
                next_index = index + 1;
            }
            else {
                next_index = 0;
            }
            is_unmasked = self.is_unmasked_at_index(next_index);
            self.index = Some(next_index);
        }
        self.index.unwrap() != self.items_length
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
        for mask_option in self.masks.iter() {
            if let Some(mask) = mask_option.get(&self.masks_key) {
                if !mask[index] {
                    return false;
                }
            }
        }
        return true;
    }
    pub fn get(&self) -> &TItem {
        self.items.get(self.index.unwrap()).unwrap()
    }
    pub fn is_in_some_state(&self) -> bool {
        self.index.is_some()
    }
    pub fn reset(&mut self) {
        self.index = Option::None;
    }
}