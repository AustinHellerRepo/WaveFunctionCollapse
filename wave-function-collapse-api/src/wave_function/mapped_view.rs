use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;


pub struct MappedView<TViewKey: Eq + Hash, TKey: Eq + Hash, TValue> {
    value_per_key_per_view_key: HashMap<TViewKey, HashMap<TKey, TValue>>,
    view_key: Option<TViewKey>
}

impl<TViewKey: Eq + Hash, TKey: Eq + Hash, TValue> MappedView<TViewKey, TKey, TValue> {
    pub fn new() -> Self {
        MappedView {
            value_per_key_per_view_key: HashMap::new(),
            view_key: Option::None
        }
    }
    pub fn insert(&mut self, view_key: TViewKey, map: HashMap<TKey, TValue>) {
        self.value_per_key_per_view_key.insert(view_key, map);
    }
    // in my use case it will be the neighbor's node id as the key and the Option<&BitVec> as the value
    pub fn get(&self, key: &TKey) -> Option<&TValue> {
        if let Some(view_key) = self.view_key {
            Some(self.value_per_key_per_view_key.get(&view_key).unwrap().get(key).unwrap())
        }
        else {
            None
        }
    }
    pub fn orient(&mut self, view_key: TViewKey) {
        self.view_key = Some(view_key);
    }
    pub fn reset(&mut self) {
        self.view_key = None;
    }
}