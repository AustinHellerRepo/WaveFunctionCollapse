use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;


pub struct MappedView<TViewKey: Eq + Hash, TKey: Eq + Hash + Copy, TValue> {
    value_per_key_per_view_key: HashMap<TViewKey, HashMap<TKey, TValue>>,
    view_key: Option<TViewKey>
}

impl<TViewKey: Eq + Hash, TKey: Eq + Hash + Copy, TValue> MappedView<TViewKey, TKey, TValue> {
    pub fn new() -> Self {
        MappedView {
            value_per_key_per_view_key: HashMap::new(),
            view_key: Option::None
        }
    }
    pub fn insert(&mut self, view_key: TViewKey, map: HashMap<TKey, TValue>) {
        self.value_per_key_per_view_key.insert(view_key, map);
    }
    pub fn insert_partial(&mut self, map: HashMap<TKey, HashMap<TViewKey, TValue>>) {
        for (key, value_per_view_key) in map.into_iter() {
            self.insert_individual(key, value_per_view_key);
        }
    }
    pub fn insert_individual(&mut self, key: TKey, map: HashMap<TViewKey, TValue>) {
        for (view_key, value) in map.into_iter() {
            if self.value_per_key_per_view_key.contains_key(&view_key) {
                self.value_per_key_per_view_key.get_mut(&view_key).unwrap().insert(key, value);
            }
            else {
                let mut hashmap: HashMap<TKey, TValue> = HashMap::new();
                hashmap.insert(key, value);
                self.value_per_key_per_view_key.insert(view_key, hashmap);
            }
        }
    }
    // in my use case it will be the neighbor's node id as the key and the Option<&BitVec> as the value
    pub fn get(&self, key: &TKey) -> Option<&TValue> {
        if self.view_key.is_some() {
            let view_key_ref: &TViewKey = self.view_key.as_ref().unwrap();
            Some(self.value_per_key_per_view_key.get(view_key_ref).unwrap().get(key).unwrap())
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