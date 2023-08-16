mod model {
    use uuid::Uuid;

    #[derive(PartialOrd, Ord, Eq, PartialEq, Hash, Clone, Debug)]
    pub struct TestStruct {
        pub id: String
    }

    #[allow(dead_code)]
    impl TestStruct {
        pub fn new(id: String) -> Self {
            TestStruct {
                id
            }
        }
        pub fn new_random() -> Self {
            TestStruct {
                id: Uuid::new_v4().to_string()
            }
        }
    }
}

#[cfg(test)]
mod probability_collection_unit_tests {

    use std::collections::HashMap;
    use crate::wave_function::probability_collection::ProbabilityCollection;
    use super::model::TestStruct;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[test]
    fn initialize() {
        let _probability_collection: ProbabilityCollection<TestStruct> = ProbabilityCollection::new(HashMap::new());
    }

    #[test]
    fn probability_collection_no_items() {
        init();

        let mut random_instance = fastrand::Rng::new();

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

        let mut random_instance = fastrand::Rng::new();
        
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
        
        let mut random_instance = fastrand::Rng::new();
        
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

        let mut random_instance = fastrand::Rng::new();
        
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

    use std::collections::HashMap;
    use uuid::Uuid;
    use crate::wave_function::probability_container::ProbabilityContainer;
    use super::model::TestStruct;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[test]
    fn initialize() {
        let _probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::new(HashMap::new());
    }

    #[test]
    fn probability_container_no_items() {
        init();

        let mut random_instance = fastrand::Rng::new();
        
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
        
        let mut random_instance = fastrand::Rng::new();
        
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
        
        let mut random_instance = fastrand::Rng::new();
        
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

        let mut random_instance = fastrand::Rng::new();
        
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

        let mut random_instance = fastrand::Rng::new();

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
            let item_result = probability_container.peek_random(&mut random_instance);
            assert!(item_result.is_some());
            let item_index = item_result.unwrap().id.parse::<usize>().unwrap();
            instances_per_index[item_index] += 1;
        }

        for index in 0..number_of_items {
            let instances_count = instances_per_index[index];
            let difference = instances_count.abs_diff(trials as u32 / number_of_items as u32);
            //println!("difference: {difference}");
            assert!(difference < 2000);
        }

        // TODO calculate standard deviation and compare each value
    }

    #[test]
    fn probability_container_push_and_get_and_remove_repeatedly() {
        init();

        let mut random_instance = fastrand::Rng::new();

        let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::default();

        for _ in 0..100 {
            let id = Uuid::new_v4().to_string();
            probability_container.push(TestStruct { id: id.clone() }, 1.0);
            let item = probability_container.peek_random(&mut random_instance).unwrap();
            assert_eq!(id, item.id);
            let item = probability_container.pop_random(&mut random_instance).unwrap();
            assert_eq!(id, item.id);
        }
    }

    #[test]
    fn probability_container_push_and_get_and_remove_two_repeatedly() {
        init();

        let mut random_instance = fastrand::Rng::new();

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
    fn probability_container_get_while_removing_equal_probability() {
        init();

        // First trial:
        //  15.37
        //  15.73
        //  16.10
        //  15.63
        // Conclusion:
        //  getting the random item takes the most time in this test, by far, usually only needing to increment less than 50 times

        let mut random_instance = fastrand::Rng::new();

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

            for _ in 0..trials {
                let item_result = probability_container.peek_random(&mut random_instance);
                assert!(item_result.is_some());
                let item_index = item_result.unwrap().id.parse::<usize>().unwrap();
                instances_per_index[item_index] += 1;
            }

            let mut zero_instances_count_total = 0;
            //println!("searching with node total: {current_number_of_items}");
            for index in 0..number_of_items {
                let instances_count = instances_per_index[index];
                if instances_count == 0 {
                    zero_instances_count_total += 1;
                }
                else {
                    let difference = instances_count.abs_diff(trials as u32 / current_number_of_items as u32);
                    println!("difference: {difference}");
                    assert!(difference < 10000);
                }
            }
            assert_eq!(number_of_items - current_number_of_items, zero_instances_count_total);

            probability_container.pop_random(&mut random_instance);
            current_number_of_items -= 1;
            instances_per_index.clear();
        }

        // TODO calculate standard deviation and compare each value
    }

    #[test]
    fn probability_container_verify_ratio_between_equal_opportunity() {
        init();

        let mut random_instance = fastrand::Rng::new();

        let number_of_nodes = 100000;
        let number_of_items = 5;

        let mut count_per_id: HashMap<String, u32> = HashMap::new();
 
        struct TestNode {
            probability_container: ProbabilityContainer<TestStruct>
        }
        let mut nodes: Vec<TestNode> = Vec::new();
        for _ in 0..number_of_nodes {
            let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::default();
            let mut previous_probability: f32 = 1.0;
            for index in 0..number_of_items {
                let id = index.to_string();
                count_per_id.insert(id.clone(), 0);
                probability_container.push(TestStruct { id: id.clone() }, previous_probability);
                previous_probability *= 1.618033988749894;
            }
            let node = TestNode {
                probability_container: probability_container
            };
            nodes.push(node);
        }
        
        for node_index in 0..number_of_nodes as usize {
            //println!("node_index: {node_index}");
            for item_index in 0..number_of_items {
                //println!("item_index: {item_index}");
                let popped_item = nodes.get_mut(node_index).unwrap().probability_container.pop_random(&mut random_instance).unwrap();
                if popped_item.id == (number_of_items - item_index - 1).to_string() {
                    let current_count = count_per_id.get(&popped_item.id).unwrap();
                    count_per_id.insert(popped_item.id, current_count + 1);
                }
            }
        }

        for item_index in 0..(number_of_items - 1) as usize {
            println!("item count: {:?}", count_per_id.get(&item_index.to_string()).unwrap());
            let current_count = count_per_id.get(&item_index.to_string()).unwrap();
            let next_count = count_per_id.get(&(item_index + 1).to_string()).unwrap();
            let scale = (*next_count as f32) / (*current_count as f32);
            println!("scale: {scale}");
        }
        println!("item count: {:?}", count_per_id.get(&((number_of_items - 1) as usize).to_string()).unwrap());
    
        // TODO calculate standard deviation and compare each value
    }

    #[test]
    fn probability_container_create_many_instances_and_pop_random_all() {
        init();

        let mut random_instance = fastrand::Rng::new();

        let number_of_nodes = 100000;
        let number_of_items = 3;

        let mut count_per_id: HashMap<String, u32> = HashMap::new();
 
        struct TestNode {
            probability_container: ProbabilityContainer<TestStruct>
        }
        let mut nodes: Vec<TestNode> = Vec::new();
        for _ in 0..number_of_nodes {
            let mut probability_container: ProbabilityContainer<TestStruct> = ProbabilityContainer::default();
            let mut previous_probability: f32 = (2.0_f32).powf(0.0);
            for index in 0..number_of_items {
                let id = index.to_string();
                count_per_id.insert(id.clone(), 0);
                if index == 0 {
                    probability_container.push(TestStruct { id: id.clone() }, 0.00001);
                }
                else {
                    probability_container.push(TestStruct { id: id.clone() }, previous_probability);
                    previous_probability *= 2.0;
                }
            }
            let node = TestNode {
                probability_container: probability_container
            };
            nodes.push(node);
        }
        
        for node_index in 0..number_of_nodes as usize {
            //println!("node_index: {node_index}");
            for item_index in 0..number_of_items {
                //println!("item_index: {item_index}");
                let popped_item = nodes.get_mut(node_index).unwrap().probability_container.pop_random(&mut random_instance).unwrap();
                if popped_item.id == (number_of_items - item_index - 1).to_string() {
                    let current_count = count_per_id.get(&popped_item.id).unwrap();
                    count_per_id.insert(popped_item.id, current_count + 1);
                }
            }
        }

        for item_index in 0..(number_of_items - 1) as usize {
            println!("item count: {:?}", count_per_id.get(&item_index.to_string()).unwrap());
            let current_count = count_per_id.get(&item_index.to_string()).unwrap();
            let next_count = count_per_id.get(&(item_index + 1).to_string()).unwrap();
            let scale = (*next_count as f32) / (*current_count as f32);
            println!("scale: {scale}");
        }
        println!("item count: {:?}", count_per_id.get(&((number_of_items - 1) as usize).to_string()).unwrap());

        assert!(count_per_id.get("0").unwrap() > &99000);
        assert!(count_per_id.get("1").unwrap() > &60000);
        assert!(count_per_id.get("2").unwrap() > &60000);
    
        // TODO calculate standard deviation and compare each value
    }
}

#[cfg(test)]
mod wave_function_unit_tests {

    use std::collections::HashMap;
    use uuid::Uuid;
    use crate::wave_function::{Node, WaveFunction, NodeStateCollection, NodeStateProbability, collapsable_wave_function::{sequential_collapsable_wave_function::SequentialCollapsableWaveFunction, collapsable_wave_function::{CollapsedWaveFunction, CollapsedNodeState, CollapsableWaveFunction}, accommodating_collapsable_wave_function::AccommodatingCollapsableWaveFunction, accommodating_sequential_collapsable_wave_function::AccommodatingSequentialCollapsableWaveFunction}};

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[test]
    fn initialize() {
        init();

        let nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        let _wave_function = WaveFunction::new(nodes, node_state_collections);
        debug!("Succeeded to initialize WaveFunction instance.");
    }

    #[test]
    fn no_nodes() {
        init();

        let nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let validation_result = wave_function.validate();

        assert_eq!("Not all nodes connect together. At least one node must be able to traverse to all other nodes.", validation_result.err().unwrap());
    }

