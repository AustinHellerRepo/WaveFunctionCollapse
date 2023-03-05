use std::{collections::{BTreeMap, HashMap}, fmt::Debug};
use ordered_float::OrderedFloat;
use rand::Rng;
use std::hash::Hash;

/// This struct is optimized better than ProbabilityContainer to search for a random item but does not permit removing a random item.
#[allow(dead_code)]
pub struct ProbabilityTree<T> {
    probability_total: f32,
    item_per_cumulative_probability: BTreeMap<OrderedFloat<f32>, T>,
    items_total: u32,
    probability_per_item: HashMap<T, f32>
}

impl<T: Eq + Hash + Clone + Debug> ProbabilityTree<T> {
    #[allow(dead_code)]
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
        ProbabilityTree {
            probability_total,
            item_per_cumulative_probability,
            items_total,
            probability_per_item
        }
    }
    #[allow(dead_code)]
    pub fn peek_random<R: Rng + ?Sized>(&self, random_instance: &mut R) -> Option<T> {
        let item_option: Option<T>;
        if self.items_total == 0 {
            debug!("no items");
            item_option = None;
        }
        else if self.items_total == 1 {
            let key = *self.item_per_cumulative_probability.keys().next().unwrap();
            debug!("one item: {:?}", key);
            item_option = Some(self.item_per_cumulative_probability.get(&key).unwrap().clone());
        }
        else {
            let random_value = OrderedFloat(random_instance.gen::<f32>() * self.probability_total);
            debug!("random_value: {:?}", random_value);
            let (temp_key, temp_value) = self.item_per_cumulative_probability.range(random_value..).next().unwrap();
            debug!("found item {:?} with probability {:?}", temp_value, temp_key);
            item_option = Some(temp_value.clone());
        }
        item_option
    }
}