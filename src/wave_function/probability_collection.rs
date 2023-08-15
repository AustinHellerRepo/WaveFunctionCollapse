use std::{fmt::Debug, collections::HashMap};
use rand::Rng;
use std::hash::Hash;

/// This struct is optimized better than ProbabilityContainer to remove a random item but does not permit searching for a random item.
#[allow(dead_code)]
pub struct ProbabilityCollection<T> {
    probability_total: f32,
    items_total: u32,
    probability_per_item: HashMap<T, f32>,
    items: Vec<T>
}

#[allow(dead_code)]
impl<T: Eq + Hash + Clone + Debug> ProbabilityCollection<T> {
    pub fn new(probability_per_item: HashMap<T, f32>) -> Self {
        let mut probability_total = 0.0;
        let mut items_total: u32 = 0;
        let mut items: Vec<T> = Vec::new();
        for (item, probability) in probability_per_item.iter() {
            if probability != &0.0 {
                probability_total += probability;
                items.push(item.clone());
                items_total += 1;
            }
        }
        ProbabilityCollection {
            probability_total: probability_total,
            items_total: items_total,
            probability_per_item: probability_per_item,
            items: items
        }
    }
    pub fn pop_random<R: Rng + ?Sized>(&mut self, random_instance: &mut R) -> Option<T> {
        debug!("current state: {:?}", self.probability_per_item);
        if self.items_total == 0 {
            debug!("no items");
            None
        }
        else {
            let mut item_option: Option<T>;
            if self.items_total == 1 {
                //self.item_per_cumulative_probability.remove(&OrderedFloat(self.probability_total))
                item_option = self.items.first().cloned();
                debug!("one item: {:?}", item_option);
                self.items.clear();
                self.items_total = 0;
                self.probability_total = 0.0;
            }
            else {
                let random_value = random_instance.gen::<f32>() * self.probability_total;
                debug!("random_value: {:?}", random_value);
                let mut current_probability = 0.0;
                let mut found_item_index: Option<usize> = None;
                item_option = None;
                for (item_index, item) in self.items.iter().enumerate() {
                    let item_probability = self.probability_per_item.get(item).unwrap();
                    current_probability += item_probability;
                    if current_probability >= random_value {
                        self.probability_total -= item_probability;
                        found_item_index = Some(item_index);
                        item_option = Some(item.clone());
                        break;
                    }
                }
                if item_option.is_none() {
                    panic!("Failed to find item even though some exists.");
                }
                debug!("more than one item: {:?}", item_option);

                // refresh cache data
                self.items.remove(found_item_index.unwrap());
                self.items_total -= 1;
            }
            item_option
        }
    }
}