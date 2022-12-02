use std::{slice::Iter, collections::HashMap, time::Instant};
use colored::{Colorize, ColoredString};
use log::debug;
extern crate pretty_env_logger;
use rand::Rng;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{
    Node,
    NodeStateCollection,
    WaveFunction,
    collapsable_wave_function::{collapsable_wave_function::CollapsableWaveFunction, accommodating_collapsable_wave_function::AccommodatingCollapsableWaveFunction, spreading_collapsable_wave_function::SpreadingCollapsableWaveFunction}
};

/// This enum represents the possible states of a node in the 2D world
#[derive(Debug, Eq, Hash, PartialEq, Clone, PartialOrd, Ord, Serialize, Deserialize)]
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
    fn get_colored_text_by_node_state_id(node_state_id: &LandscapeElement) -> ColoredString {
        let character = "\u{2588}";
        if node_state_id == &LandscapeElement::Water {
            character.blue()
        }
        else if node_state_id == &LandscapeElement::Sand {
            character.yellow()
        }
        else if node_state_id == &LandscapeElement::Grass {
            character.bright_green()
        }
        else if node_state_id == &LandscapeElement::Tree {
            character.green()
        }
        else if node_state_id == &LandscapeElement::Forest {
            character.bright_purple()
        }
        else if node_state_id == &LandscapeElement::Hill {
            character.bright_black()
        }
        else if node_state_id == &LandscapeElement::Mountain {
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

/// This struct represents a 2D landscape of possible elements.
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
    fn get_wave_function(&self) -> WaveFunction<LandscapeElement> {

        let mut node_state_collections: Vec<NodeStateCollection<LandscapeElement>> = Vec::new();
        // water
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Water,
            vec![LandscapeElement::Water, LandscapeElement::Sand]
        ));
        // sand
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Sand,
            vec![LandscapeElement::Water, LandscapeElement::Sand, LandscapeElement::Grass]
        ));
        // grass
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Grass,
            vec![LandscapeElement::Sand, LandscapeElement::Grass, LandscapeElement::Tree, LandscapeElement::Hill]
        ));
        // tree
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Tree,
            vec![LandscapeElement::Grass, LandscapeElement::Tree, LandscapeElement::Forest]
        ));
        // forest
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Forest,
            vec![LandscapeElement::Tree, LandscapeElement::Forest]
        ));
        // hill
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Hill,
            vec![LandscapeElement::Grass, LandscapeElement::Hill, LandscapeElement::Mountain]
        ));
        // mountain
        node_state_collections.push(NodeStateCollection::new(
            Uuid::new_v4().to_string(),
            LandscapeElement::Mountain,
            vec![LandscapeElement::Hill, LandscapeElement::Mountain]
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
        let mut nodes: Vec<Node<LandscapeElement>> = Vec::new();
        for from_height_index in 0..self.height {
            for from_width_index in 0..self.width {
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

                if true {
                    // fully connected set of 8-to-1
                    for to_height_index in min_to_height_index..=max_to_height_index {
                        for to_width_index in min_to_width_index..=max_to_width_index {
                            if !(from_height_index == to_height_index && from_width_index == to_width_index) {
                                debug!("connecting ({from_width_index}, {from_height_index}) to ({to_width_index}, {to_height_index})");
                                let to_node_id: String = node_id_per_x_per_y.get(&to_height_index).unwrap().get(&to_width_index).unwrap().clone();
                                node_state_collection_ids_per_neighbor_node_id.insert(to_node_id, node_state_collection_ids.clone());
                            }
                        }
                    }
                }
                else {
                    for to_height_index in min_to_height_index..=max_to_height_index {
                        let to_width_index = from_width_index;
                        if !(from_height_index == to_height_index) {
                            debug!("connecting ({from_width_index}, {from_height_index}) to ({to_width_index}, {to_height_index})");
                            let to_node_id: String = node_id_per_x_per_y.get(&to_height_index).unwrap().get(&to_width_index).unwrap().clone();
                            node_state_collection_ids_per_neighbor_node_id.insert(to_node_id, node_state_collection_ids.clone());
                        }
                    }
                    for to_width_index in min_to_width_index..=max_to_width_index {
                        let to_height_index = from_height_index;
                        if !(from_width_index == to_width_index) {
                            debug!("connecting ({from_width_index}, {from_height_index}) to ({to_width_index}, {to_height_index})");
                            let to_node_id: String = node_id_per_x_per_y.get(&to_height_index).unwrap().get(&to_width_index).unwrap().clone();
                            node_state_collection_ids_per_neighbor_node_id.insert(to_node_id, node_state_collection_ids.clone());
                        }
                    }
                }

                let mut node_state_probability_per_node_state_id: HashMap<LandscapeElement, f32> = HashMap::new();
                node_state_probability_per_node_state_id.insert(LandscapeElement::Water, 1.0);
                node_state_probability_per_node_state_id.insert(LandscapeElement::Sand, 0.1);
                node_state_probability_per_node_state_id.insert(LandscapeElement::Grass, 1.0);
                node_state_probability_per_node_state_id.insert(LandscapeElement::Hill, 0.1);
                node_state_probability_per_node_state_id.insert(LandscapeElement::Mountain, 1.0);
                node_state_probability_per_node_state_id.insert(LandscapeElement::Tree, 0.1);
                node_state_probability_per_node_state_id.insert(LandscapeElement::Forest, 1.0);

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

    let start = Instant::now();

    let width: u32 = 6;
    let height: u32 = 6;
    let landscape = Landscape::new(width, height);

    let wave_function = landscape.get_wave_function();

    wave_function.validate().unwrap();

    let mut rng = rand::thread_rng();
    let random_seed = Some(rng.gen::<u64>());

    let collapsed_wave_function = wave_function.get_collapsable_wave_function::<SpreadingCollapsableWaveFunction<LandscapeElement>>(random_seed).collapse().unwrap();

    let mut node_state_per_y_per_x: Vec<Vec<Option<LandscapeElement>>> = Vec::new();
    for _ in 0..width {
        let mut node_state_per_y: Vec<Option<LandscapeElement>> = Vec::new();
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

    let duration = start.elapsed();
    println!("Duration: {:?}", duration);
}