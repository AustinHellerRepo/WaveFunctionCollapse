#![allow(clippy::all)]
use std::{collections::HashMap, time::Instant};
use colored::{ColoredString, Colorize};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{WaveFunction, NodeStateCollection, Node, collapsable_wave_function::{accommodating_collapsable_wave_function::AccommodatingCollapsableWaveFunction, collapsable_wave_function::CollapsableWaveFunction}};
use log::debug;
extern crate pretty_env_logger;

struct Sparse {
    width: u32,
    height: u32,
    distance: u32
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, PartialOrd, Ord, Serialize, Deserialize)]
enum SparseElement {
    Empty,
    Active
}

impl SparseElement {
    fn get_colored_text_by_node_state_id(node_state_id: &SparseElement) -> ColoredString {
        let character = "\u{2588}";
        if node_state_id == &SparseElement::Active {
            character.white()
        }
        else if node_state_id == &SparseElement::Empty {
            character.black()
        }
        else {
            panic!("Unexpected node state: {node_state_id}.");
        }
    }
}

impl std::fmt::Display for SparseElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Sparse {
    fn new(width: u32, height: u32, distance: u32) -> Self {
        Sparse {
            width,
            height,
            distance
        }
    }
    fn get_wave_function(&self) -> WaveFunction<SparseElement> {

        let mut node_state_collections: Vec<NodeStateCollection<SparseElement>> = Vec::new();

        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            SparseElement::Active,
            vec![SparseElement::Empty]
        ));

        let mut node_state_collection_ids: Vec<String> = Vec::new();
        for node_state_collection in node_state_collections.iter() {
            let node_state_collection_id: String = node_state_collection.id.clone();
            node_state_collection_ids.push(node_state_collection_id);
        }

        let mut node_id_per_x_per_y: HashMap<u32, HashMap<u32, String>> = HashMap::new();
        for height_index in 0..self.height {
            let mut node_id_per_x: HashMap<u32, String> = HashMap::new();
            for width_index in 0..self.width {
                let node_id = format!("{width_index}_{height_index}");
                node_id_per_x.insert(width_index, node_id);
            }
            node_id_per_x_per_y.insert(height_index, node_id_per_x);
        }

        debug!("connecting nodes");
        let mut nodes: Vec<Node<SparseElement>> = Vec::new();
        for from_height_index in 0..self.height {
            for from_width_index in 0..self.width {
                debug!("setup ({from_width_index}, {from_height_index})");
                let from_node_id: String = node_id_per_x_per_y.get(&from_height_index).unwrap().get(&from_width_index).unwrap().clone();

                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();

                // fully connected set of 8-to-1
                for to_height_index in 0..self.height {
                    for to_width_index in 0..self.width {
                        if !(from_height_index == to_height_index && from_width_index == to_width_index) &&
                            (from_height_index.abs_diff(to_height_index) + from_width_index.abs_diff(to_width_index) <= self.distance) {

                            debug!("connecting ({from_width_index}, {from_height_index}) to ({to_width_index}, {to_height_index})");
                            let to_node_id: String = node_id_per_x_per_y.get(&to_height_index).unwrap().get(&to_width_index).unwrap().clone();
                            node_state_collection_ids_per_neighbor_node_id.insert(to_node_id, node_state_collection_ids.clone());
                        }
                    }
                }

                let mut node_state_probability_per_node_state_id: HashMap<SparseElement, f32> = HashMap::new();
                node_state_probability_per_node_state_id.insert(SparseElement::Active, 1.0);
                node_state_probability_per_node_state_id.insert(SparseElement::Empty, 100.0);

                let node = Node::new(
                    from_node_id,
                    node_state_probability_per_node_state_id,
                    node_state_collection_ids_per_neighbor_node_id
                );
                nodes.push(node);
            }
        }
        
        WaveFunction::new(nodes, node_state_collections)
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    //pretty_env_logger::init();

    let width: u32 = 60;
    let height: u32 = 60;
    let distance: u32 = 5;
    let landscape = Sparse::new(width, height, distance);

    let wave_function = landscape.get_wave_function();

    wave_function.validate().unwrap();

    let mut rng = rand::thread_rng();
    let random_seed = Some(rng.gen::<u64>());

    let start = Instant::now();

    let collapsed_wave_function = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<SparseElement>>(random_seed).collapse().unwrap();

    let mut node_state_per_y_per_x: Vec<Vec<Option<SparseElement>>> = Vec::new();
    for _ in 0..width {
        let mut node_state_per_y: Vec<Option<SparseElement>> = Vec::new();
        for _ in 0..height {
            node_state_per_y.push(None);
        }
        node_state_per_y_per_x.push(node_state_per_y);
    }

    for (node, node_state) in collapsed_wave_function.node_state_per_node.into_iter() {
        let node_split = node.split('_').collect::<Vec<&str>>();
        let x = node_split[0].parse::<u32>().unwrap() as usize;
        let y = node_split[1].parse::<u32>().unwrap() as usize;
        node_state_per_y_per_x[x][y] = Some(node_state);
    }

    print!("-");
    for _ in 0..width {
        print!("--");
    }
    println!("-");
    for y in 0..height as usize {
        print!("|");
        for x in 0..width as usize {
            let node_state_id = node_state_per_y_per_x[x][y].as_ref().unwrap();
            let colored_text = SparseElement::get_colored_text_by_node_state_id(node_state_id);
            print!("{colored_text}{colored_text}");
        }
        println!("|");
    }
    print!("-");
    for _ in 0..width {
        print!("--");
    }
    println!("-");

    let duration = start.elapsed();
    println!("Duration: {duration:?}");
}