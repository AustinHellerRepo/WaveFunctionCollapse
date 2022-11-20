use std::{slice::Iter, collections::HashMap};
use colored::{Colorize, ColoredString};
use log::debug;
extern crate pretty_env_logger;

use rand::Rng;
use uuid::Uuid;
use wave_function_collapse::wave_function::{
    Node,
    NodeStateCollection,
    WaveFunction,
    CollapsedWaveFunction, NodeStateProbability
};

#[derive(Debug)]
enum LandscapeElement {
    Water,
    Sand,
    Grass,
    Tree,
    Forest,
    Hill,
    Mountain
}

impl LandscapeElement {
    fn iter() -> Iter<'static, LandscapeElement> {
        [LandscapeElement::Water, LandscapeElement::Sand, LandscapeElement::Grass, LandscapeElement::Tree, LandscapeElement::Forest, LandscapeElement::Hill, LandscapeElement::Mountain].iter()
    }
    fn into_iter() -> std::array::IntoIter<LandscapeElement, 7> {
        [LandscapeElement::Water, LandscapeElement::Sand, LandscapeElement::Grass, LandscapeElement::Tree, LandscapeElement::Forest, LandscapeElement::Hill, LandscapeElement::Mountain].into_iter()
    }
    fn get_node_state_ids() -> Vec<String> {
        let mut node_state_ids: Vec<String> = Vec::new();
        for landscape_element in LandscapeElement::iter() {
            node_state_ids.push(landscape_element.to_string());
        }
        node_state_ids
    }
    fn get_colored_text_by_node_state_id(node_state_id: &str) -> ColoredString {
        let character = "\u{2588}";
        if String::from(node_state_id) == LandscapeElement::Water.to_string() {
            character.blue()
        }
        else if String::from(node_state_id) == LandscapeElement::Sand.to_string() {
            character.yellow()
        }
        else if String::from(node_state_id) == LandscapeElement::Grass.to_string() {
            character.bright_green()
        }
        else if String::from(node_state_id) == LandscapeElement::Tree.to_string() {
            character.green()
        }
        else if String::from(node_state_id) == LandscapeElement::Forest.to_string() {
            character.bright_purple()
        }
        else if String::from(node_state_id) == LandscapeElement::Hill.to_string() {
            character.bright_black()
        }
        else if String::from(node_state_id) == LandscapeElement::Mountain.to_string() {
            character.white()
        }
        else {
            panic!("Unexpected node state: {node_state_id}.");
        }
    }
}

impl std::fmt::Display for LandscapeElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Landscape {
    width: u32,
    height: u32
}

impl Landscape {
    fn new(width: u32, height: u32) -> Self {
        Landscape {
            width: width,
            height: height
        }
    }
    fn get_wave_function(&self) -> WaveFunction {

        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();
        // water
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Water.to_string(),
            NodeStateProbability::new_equal_probabilities(vec![LandscapeElement::Water.to_string(), LandscapeElement::Sand.to_string()])
        ));
        // sand
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Sand.to_string(),
            NodeStateProbability::new_equal_probabilities(vec![LandscapeElement::Water.to_string(), LandscapeElement::Sand.to_string(), LandscapeElement::Grass.to_string()])
        ));
        // grass
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Grass.to_string(),
            NodeStateProbability::new_equal_probabilities(vec![LandscapeElement::Sand.to_string(), LandscapeElement::Grass.to_string(), LandscapeElement::Tree.to_string(), LandscapeElement::Hill.to_string()])
        ));
        // tree
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Tree.to_string(),
            NodeStateProbability::new_equal_probabilities(vec![LandscapeElement::Grass.to_string(), LandscapeElement::Tree.to_string(), LandscapeElement::Forest.to_string()])
        ));
        // forest
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Forest.to_string(),
            NodeStateProbability::new_equal_probabilities(vec![LandscapeElement::Tree.to_string(), LandscapeElement::Forest.to_string()])
        ));
        // hill
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Hill.to_string(),
            NodeStateProbability::new_equal_probabilities(vec![LandscapeElement::Grass.to_string(), LandscapeElement::Hill.to_string(), LandscapeElement::Mountain.to_string()])
        ));
        // mountain
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Mountain.to_string(),
            NodeStateProbability::new_equal_probabilities(vec![LandscapeElement::Hill.to_string(), LandscapeElement::Mountain.to_string()])
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
                let node_id = format!("{}_{}", width_index, height_index);
                node_id_per_x.insert(width_index, node_id);
            }
            node_id_per_x_per_y.insert(height_index, node_id_per_x);
        }

        debug!("connecting nodes");
        let mut nodes: Vec<Node> = Vec::new();
        for from_height_index in 0..self.height {
            for from_width_index in (0..self.width) {
                debug!("setup ({from_width_index}, {from_height_index})");
                let from_node_id: String = node_id_per_x_per_y.get(&from_height_index).unwrap().get(&from_width_index).unwrap().clone();
                let min_to_height_index: u32;
                if from_height_index == 0 {
                    min_to_height_index = 0;
                }
                else {
                    min_to_height_index = from_height_index - 1;
                }
                let max_to_height_index: u32;
                if from_height_index == self.height - 1 {
                    max_to_height_index = self.height - 1;
                }
                else {
                    max_to_height_index = from_height_index + 1;
                }
                let min_to_width_index: u32;
                if from_width_index == 0 {
                    min_to_width_index = 0;
                }
                else {
                    min_to_width_index = from_width_index - 1;
                }
                let max_to_width_index: u32;
                if from_width_index == self.width - 1 {
                    max_to_width_index = self.width - 1;
                }
                else {
                    max_to_width_index = from_width_index + 1;
                }
                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                for to_height_index in min_to_height_index..=max_to_height_index {
                    for to_width_index in min_to_width_index..=max_to_width_index {
                        if !(from_height_index == to_height_index && from_width_index == to_width_index) {
                            debug!("connecting ({from_width_index}, {from_height_index}) to ({to_width_index}, {to_height_index})");
                            let to_node_id: String = node_id_per_x_per_y.get(&to_height_index).unwrap().get(&to_width_index).unwrap().clone();
                            node_state_collection_ids_per_neighbor_node_id.insert(to_node_id, node_state_collection_ids.clone());
                        }
                    }
                }
                let node = Node {
                    id: from_node_id,
                    node_state_ids: LandscapeElement::get_node_state_ids(),
                    node_state_collection_ids_per_neighbor_node_id: node_state_collection_ids_per_neighbor_node_id
                };
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
    let landscape = Landscape::new(width, height);

    let mut wave_function = landscape.get_wave_function();

    wave_function.validate().unwrap();
    //wave_function.sort();

    let mut rng = rand::thread_rng();
    let random_seed = Some(rng.gen::<u64>());

    let collapsed_wave_function = wave_function.collapse(random_seed).unwrap();

    let mut node_state_per_y_per_x: Vec<Vec<Option<String>>> = Vec::new();
    for _ in 0..width {
        let mut node_state_per_y: Vec<Option<String>> = Vec::new();
        for _ in 0..height {
            node_state_per_y.push(None);
        }
        node_state_per_y_per_x.push(node_state_per_y);
    }

    for (node, node_state) in collapsed_wave_function.node_state_per_node.into_iter() {
        let node_split = node.split("_").collect::<Vec<&str>>();
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
            let colored_text = LandscapeElement::get_colored_text_by_node_state_id(node_state_id);
            print!("{}{}", colored_text, colored_text);
        }
        println!("|");
    }
    print!("-");
    for _ in 0..width {
        print!("--");
    }
    println!("-");
}