    #[test]
    fn one_node_no_states_sequential() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            HashMap::new(),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        assert_eq!("Cannot collapse wave function.", collapsed_wave_function_result.err().unwrap());
    }

    #[test]
    fn one_node_no_states_accommodating() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            HashMap::new(),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<String>>(None).collapse();

        assert_eq!("Cannot collapse wave function.", collapsed_wave_function_result.err().unwrap());
    }

    #[test]
    fn one_node_no_states_acc_seq() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            HashMap::new(),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(None).collapse();

        assert_eq!("Cannot collapse wave function.", collapsed_wave_function_result.err().unwrap());
    }

    #[test]
    fn one_node_one_state_sequential() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_id: String = Uuid::new_v4().to_string();
        let node_state_id: String = Uuid::new_v4().to_string();
        
        nodes.push(Node::new(
            node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        let collapsed_wave_function = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse().unwrap();
        
        assert_eq!(1, collapsed_wave_function.node_state_per_node.keys().len());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&node_id).unwrap());
    }

    #[test]
    fn one_node_one_state_accommodating() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_id: String = Uuid::new_v4().to_string();
        let node_state_id: String = Uuid::new_v4().to_string();
        
        nodes.push(Node::new(
            node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        let collapsed_wave_function = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<String>>(None).collapse().unwrap();
        
        assert_eq!(1, collapsed_wave_function.node_state_per_node.keys().len());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&node_id).unwrap());
    }

    #[test]
    fn one_node_one_state_acc_seq() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_id: String = Uuid::new_v4().to_string();
        let node_state_id: String = Uuid::new_v4().to_string();
        
        nodes.push(Node::new(
            node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        let collapsed_wave_function = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(None).collapse().unwrap();
        
        assert_eq!(1, collapsed_wave_function.node_state_per_node.keys().len());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&node_id).unwrap());
    }

    #[test]
    fn one_node_randomly_two_states_sequential() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let one_node_state_id: String = Uuid::new_v4().to_string();
        let two_node_state_id: String = Uuid::new_v4().to_string();
        let mut count_per_node_state_id: HashMap<&str, u32> = HashMap::new();
        count_per_node_state_id.insert(&one_node_state_id, 0);
        count_per_node_state_id.insert(&two_node_state_id, 0);

        let node_id: String = Uuid::new_v4().to_string();

        nodes.push(Node::new(
            node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        
        let mut random_instance = fastrand::Rng::new();
        
        for _ in 0..100000 {
            let random_seed = Some(random_instance.u64(..));
            let collapsed_wave_function = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse().unwrap();

            let node_state_id: &str = collapsed_wave_function.node_state_per_node.get(&node_id).unwrap();
            *count_per_node_state_id.get_mut(node_state_id).unwrap() += 1;
        }

        println!("count_per_node_state_id: {:?}", count_per_node_state_id);
        assert!(count_per_node_state_id.get(one_node_state_id.as_str()).unwrap() > &49000, "The first node state was less than expected.");
        assert!(count_per_node_state_id.get(two_node_state_id.as_str()).unwrap() > &49000, "The first node state was less than expected.");
    }

    #[test]
    fn one_node_randomly_two_states_accommodating() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let one_node_state_id: String = Uuid::new_v4().to_string();
        let two_node_state_id: String = Uuid::new_v4().to_string();
        let mut count_per_node_state_id: HashMap<&str, u32> = HashMap::new();
        count_per_node_state_id.insert(&one_node_state_id, 0);
        count_per_node_state_id.insert(&two_node_state_id, 0);

        let node_id: String = Uuid::new_v4().to_string();

        nodes.push(Node::new(
            node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        
        let mut random_instance = fastrand::Rng::new();

        for _ in 0..100000 {
            let random_seed = Some(random_instance.u64(..));
            let collapsed_wave_function = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<String>>(random_seed).collapse().unwrap();

            let node_state_id: &str = collapsed_wave_function.node_state_per_node.get(&node_id).unwrap();
            *count_per_node_state_id.get_mut(node_state_id).unwrap() += 1;
        }

        println!("count_per_node_state_id: {:?}", count_per_node_state_id);
        assert!(count_per_node_state_id.get(one_node_state_id.as_str()).unwrap() > &49000, "The first node state was less than expected.");
        assert!(count_per_node_state_id.get(two_node_state_id.as_str()).unwrap() > &49000, "The first node state was less than expected.");
    }

    #[test]
    fn one_node_randomly_two_states_acc_seq() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let one_node_state_id: String = Uuid::new_v4().to_string();
        let two_node_state_id: String = Uuid::new_v4().to_string();
        let mut count_per_node_state_id: HashMap<&str, u32> = HashMap::new();
        count_per_node_state_id.insert(&one_node_state_id, 0);
        count_per_node_state_id.insert(&two_node_state_id, 0);

        let node_id: String = Uuid::new_v4().to_string();

        nodes.push(Node::new(
            node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();
        
        let mut random_instance = fastrand::Rng::new();

        for _ in 0..100000 {
            let random_seed = Some(random_instance.u64(..));
            let collapsed_wave_function = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(random_seed).collapse().unwrap();

            let node_state_id: &str = collapsed_wave_function.node_state_per_node.get(&node_id).unwrap();
            *count_per_node_state_id.get_mut(node_state_id).unwrap() += 1;
        }

        println!("count_per_node_state_id: {:?}", count_per_node_state_id);
        assert!(count_per_node_state_id.get(one_node_state_id.as_str()).unwrap() > &49000, "The first node state was less than expected.");
        assert!(count_per_node_state_id.get(two_node_state_id.as_str()).unwrap() > &49000, "The second node state was less than expected.");
    }

    #[test]
    fn two_nodes_without_neighbors() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![Uuid::new_v4().to_string()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![Uuid::new_v4().to_string()]),
            HashMap::new()
        ));

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        let validation_result = wave_function.validate();
        assert_eq!("Not all nodes connect together. At least one node must be able to traverse to all other nodes.", validation_result.err().unwrap());
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor_restriction_ignored_sequential() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let unrestricted_node_state_id: String = String::from("unrestricted");
        let from_restrictive_node_state_id: String = String::from("from_restrictive");
        let to_restrictive_node_state_id: String = String::from("to_restrictive");

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![from_restrictive_node_state_id.clone(), unrestricted_node_state_id.clone()]),
            HashMap::new()
        ));

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![unrestricted_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let restrictive_node_state_collection_id: String = Uuid::new_v4().to_string();
        let restrictive_node_state_collection = NodeStateCollection::new(
            restrictive_node_state_collection_id.clone(),
            from_restrictive_node_state_id.clone(),
            vec![to_restrictive_node_state_id.clone()]
        );
        node_state_collections.push(restrictive_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(restrictive_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&unrestricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&unrestricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor_restriction_ignored_accommodating() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let unrestricted_node_state_id: String = String::from("unrestricted");
        let from_restrictive_node_state_id: String = String::from("from_restrictive");
        let to_restrictive_node_state_id: String = String::from("to_restrictive");

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![from_restrictive_node_state_id.clone(), unrestricted_node_state_id.clone()]),
            HashMap::new()
        ));

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![unrestricted_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let restrictive_node_state_collection_id: String = Uuid::new_v4().to_string();
        let restrictive_node_state_collection = NodeStateCollection::new(
            restrictive_node_state_collection_id.clone(),
            from_restrictive_node_state_id.clone(),
            vec![to_restrictive_node_state_id.clone()]
        );
        node_state_collections.push(restrictive_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(restrictive_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&unrestricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&unrestricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor_restriction_ignored_acc_seq() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let unrestricted_node_state_id: String = String::from("unrestricted");
        let from_restrictive_node_state_id: String = String::from("from_restrictive");
        let to_restrictive_node_state_id: String = String::from("to_restrictive");

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![from_restrictive_node_state_id.clone(), unrestricted_node_state_id.clone()]),
            HashMap::new()
        ));

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![unrestricted_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let restrictive_node_state_collection_id: String = Uuid::new_v4().to_string();
        let restrictive_node_state_collection = NodeStateCollection::new(
            restrictive_node_state_collection_id.clone(),
            from_restrictive_node_state_id.clone(),
            vec![to_restrictive_node_state_id.clone()]
        );
        node_state_collections.push(restrictive_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(restrictive_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&unrestricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&unrestricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_with_parent_unrestricted_and_child_only_one_state_restricted_sequential() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let restricting_node_state_id: String = String::from("restricting");
        let restricted_node_state_id: String = String::from("restricted");
        let permitting_node_state_id: String = String::from("z_permitting");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![restricting_node_state_id.clone(), permitting_node_state_id.clone()]),
            HashMap::new()
        ));

        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(vec![restricted_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let restrictive_node_state_collection_id: String = Uuid::new_v4().to_string();
        let restrictive_node_state_collection = NodeStateCollection::new(
            restrictive_node_state_collection_id.clone(),
            restricting_node_state_id.clone(),
            vec![]
        );
        node_state_collections.push(restrictive_node_state_collection);

        let permitted_node_state_collection_id: String = Uuid::new_v4().to_string();
        let permitted_node_state_collection = NodeStateCollection::new(
            permitted_node_state_collection_id.clone(),
            permitting_node_state_id.clone(),
            vec![restricted_node_state_id.clone()]
        );
        node_state_collections.push(permitted_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(restrictive_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(permitted_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&permitting_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&restricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_with_parent_unrestricted_and_child_only_one_state_restricted_acc_seq() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let restricting_node_state_id: String = String::from("restricting");
        let restricted_node_state_id: String = String::from("restricted");
        let permitting_node_state_id: String = String::from("z_permitting");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![restricting_node_state_id.clone(), permitting_node_state_id.clone()]),
            HashMap::new()
        ));

        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(vec![restricted_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let restrictive_node_state_collection_id: String = Uuid::new_v4().to_string();
        let restrictive_node_state_collection = NodeStateCollection::new(
            restrictive_node_state_collection_id.clone(),
            restricting_node_state_id.clone(),
            vec![]
        );
        node_state_collections.push(restrictive_node_state_collection);

        let permitted_node_state_collection_id: String = Uuid::new_v4().to_string();
        let permitted_node_state_collection = NodeStateCollection::new(
            permitted_node_state_collection_id.clone(),
            permitting_node_state_id.clone(),
            vec![restricted_node_state_id.clone()]
        );
        node_state_collections.push(permitted_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(restrictive_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(permitted_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&permitting_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&restricted_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_with_child_two_states_restricted_and_parent_one_state_unrestricted_sequential() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let restricting_node_state_id: String = String::from("restricting");
        let restricted_node_state_id: String = String::from("restricted");
        let permitted_node_state_id: String = String::from("z_permitted");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![restricting_node_state_id.clone()]),
            HashMap::new()
        ));

        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(vec![restricted_node_state_id.clone(), permitted_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let restrictive_node_state_collection_id: String = Uuid::new_v4().to_string();
        let restrictive_node_state_collection = NodeStateCollection::new(
            restrictive_node_state_collection_id.clone(),
            restricting_node_state_id.clone(),
            vec![permitted_node_state_id.clone()]
        );
        node_state_collections.push(restrictive_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(restrictive_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&restricting_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&permitted_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_with_child_two_states_restricted_and_parent_one_state_unrestricted_acc_seq() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let restricting_node_state_id: String = String::from("restricting");
        let restricted_node_state_id: String = String::from("restricted");
        let permitted_node_state_id: String = String::from("z_permitted");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![restricting_node_state_id.clone()]),
            HashMap::new()
        ));

        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(vec![restricted_node_state_id.clone(), permitted_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let restrictive_node_state_collection_id: String = Uuid::new_v4().to_string();
        let restrictive_node_state_collection = NodeStateCollection::new(
            restrictive_node_state_collection_id.clone(),
            restricting_node_state_id.clone(),
            vec![permitted_node_state_id.clone()]
        );
        node_state_collections.push(restrictive_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(restrictive_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&restricting_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&permitted_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor_ordered() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_state_id: String = Uuid::new_v4().to_string();

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let mut neighbor_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..1000 {
            neighbor_node_state_ids.push(Uuid::new_v4().to_string());
        }
        neighbor_node_state_ids.push(node_state_id.clone());

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(neighbor_node_state_ids),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let same_node_state_collection_id: String = Uuid::new_v4().to_string();
        let same_node_state_collection = NodeStateCollection::new(
            same_node_state_collection_id.clone(),
            node_state_id.clone(),
            vec![node_state_id.clone()]
        );
        node_state_collections.push(same_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_with_only_one_is_a_neighbor_disordered() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_state_id: String = Uuid::new_v4().to_string();

        let mut neighbor_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..1 {
            neighbor_node_state_ids.push(Uuid::new_v4().to_string());
        }
        neighbor_node_state_ids.push(node_state_id.clone());

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(neighbor_node_state_ids),
            HashMap::new()
        ));

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[1].id.clone();
        let second_node_id: String = nodes[0].id.clone();

        let same_node_state_collection_id: String = Uuid::new_v4().to_string();
        let same_node_state_collection = NodeStateCollection::new(
            same_node_state_collection_id.clone(),
            node_state_id.clone(),
            vec![node_state_id.clone()]
        );
        node_state_collections.push(same_node_state_collection);

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();
        let collapsed_wave_function = collapsed_wave_function_result.unwrap();

        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_both_as_neighbors_only_ordered() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_state_id: String = String::from("state_A");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let mut neighbor_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..1000 {
            neighbor_node_state_ids.push(Uuid::new_v4().to_string());
        }
        neighbor_node_state_ids.push(node_state_id.clone());

        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(neighbor_node_state_ids),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let same_node_state_collection_id: String = Uuid::new_v4().to_string();
        let same_node_state_collection = NodeStateCollection::new(
            same_node_state_collection_id.clone(),
            node_state_id.clone(),
            vec![node_state_id.clone()]
        );
        node_state_collections.push(same_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(same_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_both_as_neighbors_only_disordered() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_state_id: String = String::from("state_A");

        let mut neighbor_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..1000 {
            neighbor_node_state_ids.push(Uuid::new_v4().to_string());
        }
        neighbor_node_state_ids.push(node_state_id.clone());

        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(neighbor_node_state_ids),
            HashMap::new()
        ));

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let same_node_state_collection_id: String = Uuid::new_v4().to_string();
        let same_node_state_collection = NodeStateCollection::new(
            same_node_state_collection_id.clone(),
            node_state_id.clone(),
            vec![node_state_id.clone()]
        );
        node_state_collections.push(same_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(same_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
    }

    #[test]
    fn two_nodes_both_as_neighbors_and_different_states_with_one_run() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let one_node_state_id: String = Uuid::new_v4().to_string();
        let two_node_state_id: String = Uuid::new_v4().to_string();

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let if_one_not_two_node_state_collection_id: String = Uuid::new_v4().to_string();
        let if_one_not_two_node_state_collection = NodeStateCollection::new(
            if_one_not_two_node_state_collection_id.clone(),
            one_node_state_id.clone(),
            vec![two_node_state_id.clone()]
        );
        node_state_collections.push(if_one_not_two_node_state_collection);

        let if_two_not_one_node_state_collection_id: String = Uuid::new_v4().to_string();
        let if_two_not_one_node_state_collection = NodeStateCollection::new(
            if_two_not_one_node_state_collection_id.clone(),
            two_node_state_id.clone(),
            vec![one_node_state_id.clone()]
        );
        node_state_collections.push(if_two_not_one_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_one_not_two_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_two_not_one_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_one_not_two_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_two_not_one_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        // NOTE: this cannot be used because the uuids sort into random orders
        //assert_eq!(&one_node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        //assert_eq!(&two_node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());

        assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
    }

    #[test]
    fn two_nodes_both_as_neighbors_and_different_states_with_random_runs() {
        init();

        let mut random_instance = fastrand::Rng::new();

        for _ in 0..10 {
            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

            let one_node_state_id: String = Uuid::new_v4().to_string();
            let two_node_state_id: String = Uuid::new_v4().to_string();

            nodes.push(Node::new(
                Uuid::new_v4().to_string(),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                Uuid::new_v4().to_string(),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));

            let first_node_id: String = nodes[0].id.clone();
            let second_node_id: String = nodes[1].id.clone();

            let if_one_not_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_one_not_two_node_state_collection = NodeStateCollection::new(
                if_one_not_two_node_state_collection_id.clone(),
                one_node_state_id.clone(),
                vec![two_node_state_id.clone()]
            );
            node_state_collections.push(if_one_not_two_node_state_collection);

            let if_two_not_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_two_not_one_node_state_collection = NodeStateCollection::new(
                if_two_not_one_node_state_collection_id.clone(),
                two_node_state_id.clone(),
                vec![one_node_state_id.clone()]
            );
            node_state_collections.push(if_two_not_one_node_state_collection);

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_one_not_two_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_two_not_one_node_state_collection_id.clone());

            nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_one_not_two_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_two_not_one_node_state_collection_id.clone());

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            let random_seed = Some(random_instance.u64(..));

            let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse();

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        }
    }

    #[test]
    fn two_nodes_both_as_neighbors_with_conflicting_state_requirements() {
        init();

        let mut random_instance = fastrand::Rng::new();

        for _ in 0..10 {
            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");
            let three_node_state_id: String = String::from("state_C");
            let four_node_state_id: String = String::from("state_D");

            nodes.push(Node::new(
                String::from("node_1"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone(), three_node_state_id.clone(), four_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_2"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone(), three_node_state_id.clone(), four_node_state_id.clone()]),
                HashMap::new()
            ));

            let first_node_id: String = nodes[0].id.clone();
            let second_node_id: String = nodes[1].id.clone();

            let if_one_then_three_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_one_then_three_node_state_collection = NodeStateCollection::new(
                if_one_then_three_node_state_collection_id.clone(),
                one_node_state_id.clone(),
                vec![three_node_state_id.clone()]
            );
            node_state_collections.push(if_one_then_three_node_state_collection);

            let if_two_then_four_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_two_then_four_node_state_collection = NodeStateCollection::new(
                if_two_then_four_node_state_collection_id.clone(),
                two_node_state_id.clone(),
                vec![four_node_state_id.clone()]
            );
            node_state_collections.push(if_two_then_four_node_state_collection);

            let if_three_then_no_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_three_then_no_node_state_collection = NodeStateCollection::new(
                if_three_then_no_node_state_collection_id.clone(),
                three_node_state_id.clone(),
                Vec::new()
            );
            node_state_collections.push(if_three_then_no_node_state_collection);

            let if_four_then_no_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_four_then_no_node_state_collection = NodeStateCollection::new(
                if_four_then_no_node_state_collection_id.clone(),
                four_node_state_id.clone(),
                Vec::new()
            );
            node_state_collections.push(if_four_then_no_node_state_collection);

            let if_three_then_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_three_then_two_node_state_collection = NodeStateCollection::new(
                if_three_then_two_node_state_collection_id.clone(),
                three_node_state_id.clone(),
                vec![two_node_state_id.clone()]
            );
            node_state_collections.push(if_three_then_two_node_state_collection);

            let if_four_then_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_four_then_one_node_state_collection = NodeStateCollection::new(
                if_four_then_one_node_state_collection_id.clone(),
                four_node_state_id.clone(),
                vec![one_node_state_id.clone()]
            );
            node_state_collections.push(if_four_then_one_node_state_collection);

            let if_one_then_no_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_one_then_no_node_state_collection = NodeStateCollection::new(
                if_one_then_no_node_state_collection_id.clone(),
                one_node_state_id.clone(),
                Vec::new()
            );
            node_state_collections.push(if_one_then_no_node_state_collection);

            let if_two_then_no_node_state_collection_id: String = Uuid::new_v4().to_string();
            let if_two_then_no_node_state_collection = NodeStateCollection::new(
                if_two_then_no_node_state_collection_id.clone(),
                two_node_state_id.clone(),
                Vec::new()
            );
            node_state_collections.push(if_two_then_no_node_state_collection);

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_one_then_three_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_two_then_four_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_three_then_no_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(if_four_then_no_node_state_collection_id.clone());

            nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_three_then_two_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_four_then_one_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_one_then_no_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(if_two_then_no_node_state_collection_id.clone());

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            let random_seed = Some(random_instance.u64(..));

            let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse();

            assert_eq!("Cannot collapse wave function.", collapsed_wave_function_result.err().unwrap());
        }
    }

    #[test]
    fn three_nodes_as_neighbors_all_same_state() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_state_id: String = String::from("state_A");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            String::from("node_3"),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();
        let third_node_id: String = nodes[2].id.clone();

        let same_node_state_collection_id: String = String::from("nsc_1");
        let same_node_state_collection = NodeStateCollection::new(
            same_node_state_collection_id.clone(),
            node_state_id.clone(),
            vec![node_state_id.clone()]
        );
        node_state_collections.push(same_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(same_node_state_collection_id.clone());

        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(same_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        assert_eq!(&node_state_id, collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
    }

    #[test]
    fn three_nodes_as_dense_neighbors_all_different_states_sequential() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let first_node_state_id: String = String::from("state_A");
        let second_node_state_id: String = String::from("state_B");
        let third_node_state_id: String = String::from("state_C");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            String::from("node_3"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();
        let third_node_id: String = nodes[2].id.clone();

        let all_but_first_node_state_collection_id: String = String::from("nsc_1");
        let all_but_first_node_state_collection = NodeStateCollection::new(
            all_but_first_node_state_collection_id.clone(),
            first_node_state_id.clone(),
            vec![second_node_state_id.clone(), third_node_state_id.clone()]
        );
        node_state_collections.push(all_but_first_node_state_collection);

        let all_but_second_node_state_collection_id: String = String::from("nsc_2");
        let all_but_second_node_state_collection = NodeStateCollection::new(
            all_but_second_node_state_collection_id.clone(),
            second_node_state_id.clone(),
            vec![first_node_state_id.clone(), third_node_state_id.clone()]
        );
        node_state_collections.push(all_but_second_node_state_collection);

        let all_but_third_node_state_collection_id: String = String::from("nsc_3");
        let all_but_third_node_state_collection = NodeStateCollection::new(
            all_but_third_node_state_collection_id.clone(),
            third_node_state_id.clone(),
            vec![first_node_state_id.clone(), second_node_state_id.clone()]
        );
        node_state_collections.push(all_but_third_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        debug!("collapsed_wave_function.node_state_per_node: {:?}", collapsed_wave_function.node_state_per_node);

        assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
    }

    #[test]
    fn three_nodes_as_dense_neighbors_all_different_states_accommodating() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let first_node_state_id: String = String::from("state_A");
        let second_node_state_id: String = String::from("state_B");
        let third_node_state_id: String = String::from("state_C");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            String::from("node_3"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();
        let third_node_id: String = nodes[2].id.clone();

        let all_but_first_node_state_collection_id: String = String::from("nsc_1");
        let all_but_first_node_state_collection = NodeStateCollection::new(
            all_but_first_node_state_collection_id.clone(),
            first_node_state_id.clone(),
            vec![second_node_state_id.clone(), third_node_state_id.clone()]
        );
        node_state_collections.push(all_but_first_node_state_collection);

        let all_but_second_node_state_collection_id: String = String::from("nsc_2");
        let all_but_second_node_state_collection = NodeStateCollection::new(
            all_but_second_node_state_collection_id.clone(),
            second_node_state_id.clone(),
            vec![first_node_state_id.clone(), third_node_state_id.clone()]
        );
        node_state_collections.push(all_but_second_node_state_collection);

        let all_but_third_node_state_collection_id: String = String::from("nsc_3");
        let all_but_third_node_state_collection = NodeStateCollection::new(
            all_but_third_node_state_collection_id.clone(),
            third_node_state_id.clone(),
            vec![first_node_state_id.clone(), second_node_state_id.clone()]
        );
        node_state_collections.push(all_but_third_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        debug!("collapsed_wave_function.node_state_per_node: {:?}", collapsed_wave_function.node_state_per_node);

        assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
    }

    #[test]
    fn three_nodes_as_dense_neighbors_all_different_states_acc_seq() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let first_node_state_id: String = String::from("state_A");
        let second_node_state_id: String = String::from("state_B");
        let third_node_state_id: String = String::from("state_C");

        nodes.push(Node::new(
            String::from("node_1"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            String::from("node_2"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            String::from("node_3"),
            NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();
        let third_node_id: String = nodes[2].id.clone();

        let all_but_first_node_state_collection_id: String = String::from("nsc_1");
        let all_but_first_node_state_collection = NodeStateCollection::new(
            all_but_first_node_state_collection_id.clone(),
            first_node_state_id.clone(),
            vec![second_node_state_id.clone(), third_node_state_id.clone()]
        );
        node_state_collections.push(all_but_first_node_state_collection);

        let all_but_second_node_state_collection_id: String = String::from("nsc_2");
        let all_but_second_node_state_collection = NodeStateCollection::new(
            all_but_second_node_state_collection_id.clone(),
            second_node_state_id.clone(),
            vec![first_node_state_id.clone(), third_node_state_id.clone()]
        );
        node_state_collections.push(all_but_second_node_state_collection);

        let all_but_third_node_state_collection_id: String = String::from("nsc_3");
        let all_but_third_node_state_collection = NodeStateCollection::new(
            all_but_third_node_state_collection_id.clone(),
            third_node_state_id.clone(),
            vec![first_node_state_id.clone(), second_node_state_id.clone()]
        );
        node_state_collections.push(all_but_third_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
        nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        debug!("collapsed_wave_function.node_state_per_node: {:?}", collapsed_wave_function.node_state_per_node);

        assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
        assert_ne!(collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap(), collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap());
    }

    #[test]
    fn three_nodes_as_dense_neighbors_randomly_all_different_states() {
        init();

        let mut random_instance = fastrand::Rng::new();

        for _ in 0..10 {
            
            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

            let first_node_state_id: String = String::from("state_A");
            let second_node_state_id: String = String::from("state_B");
            let third_node_state_id: String = String::from("state_C");

            nodes.push(Node::new(
                String::from("node_1"),
                NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_2"),
                NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_3"),
                NodeStateProbability::get_equal_probability(vec![first_node_state_id.clone(), second_node_state_id.clone(), third_node_state_id.clone()]),
                HashMap::new()
            ));

            let first_node_id: String = nodes[0].id.clone();
            let second_node_id: String = nodes[1].id.clone();
            let third_node_id: String = nodes[2].id.clone();

            let all_but_first_node_state_collection_id: String = String::from("nsc_1");
            let all_but_first_node_state_collection = NodeStateCollection::new(
                all_but_first_node_state_collection_id.clone(),
                first_node_state_id.clone(),
                vec![second_node_state_id.clone(), third_node_state_id.clone()]
            );
            node_state_collections.push(all_but_first_node_state_collection);

            let all_but_second_node_state_collection_id: String = String::from("nsc_2");
            let all_but_second_node_state_collection = NodeStateCollection::new(
                all_but_second_node_state_collection_id.clone(),
                second_node_state_id.clone(),
                vec![first_node_state_id.clone(), third_node_state_id.clone()]
            );
            node_state_collections.push(all_but_second_node_state_collection);

            let all_but_third_node_state_collection_id: String = String::from("nsc_3");
            let all_but_third_node_state_collection = NodeStateCollection::new(
                all_but_third_node_state_collection_id.clone(),
                third_node_state_id.clone(),
                vec![first_node_state_id.clone(), second_node_state_id.clone()]
            );
            node_state_collections.push(all_but_third_node_state_collection);

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

            nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(third_node_id.clone(), Vec::new());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&third_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

            nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_first_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_second_node_state_collection_id.clone());
            nodes[2].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(all_but_third_node_state_collection_id.clone());

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();
            let random_seed = Some(random_instance.u64(..));

            let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse();
            
            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            let first_node_state_id = collapsed_wave_function.node_state_per_node.get(&first_node_id).unwrap();
            let second_node_state_id = collapsed_wave_function.node_state_per_node.get(&second_node_id).unwrap();
            let third_node_state_id = collapsed_wave_function.node_state_per_node.get(&third_node_id).unwrap();
            assert_ne!(second_node_state_id, first_node_state_id);
            assert_ne!(third_node_state_id, first_node_state_id);
            assert_ne!(first_node_state_id, second_node_state_id);
            assert_ne!(third_node_state_id, second_node_state_id);
            assert_ne!(first_node_state_id, third_node_state_id);
            assert_ne!(second_node_state_id, third_node_state_id);
        }
    }
    
    #[test]
    fn many_nodes_as_dense_neighbors_all_different_states_sequential() {
        //init();

        let nodes_total = 50;

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_ids: Vec<String> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_collection_ids: Vec<String> = Vec::new();

        for _ in 0..nodes_total {
            node_state_ids.push(Uuid::new_v4().to_string());
        }

        for _index in 0..nodes_total {
            let node_id: String = Uuid::new_v4().to_string();
            node_ids.push(node_id.clone());
            let node = Node::new(
                node_id,
                NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                HashMap::new()
            );
            nodes.push(node);
        }

        for node_state_id in node_state_ids.iter() {
            let mut other_node_state_ids: Vec<String> = Vec::new();
            for other_node_state_id in node_state_ids.iter() {
                if node_state_id != other_node_state_id {
                    other_node_state_ids.push(other_node_state_id.clone());
                }
            }
            
            let node_state_collection_id: String = Uuid::new_v4().to_string();
            node_state_collection_ids.push(node_state_collection_id.clone());
            node_state_collections.push(NodeStateCollection::new(
                node_state_collection_id,
                node_state_id.clone(),
                other_node_state_ids
            ));
        }

        // tie nodes to their neighbors
        for node in nodes.iter_mut() {
            for other_node_id in node_ids.iter() {
                if *other_node_id != node.id {
                    node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                }
            }
        }

        let wave_function: WaveFunction<String>;

        wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        let collapsed_wave_function_result: Result<CollapsedWaveFunction<String>, String>;

        collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        // check that no nodes have the same state
        for (first_index, (first_node, first_node_state)) in collapsed_wave_function.node_state_per_node.iter().enumerate() {
            for (second_index, (second_node, second_node_state)) in collapsed_wave_function.node_state_per_node.iter().enumerate() {
                if first_index == second_index {
                    assert_eq!(first_node, second_node);
                    assert_eq!(first_node_state, second_node_state);
                }
                else {
                    assert_ne!(first_node, second_node);
                    assert_ne!(first_node_state, second_node_state);
                }
            }
        }

    }
    
    #[test]
    fn many_nodes_as_dense_neighbors_all_different_states_accommodating() {
        //init();

        let nodes_total = 50;

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_ids: Vec<String> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_collection_ids: Vec<String> = Vec::new();

        for _ in 0..nodes_total {
            node_state_ids.push(Uuid::new_v4().to_string());
        }

        for _index in 0..nodes_total {
            let node_id: String = Uuid::new_v4().to_string();
            node_ids.push(node_id.clone());
            let node = Node::new(
                node_id,
                NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                HashMap::new()
            );
            nodes.push(node);
        }

        for node_state_id in node_state_ids.iter() {
            let mut other_node_state_ids: Vec<String> = Vec::new();
            for other_node_state_id in node_state_ids.iter() {
                if node_state_id != other_node_state_id {
                    other_node_state_ids.push(other_node_state_id.clone());
                }
            }
            
            let node_state_collection_id: String = Uuid::new_v4().to_string();
            node_state_collection_ids.push(node_state_collection_id.clone());
            node_state_collections.push(NodeStateCollection::new(
                node_state_collection_id,
                node_state_id.clone(),
                other_node_state_ids
            ));
        }

        // tie nodes to their neighbors
        for node in nodes.iter_mut() {
            for other_node_id in node_ids.iter() {
                if *other_node_id != node.id {
                    node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                }
            }
        }

        let wave_function: WaveFunction<String>;

        wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        let collapsed_wave_function_result: Result<CollapsedWaveFunction<String>, String>;

        collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        // check that no nodes have the same state
        for (first_index, (first_node, first_node_state)) in collapsed_wave_function.node_state_per_node.iter().enumerate() {
            for (second_index, (second_node, second_node_state)) in collapsed_wave_function.node_state_per_node.iter().enumerate() {
                if first_index == second_index {
                    assert_eq!(first_node, second_node);
                    assert_eq!(first_node_state, second_node_state);
                }
                else {
                    assert_ne!(first_node, second_node);
                    assert_ne!(first_node_state, second_node_state);
                }
            }
        }

    }
    
    #[test]
    fn many_nodes_as_dense_neighbors_all_different_states_acc_seq() {
        //init();

        let nodes_total = 50;

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_ids: Vec<String> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_collection_ids: Vec<String> = Vec::new();

        for _ in 0..nodes_total {
            node_state_ids.push(Uuid::new_v4().to_string());
        }

        for _index in 0..nodes_total {
            let node_id: String = Uuid::new_v4().to_string();
            node_ids.push(node_id.clone());
            let node = Node::new(
                node_id,
                NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                HashMap::new()
            );
            nodes.push(node);
        }

        for node_state_id in node_state_ids.iter() {
            let mut other_node_state_ids: Vec<String> = Vec::new();
            for other_node_state_id in node_state_ids.iter() {
                if node_state_id != other_node_state_id {
                    other_node_state_ids.push(other_node_state_id.clone());
                }
            }
            
            let node_state_collection_id: String = Uuid::new_v4().to_string();
            node_state_collection_ids.push(node_state_collection_id.clone());
            node_state_collections.push(NodeStateCollection::new(
                node_state_collection_id,
                node_state_id.clone(),
                other_node_state_ids
            ));
        }

        // tie nodes to their neighbors
        for node in nodes.iter_mut() {
            for other_node_id in node_ids.iter() {
                if *other_node_id != node.id {
                    node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                }
            }
        }

        let wave_function: WaveFunction<String>;

        wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        let collapsed_wave_function_result: Result<CollapsedWaveFunction<String>, String>;

        collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        // check that no nodes have the same state
        for (first_index, (first_node, first_node_state)) in collapsed_wave_function.node_state_per_node.iter().enumerate() {
            for (second_index, (second_node, second_node_state)) in collapsed_wave_function.node_state_per_node.iter().enumerate() {
                if first_index == second_index {
                    assert_eq!(first_node, second_node);
                    assert_eq!(first_node_state, second_node_state);
                }
                else {
                    assert_ne!(first_node, second_node);
                    assert_ne!(first_node_state, second_node_state);
                }
            }
        }

    }

    #[test]
    fn many_nodes_as_dense_neighbors_randomly_all_different_states() {
        //init();

        let mut random_instance = fastrand::Rng::new();

        for _ in 0..10 {

            let nodes_total = 20;

            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_ids: Vec<String> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
            let mut node_state_ids: Vec<String> = Vec::new();
            let mut node_state_collection_ids: Vec<String> = Vec::new();

            for _ in 0..nodes_total {
                node_state_ids.push(Uuid::new_v4().to_string());
            }

            for _index in 0..nodes_total {
                let node_id: String = Uuid::new_v4().to_string();
                node_ids.push(node_id.clone());
                let node = Node::new(
                    node_id,
                    NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                    HashMap::new()
                );
                nodes.push(node);
            }

            for node_state_id in node_state_ids.iter() {
                let mut other_node_state_ids: Vec<String> = Vec::new();
                for other_node_state_id in node_state_ids.iter() {
                    if node_state_id != other_node_state_id {
                        other_node_state_ids.push(other_node_state_id.clone());
                    }
                }
                
                let node_state_collection_id: String = Uuid::new_v4().to_string();
                node_state_collection_ids.push(node_state_collection_id.clone());
                node_state_collections.push(NodeStateCollection::new(
                    node_state_collection_id,
                    node_state_id.clone(),
                    other_node_state_ids
                ));
            }

            // tie nodes to their neighbors
            for node in nodes.iter_mut() {
                for other_node_id in node_ids.iter() {
                    if *other_node_id != node.id {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                    }
                }
            }

            let wave_function: WaveFunction<String>;

            wave_function = WaveFunction::new(nodes, node_state_collections);

            wave_function.validate().unwrap();

            let collapsed_wave_function_result: Result<CollapsedWaveFunction<String>, String>;
            
            let random_seed = Some(random_instance.u64(..));
            collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse();

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let _collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            let mut all_node_state_ids: Vec<String> = Vec::new();
            for (node_state_id, _node_id) in std::iter::zip(&node_state_ids, &node_ids) {
                if !all_node_state_ids.contains(&node_state_id) {
                    all_node_state_ids.push(node_state_id.clone());
                }
            }

            assert_eq!(nodes_total, all_node_state_ids.len());
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn many_nodes_as_3D_grid_all_different_states_sequential() {
        init();

        let nodes_height = 25;
        let nodes_width = 25;
        let nodes_depth = 25;
        let nodes_total = nodes_height * nodes_width * nodes_depth;
        let node_states_total = 12;

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_ids: Vec<String> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_collection_ids: Vec<String> = Vec::new();

        for _ in 0..node_states_total {
            node_state_ids.push(Uuid::new_v4().to_string());
        }

        for _ in 0..nodes_total {
            let node_id: String = Uuid::new_v4().to_string();
            node_ids.push(node_id.clone());
            nodes.push(Node::new(
                node_id,
                NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                HashMap::new()
            ));
        }

        for node_state_id in node_state_ids.iter() {
            let mut other_node_state_ids: Vec<String> = Vec::new();
            for other_node_state_id in node_state_ids.iter() {
                if node_state_id != other_node_state_id {
                    other_node_state_ids.push(other_node_state_id.clone());
                }
            }
            
            let node_state_collection_id: String = Uuid::new_v4().to_string();
            node_state_collection_ids.push(node_state_collection_id.clone());
            node_state_collections.push(NodeStateCollection::new(
                node_state_collection_id,
                node_state_id.clone(),
                other_node_state_ids
            ));
        }

        // tie nodes to their neighbors
        for (node_index, node) in std::iter::zip(0..nodes_total, nodes.iter_mut()) {
            let node_x: i32 = node_index % nodes_width;
            let node_y: i32 = (node_index / nodes_width) % nodes_height;
            let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
            //debug!("processing node {node_x}, {node_y}, {node_z}.");
            for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                let other_node_x: i32 = other_node_index % nodes_width;
                let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                    //debug!("found neighbor at {other_node_x}, {other_node_y}, {other_node_z}.");
                    node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                }
            }
        }

        let wave_function: WaveFunction<String>;

        wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        let collapsed_wave_function_result: Result<CollapsedWaveFunction<String>, String>;
        
        collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        // check that none of the neighbors match the same state
        for (node_index, node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
            let node_x: i32 = node_index % nodes_width;
            let node_y: i32 = (node_index / nodes_width) % nodes_height;
            let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
            for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                let other_node_x: i32 = other_node_index % nodes_width;
                let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                    assert_ne!(collapsed_wave_function.node_state_per_node.get(node_id), collapsed_wave_function.node_state_per_node.get(other_node_id));
                }
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn many_nodes_as_3D_grid_all_different_states_accommodating() {
        init();

        let nodes_height = 3;
        let nodes_width = 3;
        let nodes_depth = 3;
        let nodes_total = nodes_height * nodes_width * nodes_depth;
        let node_states_total = 12;

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_ids: Vec<String> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_collection_ids: Vec<String> = Vec::new();

        for _ in 0..node_states_total {
            node_state_ids.push(Uuid::new_v4().to_string());
        }

        for _ in 0..nodes_total {
            let node_id: String = Uuid::new_v4().to_string();
            node_ids.push(node_id.clone());
            nodes.push(Node::new(
                node_id,
                NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                HashMap::new()
            ));
        }

        for node_state_id in node_state_ids.iter() {
            let mut other_node_state_ids: Vec<String> = Vec::new();
            for other_node_state_id in node_state_ids.iter() {
                if node_state_id != other_node_state_id {
                    other_node_state_ids.push(other_node_state_id.clone());
                }
            }
            
            let node_state_collection_id: String = Uuid::new_v4().to_string();
            node_state_collection_ids.push(node_state_collection_id.clone());
            node_state_collections.push(NodeStateCollection::new(
                node_state_collection_id,
                node_state_id.clone(),
                other_node_state_ids
            ));
        }

        // tie nodes to their neighbors
        for (node_index, node) in std::iter::zip(0..nodes_total, nodes.iter_mut()) {
            let node_x: i32 = node_index % nodes_width;
            let node_y: i32 = (node_index / nodes_width) % nodes_height;
            let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
            //debug!("processing node {node_x}, {node_y}, {node_z}.");
            for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                let other_node_x: i32 = other_node_index % nodes_width;
                let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                    //debug!("found neighbor at {other_node_x}, {other_node_y}, {other_node_z}.");
                    node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                }
            }
        }

        let wave_function: WaveFunction<String>;

        wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        let collapsed_wave_function_result: Result<CollapsedWaveFunction<String>, String>;
        
        collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        // check that none of the neighbors match the same state
        for (node_index, node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
            let node_x: i32 = node_index % nodes_width;
            let node_y: i32 = (node_index / nodes_width) % nodes_height;
            let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
            for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                let other_node_x: i32 = other_node_index % nodes_width;
                let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                    assert_ne!(collapsed_wave_function.node_state_per_node.get(node_id), collapsed_wave_function.node_state_per_node.get(other_node_id));
                }
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn many_nodes_as_3D_grid_all_different_states_acc_seq() {
        init();

        let nodes_height = 3;
        let nodes_width = 3;
        let nodes_depth = 3;
        let nodes_total = nodes_height * nodes_width * nodes_depth;
        let node_states_total = 8;

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_ids: Vec<String> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_collection_ids: Vec<String> = Vec::new();

        for _ in 0..node_states_total {
            node_state_ids.push(Uuid::new_v4().to_string());
        }

        for _ in 0..nodes_total {
            let node_id: String = Uuid::new_v4().to_string();
            node_ids.push(node_id.clone());
            nodes.push(Node::new(
                node_id,
                NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                HashMap::new()
            ));
        }

        for node_state_id in node_state_ids.iter() {
            let mut other_node_state_ids: Vec<String> = Vec::new();
            for other_node_state_id in node_state_ids.iter() {
                if node_state_id != other_node_state_id {
                    other_node_state_ids.push(other_node_state_id.clone());
                }
            }
            
            let node_state_collection_id: String = Uuid::new_v4().to_string();
            node_state_collection_ids.push(node_state_collection_id.clone());
            node_state_collections.push(NodeStateCollection::new(
                node_state_collection_id,
                node_state_id.clone(),
                other_node_state_ids
            ));
        }

        // tie nodes to their neighbors
        for (node_index, node) in std::iter::zip(0..nodes_total, nodes.iter_mut()) {
            let node_x: i32 = node_index % nodes_width;
            let node_y: i32 = (node_index / nodes_width) % nodes_height;
            let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
            //debug!("processing node {node_x}, {node_y}, {node_z}.");
            for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                let other_node_x: i32 = other_node_index % nodes_width;
                let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                    //debug!("found neighbor at {other_node_x}, {other_node_y}, {other_node_z}.");
                    node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                }
            }
        }

        let wave_function: WaveFunction<String>;

        wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        let collapsed_wave_function_result: Result<CollapsedWaveFunction<String>, String>;
        
        collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        // check that none of the neighbors match the same state
        for (node_index, node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
            let node_x: i32 = node_index % nodes_width;
            let node_y: i32 = (node_index / nodes_width) % nodes_height;
            let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
            for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                let other_node_x: i32 = other_node_index % nodes_width;
                let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                    assert_ne!(collapsed_wave_function.node_state_per_node.get(node_id), collapsed_wave_function.node_state_per_node.get(other_node_id));
                }
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn many_nodes_as_3D_grid_randomly_all_different_states_getting_collapsed_function() {
        init();

        let random_seed = Some(15177947778026677010);
        //let random_seed = None;

        let max_runs = 1;

        for _index in 0..max_runs {

            //let random_seed = Some(rng.next_u64());

            let nodes_height = 4;
            let nodes_width = 4;
            let nodes_depth = 4;
            let nodes_total = nodes_height * nodes_width * nodes_depth;
            let node_states_total = 8;

            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_ids: Vec<String> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
            let mut node_state_ids: Vec<String> = Vec::new();
            let mut node_state_collection_ids: Vec<String> = Vec::new();

            for index in 0..node_states_total {
                let node_state_id: String = format!("{}_{}", index, Uuid::new_v4());
                node_state_ids.push(node_state_id);
            }

            for _ in 0..nodes_total {
                let node_id: String = Uuid::new_v4().to_string();
                node_ids.push(node_id.clone());
                nodes.push(Node::new(
                    node_id,
                    NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                    HashMap::new()
                ));
            }

            for node_state_id in node_state_ids.iter() {
                let mut other_node_state_ids: Vec<String> = Vec::new();
                for other_node_state_id in node_state_ids.iter() {
                    if node_state_id != other_node_state_id {
                        other_node_state_ids.push(other_node_state_id.clone());
                    }
                }
                
                let node_state_collection_id: String = Uuid::new_v4().to_string();
                node_state_collection_ids.push(node_state_collection_id.clone());
                node_state_collections.push(NodeStateCollection::new(
                    node_state_collection_id,
                    node_state_id.clone(),
                    other_node_state_ids
                ));
            }

            // tie nodes to their neighbors
            for (node_index, node) in std::iter::zip(0..nodes_total, nodes.iter_mut()) {
                let node_x: i32 = node_index % nodes_width;
                let node_y: i32 = (node_index / nodes_width) % nodes_height;
                let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
                //debug!("processing node {node_x}, {node_y}, {node_z}.");
                for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                    let other_node_x: i32 = other_node_index % nodes_width;
                    let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                    let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                    if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                        //debug!("found neighbor at {other_node_x}, {other_node_y}, {other_node_z}.");
                        node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                    }
                }
            }

            let wave_function: WaveFunction<String>;

            wave_function = WaveFunction::new(nodes, node_state_collections);

            wave_function.validate().unwrap();

            let collapsed_wave_function_result: Result<CollapsedWaveFunction<String>, String>;
            
            //let random_seed = Some(rng.gen::<u64>());  // TODO uncomment after fixing
            collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse();

            if let Err(error_message) = collapsed_wave_function_result {
                println!("tried random seed: {:?}.", random_seed);
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            for (node_index, node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                let node_x: i32 = node_index % nodes_width;
                let node_y: i32 = (node_index / nodes_width) % nodes_height;
                let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
                for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                    let other_node_x: i32 = other_node_index % nodes_width;
                    let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                    let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                    if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                        assert_ne!(collapsed_wave_function.node_state_per_node.get(node_id), collapsed_wave_function.node_state_per_node.get(other_node_id));
                    }
                }
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn many_nodes_as_3D_grid_randomly_all_different_states_uncollapsed_wave_functions() {
        init();

        for _ in 0..1 {

            //let random_seed = Some(rng.next_u64());
            //let random_seed = Some(3137775564618414013);
            let random_seed = Some(15177947778026677010);

            //let random_seed = None;

            let size = 4;

            let nodes_height = size;
            let nodes_width = size;
            let nodes_depth = size;
            let nodes_total = nodes_height * nodes_width * nodes_depth;
            let node_states_total = 8;

            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_ids: Vec<String> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();
            let mut node_state_ids: Vec<String> = Vec::new();
            let mut node_state_collection_ids: Vec<String> = Vec::new();

            for index in 0..node_states_total {
                let node_state_id: String = format!("{}{}", index, Uuid::new_v4());
                node_state_ids.push(node_state_id);
            }

            for index in 0..nodes_total {
                let node_id: String = format!("{}{}", index, Uuid::new_v4());
                node_ids.push(node_id.clone());
                nodes.push(Node::new(
                    node_id,
                    NodeStateProbability::get_equal_probability(node_state_ids.clone()),
                    HashMap::new()
                ));
            }

            for node_state_id in node_state_ids.iter() {
                let mut other_node_state_ids: Vec<String> = Vec::new();
                for other_node_state_id in node_state_ids.iter() {
                    if node_state_id != other_node_state_id {
                        other_node_state_ids.push(other_node_state_id.clone());
                    }
                }
                
                let node_state_collection_id: String = format!("{}{}", node_state_id, Uuid::new_v4().to_string());
                node_state_collection_ids.push(node_state_collection_id.clone());
                node_state_collections.push(NodeStateCollection::new(
                    node_state_collection_id,
                    node_state_id.clone(),
                    other_node_state_ids
                ));
            }

            // tie nodes to their neighbors
            for (node_index, node) in std::iter::zip(0..nodes_total, nodes.iter_mut()) {
                let node_x: i32 = node_index % nodes_width;
                let node_y: i32 = (node_index / nodes_width) % nodes_height;
                let node_z: i32 = (node_index / (nodes_width * nodes_height)) % nodes_depth;
                //debug!("processing node {node_x}, {node_y}, {node_z}.");
                for (other_node_index, other_node_id) in std::iter::zip(0..nodes_total, node_ids.iter()) {
                    let other_node_x: i32 = other_node_index % nodes_width;
                    let other_node_y: i32 = (other_node_index / nodes_width) % nodes_height;
                    let other_node_z: i32 = (other_node_index / (nodes_width * nodes_height)) % nodes_depth;
                    if node_index != other_node_index && (node_x - other_node_x).abs() <= 1 && (node_y - other_node_y).abs() <= 1 && (node_z - other_node_z).abs() <= 1 {
                        //debug!("found neighbor at {other_node_x}, {other_node_y}, {other_node_z}.");
                        node.node_state_collection_ids_per_neighbor_node_id.insert(other_node_id.clone(), node_state_collection_ids.clone());
                    }
                }
            }

            let wave_function: WaveFunction<String>;

            wave_function = WaveFunction::new(nodes, node_state_collections);

            wave_function.validate().unwrap();

            let collapsed_node_states_result: Result<Vec<CollapsedNodeState<String>>, String>;
            
            //let random_seed = Some(rng.gen::<u64>());  // TODO uncomment after fixing
            collapsed_node_states_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse_into_steps();

            if let Err(error_message) = collapsed_node_states_result {
                println!("tried random seed: {:?}.", random_seed);
                panic!("Error: {error_message}");
            }

            let node_states = collapsed_node_states_result.ok().unwrap();

            // TODO assert something about the uncollapsed wave functions
            //println!("States: {:?}", node_states);
            println!("Found {:?} node states.", node_states.len());
            println!("tried random seed: {:?}.", random_seed);
        }
    }

    #[test]
    fn write_and_read_wave_function_from_tempfile() {
        init();

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let node_state_id: String = Uuid::new_v4().to_string();

        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            Uuid::new_v4().to_string(),
            NodeStateProbability::get_equal_probability(vec![node_state_id.clone()]),
            HashMap::new()
        ));

        let first_node_id: String = nodes[0].id.clone();
        let second_node_id: String = nodes[1].id.clone();

        let same_node_state_collection_id: String = Uuid::new_v4().to_string();
        let same_node_state_collection = NodeStateCollection::new(
            same_node_state_collection_id.clone(),
            node_state_id.clone(),
            vec![node_state_id.clone()]
        );
        node_state_collections.push(same_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(second_node_id.clone(), Vec::new());
        nodes[0].node_state_collection_ids_per_neighbor_node_id.get_mut(&second_node_id).unwrap().push(same_node_state_collection_id.clone());

        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(first_node_id.clone(), Vec::new());
        nodes[1].node_state_collection_ids_per_neighbor_node_id.get_mut(&first_node_id).unwrap().push(same_node_state_collection_id.clone());

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let file = tempfile::NamedTempFile::new().unwrap();
        let file_path: &str = file.path().to_str().unwrap();
        debug!("Saving wave function to {:?}", file_path);
        wave_function.save_to_file(file_path);

        let loaded_wave_function: WaveFunction<String> = WaveFunction::load_from_file(file_path);
        loaded_wave_function.validate().unwrap();

        file.close().unwrap();

        let collapsed_wave_function = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse().unwrap();
        let loaded_collapsed_wave_function = loaded_wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse().unwrap();

        assert_eq!(collapsed_wave_function.node_state_per_node, loaded_collapsed_wave_function.node_state_per_node);
    }

    #[test]
    fn four_nodes_as_square_neighbors_randomly() {
        init();

        let mut random_instance = fastrand::Rng::new();

        for _ in 0..1000 {

            let random_seed = Some(random_instance.u64(..));

            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");

            nodes.push(Node::new(
                String::from("node_1"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_2"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_3"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_4"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));

            let one_forces_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let one_forces_two_node_state_collection = NodeStateCollection::new(
                one_forces_two_node_state_collection_id.clone(),
                one_node_state_id.clone(),
                vec![two_node_state_id.clone()]
            );
            node_state_collections.push(one_forces_two_node_state_collection);

            let two_forces_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let two_forces_one_node_state_collection = NodeStateCollection::new(
                two_forces_one_node_state_collection_id.clone(),
                two_node_state_id.clone(),
                vec![one_node_state_id.clone()]
            );
            node_state_collections.push(two_forces_one_node_state_collection);

            let possible_node_ids: Vec<&str> = vec!["node_1", "node_2", "node_3", "node_4"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                    if node_index != other_node_index && node_index % 2 != other_node_index % 2 {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                    }
                }
            }

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();

            let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse();

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
        }
    }

    #[test]
    fn four_nodes_as_square_neighbors_in_cycle_alone() {
        init();

        let mut random_instance = fastrand::Rng::new();

        for _ in 0..100 {

            let random_seed = Some(random_instance.u64(..));

            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");

            nodes.push(Node::new(
                String::from("node_1"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_2"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_3"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_4"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));

            let one_forces_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let one_forces_two_node_state_collection = NodeStateCollection::new(
                one_forces_two_node_state_collection_id.clone(),
                one_node_state_id.clone(),
                vec![two_node_state_id.clone()]
            );
            node_state_collections.push(one_forces_two_node_state_collection);

            let two_forces_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let two_forces_one_node_state_collection = NodeStateCollection::new(
                two_forces_one_node_state_collection_id.clone(),
                two_node_state_id.clone(),
                vec![one_node_state_id.clone()]
            );
            node_state_collections.push(two_forces_one_node_state_collection);

            let possible_node_ids: Vec<&str> = vec!["node_1", "node_2", "node_3", "node_4"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                    if (node_index + 1) % 4 == other_node_index {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                    }
                }
            }

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();

            let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse();

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3").unwrap());
        }
    }

    #[test]
    fn four_nodes_as_square_neighbors_in_cycle_affects_another_square_sequential() {
        init();

        let mut random_instance = fastrand::Rng::new();

        for _ in 0..100 {

            let random_seed = Some(random_instance.u64(..));

            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");

            let one_forces_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let one_forces_two_node_state_collection = NodeStateCollection::new(
                one_forces_two_node_state_collection_id.clone(),
                one_node_state_id.clone(),
                vec![two_node_state_id.clone()]
            );
            node_state_collections.push(one_forces_two_node_state_collection);

            let two_forces_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let two_forces_one_node_state_collection = NodeStateCollection::new(
                two_forces_one_node_state_collection_id.clone(),
                two_node_state_id.clone(),
                vec![one_node_state_id.clone()]
            );
            node_state_collections.push(two_forces_one_node_state_collection);

            nodes.push(Node::new(
                String::from("node_1a"),
                NodeStateProbability::get_equal_probability(vec![two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_2a"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_3a"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_4a"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));

            let possible_node_ids: Vec<&str> = vec!["node_1a", "node_2a", "node_3a", "node_4a"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                    if (node_index + 1) % 4 == other_node_index {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                    }
                }
            }

            nodes.push(Node::new(
                String::from("node_1b"),
                NodeStateProbability::get_equal_probability(vec![two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_2b"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_3b"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_4b"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));

            let possible_node_ids: Vec<&str> = vec!["node_1b", "node_2b", "node_3b", "node_4b"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                if node_index > 3 {
                    for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                        if (node_index + 1) % 4 == other_node_index {
                            node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                        }
                    }
                }
            }

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(String::from("node_1b"), vec![one_forces_two_node_state_collection_id]);

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();

            let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(random_seed).collapse();

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
        }
    }

    #[test]
    fn four_nodes_as_square_neighbors_in_cycle_affects_another_square_acc_seq() {
        init();

        let mut random_instance = fastrand::Rng::new();

        for _ in 0..100 {

            let random_seed = Some(random_instance.u64(..));

            let mut nodes: Vec<Node<String>> = Vec::new();
            let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

            let one_node_state_id: String = String::from("state_A");
            let two_node_state_id: String = String::from("state_B");

            let one_forces_two_node_state_collection_id: String = Uuid::new_v4().to_string();
            let one_forces_two_node_state_collection = NodeStateCollection::new(
                one_forces_two_node_state_collection_id.clone(),
                one_node_state_id.clone(),
                vec![two_node_state_id.clone()]
            );
            node_state_collections.push(one_forces_two_node_state_collection);

            let two_forces_one_node_state_collection_id: String = Uuid::new_v4().to_string();
            let two_forces_one_node_state_collection = NodeStateCollection::new(
                two_forces_one_node_state_collection_id.clone(),
                two_node_state_id.clone(),
                vec![one_node_state_id.clone()]
            );
            node_state_collections.push(two_forces_one_node_state_collection);

            nodes.push(Node::new(
                String::from("node_1a"),
                NodeStateProbability::get_equal_probability(vec![two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_2a"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_3a"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_4a"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));

            let possible_node_ids: Vec<&str> = vec!["node_1a", "node_2a", "node_3a", "node_4a"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                    if (node_index + 1) % 4 == other_node_index {
                        node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                    }
                }
            }

            nodes.push(Node::new(
                String::from("node_1b"),
                NodeStateProbability::get_equal_probability(vec![two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_2b"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_3b"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));
            nodes.push(Node::new(
                String::from("node_4b"),
                NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
                HashMap::new()
            ));

            let possible_node_ids: Vec<&str> = vec!["node_1b", "node_2b", "node_3b", "node_4b"];
            for (node_index, node) in nodes.iter_mut().enumerate() {
                if node_index > 3 {
                    for (other_node_index, other_node_id) in possible_node_ids.iter().enumerate() {
                        if (node_index + 1) % 4 == other_node_index {
                            node.node_state_collection_ids_per_neighbor_node_id.insert(String::from(*other_node_id), vec![one_forces_two_node_state_collection_id.clone(), two_forces_one_node_state_collection_id.clone()]);
                        }
                    }
                }
            }

            nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(String::from("node_1b"), vec![one_forces_two_node_state_collection_id]);

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            wave_function.validate().unwrap();

            let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<AccommodatingSequentialCollapsableWaveFunction<String>>(random_seed).collapse();

            if let Err(error_message) = collapsed_wave_function_result {
                panic!("Error: {error_message}");
            }

            let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2a").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3a").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1a").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_1b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_2b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_3b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_4b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_1b").unwrap());
            assert_eq!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_2b").unwrap());
            assert_ne!(collapsed_wave_function.node_state_per_node.get("node_4b").unwrap(), collapsed_wave_function.node_state_per_node.get("node_3b").unwrap());
        }
    }

    #[test]
    fn four_nodes_that_would_skip_over_nonneighbor() {
        init();

        // TODO add randomization

        let mut nodes: Vec<Node<String>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<String>> = Vec::new();

        let one_node_id: String = String::from("node_1");
        let two_node_id: String = String::from("node_2");
        let three_node_id: String = String::from("node_3");
        let four_node_id: String = String::from("node_4");
        
        let one_node_state_id: String = String::from("state_A");
        let two_node_state_id: String = String::from("state_B");

        nodes.push(Node::new(
            one_node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            two_node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            three_node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));
        nodes.push(Node::new(
            four_node_id.clone(),
            NodeStateProbability::get_equal_probability(vec![one_node_state_id.clone(), two_node_state_id.clone()]),
            HashMap::new()
        ));

        let one_node_state_id: String = String::from("state_A");
        let two_node_state_id: String = String::from("state_B");

        let one_permits_one_and_two_node_state_collection_id: String = Uuid::new_v4().to_string();
        let one_permits_one_and_two_node_state_collection = NodeStateCollection::new(
            one_permits_one_and_two_node_state_collection_id.clone(),
            one_node_state_id.clone(),
            vec![one_node_state_id.clone(), two_node_state_id.clone()]
        );
        node_state_collections.push(one_permits_one_and_two_node_state_collection);

        let two_permits_none_node_state_collection_id: String = Uuid::new_v4().to_string();
        let two_permits_none_node_state_collection = NodeStateCollection::new(
            two_permits_none_node_state_collection_id.clone(),
            two_node_state_id.clone(),
            vec![]
        );
        node_state_collections.push(two_permits_none_node_state_collection);

        let two_permits_one_node_state_collection_id: String = Uuid::new_v4().to_string();
        let two_permits_one_node_state_collection = NodeStateCollection::new(
            two_permits_one_node_state_collection_id.clone(),
            two_node_state_id.clone(),
            vec![one_node_state_id.clone()]
        );
        node_state_collections.push(two_permits_one_node_state_collection);

        let one_permits_two_node_state_collection_id: String = Uuid::new_v4().to_string();
        let one_permits_two_node_state_collection = NodeStateCollection::new(
            one_permits_two_node_state_collection_id.clone(),
            one_node_state_id.clone(),
            vec![two_node_state_id.clone()]
        );
        node_state_collections.push(one_permits_two_node_state_collection);

        let one_permits_one_node_state_collection_id: String = Uuid::new_v4().to_string();
        let one_permits_one_node_state_collection = NodeStateCollection::new(
            one_permits_one_node_state_collection_id.clone(),
            one_node_state_id.clone(),
            vec![one_node_state_id.clone()]
        );
        node_state_collections.push(one_permits_one_node_state_collection);

        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(two_node_id.clone(), vec![one_permits_one_and_two_node_state_collection_id.clone(), two_permits_none_node_state_collection_id.clone()]);
        nodes[0].node_state_collection_ids_per_neighbor_node_id.insert(three_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(one_node_id.clone(), vec![one_permits_one_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[1].node_state_collection_ids_per_neighbor_node_id.insert(four_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(one_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[2].node_state_collection_ids_per_neighbor_node_id.insert(four_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[3].node_state_collection_ids_per_neighbor_node_id.insert(two_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);
        nodes[3].node_state_collection_ids_per_neighbor_node_id.insert(three_node_id.clone(), vec![one_permits_two_node_state_collection_id.clone(), two_permits_one_node_state_collection_id.clone()]);

        let wave_function = WaveFunction::new(nodes, node_state_collections);
        wave_function.validate().unwrap();

        let collapsed_wave_function_result = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<String>>(None).collapse();

        if let Err(error_message) = collapsed_wave_function_result {
            panic!("Error: {error_message}");
        }

        let collapsed_wave_function = collapsed_wave_function_result.ok().unwrap();

        assert_eq!(&one_node_state_id, collapsed_wave_function.node_state_per_node.get(&one_node_id).unwrap());
        assert_eq!(&two_node_state_id, collapsed_wave_function.node_state_per_node.get(&two_node_id).unwrap());
        assert_eq!(&two_node_state_id, collapsed_wave_function.node_state_per_node.get(&three_node_id).unwrap());
        assert_eq!(&one_node_state_id, collapsed_wave_function.node_state_per_node.get(&four_node_id).unwrap());
    }
}

#[cfg(test)]
mod indexed_view_unit_tests {

    use uuid::Uuid;
    use crate::wave_function::indexed_view::IndexedView;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[test]
    fn initialize() {
        init();

        let node_state_ids: Vec<u32> = Vec::new();
        let node_state_probabilities: Vec<f32> = Vec::new();
        let _indexed_view = IndexedView::new(node_state_ids, node_state_probabilities);
        
        debug!("Succeeded to initialize IndexedView instance.");
    }

    #[test]
    fn one_item() {
        init();

        let mut node_state_ids: Vec<u32> = Vec::new();
        let mut node_state_probabilities: Vec<f32> = Vec::new();
        let original_node_state_id: u32 = 1;
        node_state_ids.push(original_node_state_id);
        node_state_probabilities.push(1.0);

        let mut indexed_view = IndexedView::new(node_state_ids, node_state_probabilities);

        assert!(indexed_view.try_move_next());
        let found_node_state_id = indexed_view.get().unwrap();

        assert_eq!(&original_node_state_id, found_node_state_id);
        assert!(!indexed_view.try_move_next());
    }

    #[test]
    fn two_items() {
        init();

        let mut node_state_ids: Vec<u32> = Vec::new();
        let mut node_state_probabilities: Vec<f32> = Vec::new();
        let one_original_node_state_id: u32 = 1;
        let two_original_node_state_id: u32 = 2;
        node_state_ids.push(one_original_node_state_id);
        node_state_ids.push(two_original_node_state_id);
        node_state_probabilities.push(1.0);
        node_state_probabilities.push(1.0);

        let mut indexed_view = IndexedView::new(node_state_ids, node_state_probabilities);
        assert!(indexed_view.try_move_next());
        let first_found_node_state_id = *indexed_view.get().unwrap();
        assert!(indexed_view.try_move_next());
        let second_found_node_state_id = *indexed_view.get().unwrap();

        assert_ne!(first_found_node_state_id, second_found_node_state_id);
        assert!(!indexed_view.try_move_next());
    }

    #[test]
    fn many_items_sequential_order() {
        init();

        let mut node_state_ids: Vec<u32> = Vec::new();
        let mut node_state_probabilities: Vec<f32> = Vec::new();
        let number_of_items: u32 = 10000;
        for node_state_id in 0..number_of_items {
            node_state_ids.push(node_state_id);
            node_state_probabilities.push(1.0);
        }

        let mut indexed_view = IndexedView::new(node_state_ids, node_state_probabilities);
        let mut popped_node_state_ids: Vec<u32> = Vec::new();
        for _ in 0..number_of_items {
            assert!(indexed_view.try_move_next());
            let node_state_id = *indexed_view.get().unwrap();
            assert!(!popped_node_state_ids.contains(&node_state_id));
            popped_node_state_ids.push(node_state_id);
        }
        assert!(!indexed_view.try_move_next());
    }

    #[test]
    fn many_items_reverse_order() {
        init();

        let mut node_state_ids: Vec<u32> = Vec::new();
        let mut node_state_probabilities: Vec<f32> = Vec::new();
        let number_of_items: u32 = 10000;
        for node_state_id in (0..number_of_items).rev() {
            node_state_ids.push(node_state_id);
            node_state_probabilities.push(1.0);
        }

        let mut indexed_view = IndexedView::new(node_state_ids, node_state_probabilities);
        let mut popped_node_state_ids: Vec<u32> = Vec::new();
        for _ in 0..number_of_items {
            assert!(indexed_view.try_move_next());
            let node_state_id = *indexed_view.get().unwrap();
            assert!(!popped_node_state_ids.contains(&node_state_id));
            popped_node_state_ids.push(node_state_id);
        }
        assert!(!indexed_view.try_move_next());
    }

    #[test]
    fn many_items_random_order_of_u32() {
        init();

        let number_of_items: u32 = 10000;
        let mut random_instance = fastrand::Rng::new();

        let mut node_state_ids: Vec<u32> = Vec::new();
        let mut node_state_probabilities: Vec<f32> = Vec::new();
        for node_state_id in 0..number_of_items {
            node_state_ids.push(node_state_id);
            node_state_probabilities.push(1.0);
        }
        random_instance.shuffle(node_state_ids.as_mut_slice());

        let mut indexed_view = IndexedView::new(node_state_ids, node_state_probabilities);
        let mut popped_node_state_ids: Vec<u32> = Vec::new();
        for _ in 0..number_of_items {
            assert!(indexed_view.try_move_next());
            let node_state_id = *indexed_view.get().unwrap();
            assert!(!popped_node_state_ids.contains(&node_state_id));
            popped_node_state_ids.push(node_state_id);
        }
        assert!(!indexed_view.try_move_next());
    }

    #[test]
    fn many_items_random_order_of_uuid_unshuffled() {
        init();

        let number_of_items: u32 = 10000;
        let mut random_instance = fastrand::Rng::new();

        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_probabilities: Vec<f32> = Vec::new();
        for _ in 0..number_of_items {
            node_state_ids.push(Uuid::new_v4().to_string());
            node_state_probabilities.push(1.0);
        }
        random_instance.shuffle(node_state_ids.as_mut_slice());

        let mut indexed_view = IndexedView::new(node_state_ids, node_state_probabilities);
        let mut popped_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..number_of_items {
            assert!(indexed_view.try_move_next());
            let node_state_id = indexed_view.get().unwrap();
            assert!(!popped_node_state_ids.contains(node_state_id));
            popped_node_state_ids.push(node_state_id.clone());
        }
        assert!(!indexed_view.try_move_next());
    }

    #[test]
    fn many_items_random_order_of_uuid_shuffled() {
        init();

        let number_of_items: u32 = 10000;
        let mut random_instance = fastrand::Rng::new();

        let mut node_state_ids: Vec<String> = Vec::new();
        let mut node_state_probabilities: Vec<f32> = Vec::new();
        for _ in 0..number_of_items {
            node_state_ids.push(Uuid::new_v4().to_string());
            node_state_probabilities.push(1.0);
        }
        random_instance.shuffle(node_state_ids.as_mut_slice());

        let mut indexed_view = IndexedView::new(node_state_ids, node_state_probabilities);
        
        indexed_view.shuffle(&mut random_instance);

        let mut popped_node_state_ids: Vec<String> = Vec::new();
        for _ in 0..number_of_items {
            assert!(indexed_view.try_move_next());
            let node_state_id = indexed_view.get().unwrap();
            assert!(!popped_node_state_ids.contains(node_state_id));
            popped_node_state_ids.push(node_state_id.clone());
        }
        assert!(!indexed_view.try_move_next());
    }
}