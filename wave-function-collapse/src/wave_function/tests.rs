use std::collections::HashMap;

use ordered_float::OrderedFloat;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use super::probability_collection::ProbabilityCollection;
use super::probability_tree::ProbabilityTree;
use uuid::Uuid;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
struct TestStruct {
    id: String
}

impl TestStruct {
    pub fn new(id: String) -> Self {
        TestStruct {
            id: id
        }
    }
    pub fn new_random() -> Self {
        TestStruct {
            id: Uuid::new_v4().to_string()
        }
    }
}

#[cfg(test)]
mod probability_collection_unit_tests {

    use super::*;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[test]
    fn initialize() {
        let probability_collection: ProbabilityCollection<TestStruct> = ProbabilityCollection::new(HashMap::new());
    }

    #[test]
    fn probability_collection_no_items() {
        init();

        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);
        
        for _ in 0..100 {
            let probability_per_item: HashMap<TestStruct, f32> = HashMap::new();
            let mut probability_collection: ProbabilityCollection<TestStruct> = ProbabilityCollection::new(probability_per_item);
        
            for _ in 0..100 {
                let item_result = probability_collection.pop_random(&mut random_instance);
                assert!(item_result.is_none());
            }
        }
    }

    #[test]
    fn probability_collection_one_item() {
        init();
        
        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);
        
        for _ in 0..100 {
            let mut probability_per_item: HashMap<TestStruct, f32> = HashMap::new();
            probability_per_item.insert(TestStruct::new_random(), 1.0);
            let mut probability_collection: ProbabilityCollection<TestStruct> = ProbabilityCollection::new(probability_per_item);
        
            let item_result = probability_collection.pop_random(&mut random_instance);
            assert!(item_result.is_some());
            for _ in 0..100 {
                let item_result = probability_collection.pop_random(&mut random_instance);
                assert!(item_result.is_none());
            }
        }
    }

    #[test]
    fn probability_collection_many_items_equal_probability() {
        init();
        
        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);
        
        for _ in 0..100 {
            let mut probability_per_item: HashMap<TestStruct, f32> = HashMap::new();

            //let number_of_items = rng.gen::<u8>(); // TODO uncomment
            let number_of_items = 13;
            debug!("inserting {number_of_items} items");
            for _ in 0..number_of_items {
                probability_per_item.insert(TestStruct::new_random(), 1.0);
            }
            let mut probability_collection: ProbabilityCollection<TestStruct> = ProbabilityCollection::new(probability_per_item);
        
            for index in 0..number_of_items {
                debug!("pulling index {index}");
                let item_result = probability_collection.pop_random(&mut random_instance);
                assert!(item_result.is_some());
            }
            for _ in 0..100 {
                let item_result = probability_collection.pop_random(&mut random_instance);
                assert!(item_result.is_none());
            }
        }
    }

    #[test]
    fn probability_collection_many_items_one_high_probability() {
        init();

        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);
        
        for _ in 0..100 {
            let mut probability_per_item: HashMap<TestStruct, f32> = HashMap::new();

            //let number_of_items = rng.gen::<u8>(); // TODO uncomment
            let mut number_of_items = 13;
            debug!("inserting {number_of_items} items");
            for _ in 0..number_of_items {
                probability_per_item.insert(TestStruct::new_random(), 0.000001);
            }
            probability_per_item.insert(TestStruct::new(String::from("special")), 1.0);
            number_of_items += 1;
            let mut probability_collection: ProbabilityCollection<TestStruct> = ProbabilityCollection::new(probability_per_item);
        
            for index in 0..number_of_items {
                debug!("pulling index {index}");
                let item_result = probability_collection.pop_random(&mut random_instance);
                assert!(item_result.is_some());
                if index == 0 {
                    assert_eq!(item_result.unwrap().id, "special");
                }
                else {
                    assert_ne!(item_result.unwrap().id, "special");
                }
            }
            for _ in 0..100 {
                let item_result = probability_collection.pop_random(&mut random_instance);
                assert!(item_result.is_none());
            }
        }
    }
}