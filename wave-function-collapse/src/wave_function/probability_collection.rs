use std::{collections::{BTreeMap, HashMap}, fmt::Debug};
use log::kv::ToValue;
use ordered_float::OrderedFloat;
use rand::Rng;
use std::hash::Hash;

pub struct ProbabilityCollection<T> {
    probability_total: f32,
    item_per_cumulative_probability: BTreeMap<OrderedFloat<f32>, T>,
    items_total: u32,
    probability_per_item: HashMap<T, f32>
}

impl<T: Eq + Hash + Clone + Debug> ProbabilityCollection<T> {
    pub fn new(probability_per_item: HashMap<T, f32>) -> Self {
        let mut probability_total = 0.0;
        let mut item_per_cumulative_probability: BTreeMap<OrderedFloat<f32>, T> = BTreeMap::new();
        let mut items_total: u32 = 0;
        for (item, probability) in probability_per_item.iter() {
            if probability != &0.0 {
                probability_total += probability;
                item_per_cumulative_probability.insert(OrderedFloat(probability_total), item.clone());
                items_total += 1;
            }
        }
        ProbabilityCollection {
            probability_total: probability_total,
            item_per_cumulative_probability: item_per_cumulative_probability,
            items_total: items_total,
            probability_per_item: probability_per_item
        }
    }
    pub fn pop_item<R: Rng + ?Sized>(&mut self, random_instance: &mut R) -> Option<T> {
        debug!("current state: {:?}", self.item_per_cumulative_probability);
        if self.items_total == 0 {
            debug!("no items");
            None
        }
        else {
            let item_option: Option<T>;
            let mut key: Option<OrderedFloat<f32>> = None;
            if self.items_total == 1 {
                //self.item_per_cumulative_probability.remove(&OrderedFloat(self.probability_total))
                key = Some(*self.item_per_cumulative_probability.keys().next().unwrap());
                debug!("one item: {:?}", key);
                item_option = self.item_per_cumulative_probability.remove(&key.unwrap());
                self.items_total = 0;
                self.probability_total = 0.0;
            }
            else {
                let random_value = OrderedFloat(random_instance.gen::<f32>() * self.probability_total);
                debug!("random_value: {:?}", random_value);
                let mut restructured_item_per_cumulative_probability: HashMap<OrderedFloat<f32>, T> = HashMap::new();
                let mut remove_keys: Vec<OrderedFloat<f32>> = Vec::new();
                let mut found_item_probability: Option<f32> = None;
                for (index, (temp_key, temp_value)) in self.item_per_cumulative_probability.range(random_value..).enumerate() {
                    let item_probability = self.probability_per_item.get(temp_value).unwrap();
                    if index == 0 {
                        key = Some(*temp_key);
                        found_item_probability = Some(*item_probability);
                    }
                    else {
                        let recalculated_cumulative_probability = OrderedFloat(temp_key.0 - item_probability);
                        restructured_item_per_cumulative_probability.insert(recalculated_cumulative_probability, temp_value.clone());
                        remove_keys.push(*temp_key);
                    }
                }
                debug!("more than one item: {:?}", key);
                // remove keys first
                for remove_key in remove_keys.iter() {
                    self.item_per_cumulative_probability.remove(remove_key);
                }

                // resupply newly calculated keys
                item_option = self.item_per_cumulative_probability.remove(&key.unwrap());
                for (index, (temp_key, temp_value)) in restructured_item_per_cumulative_probability.into_iter().enumerate() {
                    self.item_per_cumulative_probability.insert(temp_key, temp_value);
                }

                // refresh cache data
                self.items_total -= 1;
                self.probability_total -= found_item_probability.unwrap();
            }
            item_option
        }
    }
}