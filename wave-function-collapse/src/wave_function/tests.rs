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




























#[cfg(test)]
mod probability_container_unit_tests {

    use crate::wave_function::probability_container::ProbabilityContainer;

    use super::*;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[test]
    fn initialize() {
        let probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::new(HashMap::new());
    }

    #[test]
    fn probability_container_no_items() {
        init();

        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);
        
        for _ in 0..100 {
            let probability_per_item: HashMap<TestStruct, f32> = HashMap::new();
            let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::new(probability_per_item);
        
            for _ in 0..100 {
                let item_result = probability_container.pop_random(&mut random_instance);
                assert!(item_result.is_none());
            }
        }
    }

    #[test]
    fn probability_container_one_item() {
        init();
        
        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);
        
        for _ in 0..100 {
            let mut probability_per_item: HashMap<TestStruct, f32> = HashMap::new();
            probability_per_item.insert(TestStruct::new_random(), 1.0);
            let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::new(probability_per_item);
        
            let item_result = probability_container.pop_random(&mut random_instance);
            assert!(item_result.is_some());
            for _ in 0..100 {
                let item_result = probability_container.pop_random(&mut random_instance);
                assert!(item_result.is_none());
            }
        }
    }

    #[test]
    fn probability_container_many_items_equal_probability() {
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
            let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::new(probability_per_item);
        
            for index in 0..number_of_items {
                debug!("pulling index {index}");
                let item_result = probability_container.pop_random(&mut random_instance);
                assert!(item_result.is_some());
            }
            for _ in 0..100 {
                let item_result = probability_container.pop_random(&mut random_instance);
                assert!(item_result.is_none());
            }
        }
    }

    #[test]
    fn probability_container_many_items_one_high_probability() {
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
            let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::new(probability_per_item);
        
            for index in 0..number_of_items {
                debug!("pulling index {index}");
                let item_result = probability_container.pop_random(&mut random_instance);
                assert!(item_result.is_some());
                if index == 0 {
                    assert_eq!(item_result.unwrap().id, "special");
                }
                else {
                    assert_ne!(item_result.unwrap().id, "special");
                }
            }
            for _ in 0..100 {
                let item_result = probability_container.pop_random(&mut random_instance);
                assert!(item_result.is_none());
            }
        }
    }

    #[test]
    fn probability_container_ensure_get_is_equal_probability() {
        init();

        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);
        let mut probability_per_item: HashMap<TestStruct, f32> = HashMap::new();
        let number_of_items = 100;
        let mut instances_per_index: Vec<u32> = Vec::new();
        debug!("inserting {number_of_items} items");
        for index in 0..number_of_items {
            probability_per_item.insert(TestStruct::new(index.to_string()), 1.0);
            instances_per_index.push(0);
        }
        let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::new(probability_per_item);
        
        let trials = 10000000;
        for _ in 0..trials {
            let item_result = probability_container.get_random(&mut random_instance);
            assert!(item_result.is_some());
            let item_index = item_result.unwrap().id.parse::<usize>().unwrap();
            instances_per_index[item_index] += 1;
        }

        for index in 0..number_of_items {
            let instances_count = instances_per_index[index];
            let difference = instances_count.abs_diff(trials as u32 / number_of_items as u32);
            println!("difference: {difference}");
            assert!(difference < 2000);
        }

        // TODO calculate standard deviation and compare each value
    }

    #[test]
    fn probability_container_push_and_get_and_remove_repeatedly() {
        init();

        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);

        let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::default();

        for _ in 0..100 {
            let id = Uuid::new_v4().to_string();
            probability_container.push(TestStruct { id: id.clone() }, 1.0);
            let item = probability_container.get_random(&mut random_instance).unwrap();
            assert_eq!(id, item.id);
            let item = probability_container.pop_random(&mut random_instance).unwrap();
            assert_eq!(id, item.id);
        }
    }

    #[test]
    fn probability_container_push_and_get_and_remove_two_repeatedly() {
        init();

        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);

        let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::default();

        let population_total: u8 = 20;
        for _ in 0..100 {
            let mut ids: Vec<String> = Vec::new();
            for _ in 0..population_total {
                let id = Uuid::new_v4().to_string();
                ids.push(id);
            }
            for id in ids.iter() {
                probability_container.push(TestStruct { id: id.clone() }, 1.0);
            }
            /*let mut get_items: Vec<TestStruct> = Vec::new();
            for _ in 0..population_total {
                let item = probability_container.get_random(&mut random_instance).unwrap();
                get_items.push(item);
            }
            debug!("get_items: {:?}", get_items);
            let mut get_ids: Vec<String> = ids.clone();
            for (from_index, from_item) in get_items.iter().enumerate() {
                assert!(get_ids.contains(&from_item.id));
                let ids_index = get_ids.iter().position(|item| item == &from_item.id).unwrap();
                get_ids.remove(ids_index);
                for (to_index, to_item) in get_items.iter().enumerate() {
                    if from_index == to_index {
                        assert_eq!(from_item.id, to_item.id);
                    }
                    else {
                        assert_ne!(from_item.id, to_item.id);
                    }
                }
            }*/
            let mut pop_items: Vec<TestStruct> = Vec::new();
            for _ in 0..population_total {
                let item = probability_container.pop_random(&mut random_instance).unwrap();
                pop_items.push(item);
            }
            let mut pop_ids: Vec<String> = ids.clone();
            for (from_index, from_item) in pop_items.iter().enumerate() {
                assert!(pop_ids.contains(&from_item.id));
                let ids_index = pop_ids.iter().position(|item| item == &from_item.id).unwrap();
                pop_ids.remove(ids_index);
                for (to_index, to_item) in pop_items.iter().enumerate() {
                    if from_index == to_index {
                        assert_eq!(from_item.id, to_item.id);
                    }
                    else {
                        assert_ne!(from_item.id, to_item.id);
                    }
                }
            }
        }
    }

    #[test]
    #[time_graph::instrument]
    fn probability_container_get_while_removing_equal_probability() {
        init();

        time_graph::enable_data_collection(true);

        // First trial:
        //  15.37
        //  15.73
        //  16.10
        //  15.63

        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u64>();
        let mut random_instance = ChaCha8Rng::seed_from_u64(random_seed);
        let number_of_items = 20;
        
        let mut probability_per_item: HashMap<TestStruct, f32> = HashMap::new();
        //debug!("inserting {number_of_items} items");
        for index in 0..number_of_items {
            probability_per_item.insert(TestStruct::new(index.to_string()), 1.0);
        }
        let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::new(probability_per_item);
        
        let mut current_number_of_items = number_of_items;
        for _ in 0..number_of_items {

            let mut instances_per_index: Vec<u32> = Vec::new();
            for _ in 0..number_of_items {
                instances_per_index.push(0);
            }

            let trials = 10000000;

            time_graph::spanned!("get_random", {
                for _ in 0..trials {
                    let item_result = probability_container.get_random(&mut random_instance);
                    assert!(item_result.is_some());
                    let item_index = item_result.unwrap().id.parse::<usize>().unwrap();
                    instances_per_index[item_index] += 1;
                }
            });

            time_graph::spanned!("check results", {
                let mut zero_instances_count_total = 0;
                //println!("searching with node total: {current_number_of_items}");
                for index in 0..number_of_items {
                    let instances_count = instances_per_index[index];
                    if instances_count == 0 {
                        zero_instances_count_total += 1;
                    }
                    else {
                        let difference = instances_count.abs_diff(trials as u32 / current_number_of_items as u32);
                        //println!("difference: {difference}");
                        assert!(difference < 4000);
                    }
                }
                assert_eq!(number_of_items - current_number_of_items, zero_instances_count_total);
            });

            time_graph::spanned!("prepare for next loop", {
                probability_container.pop_random(&mut random_instance);
                current_number_of_items -= 1;
                instances_per_index.clear();
            });
        }

        println!("{}", time_graph::get_full_graph().as_dot());

        // TODO calculate standard deviation and compare each value
    }
}