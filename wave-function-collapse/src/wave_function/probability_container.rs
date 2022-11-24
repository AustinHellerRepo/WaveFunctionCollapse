use std::{collections::{BTreeMap, HashMap}, fmt::Debug};
use log::kv::ToValue;
use ordered_float::OrderedFloat;
use rand::Rng;
use std::hash::Hash;

pub struct ProbabilityContainer<T> {
    probability_total: f32,
    items_total: u32,
    probability_per_item: HashMap<T, f32>,
    items: Vec<T>,
    item_index_per_cumulative_probability: BTreeMap<OrderedFloat<f32>, usize>,
    last_item_index_to_apply_to_item_index_per_cumulative_probability: usize,
    last_cumulative_probability: f32
}

impl<T: Eq + Hash + Clone + Debug> ProbabilityContainer<T> {
    pub fn default() -> Self {
        ProbabilityContainer {
            probability_total: 0.0,
            items_total: 0,
            probability_per_item: HashMap::new(),
            items: Vec::new(),
            item_index_per_cumulative_probability: BTreeMap::new(),
            last_item_index_to_apply_to_item_index_per_cumulative_probability: 0,
            last_cumulative_probability: 0.0
        }
    }
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
        ProbabilityContainer {
            probability_total: probability_total,
            items_total: items_total,
            probability_per_item: probability_per_item,
            items: items,
            item_index_per_cumulative_probability: BTreeMap::new(),
            last_item_index_to_apply_to_item_index_per_cumulative_probability: 0,
            last_cumulative_probability: 0.0
        }
    }
    pub fn push(&mut self, item: T, probability: f32) {
        self.probability_total += probability;
        self.items_total += 1;
        self.probability_per_item.insert(item.clone(), probability);
        self.items.push(item);
    }
    pub fn get_random<R: Rng + ?Sized>(&mut self, random_instance: &mut R) -> Option<T> {
        let item_option: Option<T>;
        if self.items_total == 0 {
            debug!("no items");
            item_option = None;
        }
        else {
            if self.items_total == 1 {
                item_option = Some(self.items.iter().next().unwrap().clone());
                debug!("one item: {:?}", item_option);
            }
            else {
                let random_value = random_instance.gen::<f32>() * self.probability_total;
                if random_value > self.last_cumulative_probability {
                    let mut current_item: Option<&T> = None;
                    while random_value > self.last_cumulative_probability {
                        current_item = Some(self.items.get(self.last_item_index_to_apply_to_item_index_per_cumulative_probability).unwrap());
                        let item_probability = self.probability_per_item.get(current_item.unwrap()).unwrap();
                        if item_probability != &0.0 {
                            self.last_cumulative_probability += item_probability;
                            debug!("inserting {:?} with cumulative probability {:?}", self.last_item_index_to_apply_to_item_index_per_cumulative_probability, self.last_cumulative_probability);
                            self.item_index_per_cumulative_probability.insert(OrderedFloat(self.last_cumulative_probability), self.last_item_index_to_apply_to_item_index_per_cumulative_probability);
                        }
                        self.last_item_index_to_apply_to_item_index_per_cumulative_probability += 1;
                    }
                    let current_item = current_item.unwrap().clone();
                    debug!("found item {:?}", current_item);
                    item_option = Some(current_item.clone());
                }
                else {
                    debug!("random_value: {:?}", random_value);
                    let (temp_key, temp_value) = self.item_index_per_cumulative_probability.range(OrderedFloat(random_value)..).next().unwrap();
                    debug!("found item {:?} with probability {:?}", temp_value, temp_key);
                    item_option = Some(self.items.get(*temp_value).unwrap().clone());
                }
            }
        }
        item_option
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
                self.item_index_per_cumulative_probability.clear();
                self.last_item_index_to_apply_to_item_index_per_cumulative_probability = 0;
                self.last_cumulative_probability = 0.0;
                self.probability_per_item.clear();
            }
            else {
                let random_value = random_instance.gen::<f32>() * self.probability_total;
                debug!("random_value: {:?}", random_value);
                debug!("self.probability_total: {:?}", self.probability_total);
                debug!("self.last_cumulative_probability: {:?}", self.last_cumulative_probability);
                debug!("self.last_item_index_to_apply_to_item_index_per_cumulative_probability: {:?}", self.last_item_index_to_apply_to_item_index_per_cumulative_probability);
                
                let mut is_item_outside_random_value: bool;
                if self.last_item_index_to_apply_to_item_index_per_cumulative_probability as u32 == self.items_total {
                    is_item_outside_random_value = false;
                }
                else if random_value == 0.0 && self.last_item_index_to_apply_to_item_index_per_cumulative_probability == 0 {
                    is_item_outside_random_value = true;
                }
                else if random_value > self.last_cumulative_probability {
                    is_item_outside_random_value = true;
                }
                else {
                    is_item_outside_random_value = false;
                }

                if is_item_outside_random_value {
                    let mut current_item: &T;
                    // if the random value is out of range of the known probabilities
                    while is_item_outside_random_value {
                        current_item = self.items.get(self.last_item_index_to_apply_to_item_index_per_cumulative_probability).unwrap();
                        let item_probability = self.probability_per_item.get(current_item).unwrap();
                        if item_probability != &0.0 {
                            if self.last_cumulative_probability + item_probability >= random_value {
                                debug!("found next item with probability {:?}", item_probability);

                                // that there hasn't been floating point errors leading to missing the last item
                                if (self.last_item_index_to_apply_to_item_index_per_cumulative_probability as u32) + 1 == self.items_total {
                                    self.probability_total = self.last_cumulative_probability + item_probability;
                                    debug!("fixed probability total after incrementing to item");
                                }
                                
                                break;
                            }
                            else {
                                self.last_cumulative_probability += item_probability;
                                debug!("inserting {:?} with cumulative probability {:?} into index {:?}", current_item, self.last_cumulative_probability, self.last_item_index_to_apply_to_item_index_per_cumulative_probability);
                                self.item_index_per_cumulative_probability.insert(OrderedFloat(self.last_cumulative_probability), self.last_item_index_to_apply_to_item_index_per_cumulative_probability);
                            }
                        }
                        self.last_item_index_to_apply_to_item_index_per_cumulative_probability += 1;
                        debug!("self.last_item_index_to_apply_to_item_index_per_cumulative_probability: {:?}", self.last_item_index_to_apply_to_item_index_per_cumulative_probability);

                        
                        // that there hasn't been floating point errors leading to missing the last item
                        if (self.last_item_index_to_apply_to_item_index_per_cumulative_probability as u32) == self.items_total {
                            self.probability_total = self.last_cumulative_probability;
                            debug!("fixed probability total after missing item");

                            // move back one item so that the process ends up grabbing the last item
                            self.last_item_index_to_apply_to_item_index_per_cumulative_probability -= 1;
                            break;
                        }

                        is_item_outside_random_value = random_value > self.last_cumulative_probability;
                    }

                    let item = self.items.remove(self.last_item_index_to_apply_to_item_index_per_cumulative_probability);
                    self.probability_total -= self.probability_per_item.remove(&item).unwrap();
                    item_option = Some(item);
                    self.items_total -= 1;

                    debug!("found item {:?}", item_option);
                }
                else {
                    let found_key: f32;
                    let found_index: usize;
                    let found_item: T;
                    {
                        let (temp_key, temp_value) = self.item_index_per_cumulative_probability.range(OrderedFloat(random_value)..).next().unwrap();
                        debug!("found item {:?} with probability {:?}", temp_value, temp_key);
                        found_item = self.items.remove(*temp_value);
                        self.items_total -= 1;
                        item_option = Some(found_item.clone());
                        
                        found_key = temp_key.0;
                        found_index = *temp_value;
                    }

                    let found_key_ordered_float = &OrderedFloat(found_key);
                    self.item_index_per_cumulative_probability.retain(|probability, _| probability < found_key_ordered_float);
                    self.last_item_index_to_apply_to_item_index_per_cumulative_probability = found_index;
                    debug!("self.last_item_index_to_apply_to_item_index_per_cumulative_probability: {:?}", self.last_item_index_to_apply_to_item_index_per_cumulative_probability);
                    let found_item_probability = self.probability_per_item.remove(&found_item).unwrap();
                    self.last_cumulative_probability = found_key - found_item_probability;

                    // that there hasn't been floating point errors leading to missing the last item
                    if (self.last_item_index_to_apply_to_item_index_per_cumulative_probability as u32) == self.items_total {
                        self.probability_total = self.last_cumulative_probability;
                        debug!("fixed probability total after finding item");
                    }
                    else {
                        self.probability_total -= found_item_probability;
                    }
                }

                if item_option.is_none() {
                    panic!("Failed to find item even though some exists.");
                }
                debug!("more than one item: {:?}", item_option);
            }
            item_option
        }
    }
}