use std::{collections::{HashSet, HashMap}, io::Write, time::{Instant, Duration}};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{WaveFunction, NodeStateCollection, Node, collapsable_wave_function::{collapsable_wave_function::{CollapsableWaveFunction, CollapsedWaveFunction, CollapsedNodeState}, entropic_collapsable_wave_function::EntropicCollapsableWaveFunction}};
use image::{io::Reader as ImageReader, GenericImageView, DynamicImage, ImageFormat};
use colored::Colorize;
use std::cmp;

fn print_pixel(color: &[u8; 4]) {
    let character = "\u{2588}";
    print!("{}{}", character.truecolor(color[0], color[1], color[2]), character.truecolor(color[0], color[1], color[2]));
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
struct ImageFragment {
    // the RGBA color per height per width
    pixels: Vec<Vec<[u8; 4]>>,
    width: u32,
    height: u32
}

impl ImageFragment {
    fn new_from_image(image: &DynamicImage, width_index: u32, height_index: u32, fragment_width: u32, fragment_height: u32) -> ImageFragment {
        let mut pixels: Vec<Vec<[u8; 4]>> = Vec::new();
        for index in 0..fragment_width {
            pixels.push(Vec::new());
            for _ in 0..fragment_height {
                pixels[index as usize].push([0, 0, 0, 0]);
            }
        }
        for pixel_height_offset in 0..fragment_height {
            for pixel_width_offset in 0..fragment_width {
                let pixel_width_index = width_index + pixel_width_offset;
                let pixel_height_index = height_index + pixel_height_offset;
                let pixel = image.get_pixel(pixel_width_index, pixel_height_index);
                pixels[pixel_width_offset as usize][pixel_height_offset as usize] = pixel.0;
            }
        }
        ImageFragment {
            pixels,
            width: fragment_width,
            height: fragment_height
        }
    }
    fn is_overlapping(&self, other_image_fragment: &ImageFragment, width_offset: i8, height_offset: i8) -> bool {
        let mut is_at_least_one_pixel_nonoverlapping: bool = false;
        for self_height_index in cmp::max(0, height_offset)..cmp::min(self.height as i8, height_offset + self.height as i8) {
            let other_height_index = self_height_index - height_offset;
            for self_width_index in cmp::max(0, width_offset)..cmp::min(self.width as i8, width_offset + self.width as i8) {
                let other_width_index = self_width_index - width_offset;
                let other_pixel = other_image_fragment.pixels[other_width_index as usize][other_height_index as usize];
                let self_pixel = self.pixels[self_width_index as usize][self_height_index as usize];
                if other_pixel != self_pixel {
                    is_at_least_one_pixel_nonoverlapping = true;
                    break;
                }
            }
        }
        !is_at_least_one_pixel_nonoverlapping
    }
    #[allow(dead_code)]
    fn print(&self) {
        for height_index in 0..self.height as usize {
            for width_index in 0..self.width as usize {
                let color = self.pixels[width_index][height_index];
                print_pixel(&color);
            }
            println!();
        }
    }
    fn rotate(&self) -> Self {
        let mut pixels: Vec<Vec<[u8; 4]>> = Vec::new();
        for index in 0..self.height {
            pixels.push(Vec::new());
            for _ in 0..self.width {
                pixels[index as usize].push([0, 0, 0, 0]);
            }
        }
        for height_index in 0..self.height {
            for width_index in 0..self.width {
                let source_color = self.pixels[width_index as usize][height_index as usize];
                let destination_width_index = (self.height - 1) - height_index;
                let destination_height_index = width_index;
                pixels[destination_width_index as usize][destination_height_index as usize] = source_color;
            }
        }
        ImageFragment {
            pixels: pixels,
            width: self.height,
            height: self.width
        }
    }
    fn flip(&self) -> Self {
        let mut pixels: Vec<Vec<[u8; 4]>> = Vec::new();
        for index in 0..self.width {
            pixels.push(Vec::new());
            for _ in 0..self.height {
                pixels[index as usize].push([0, 0, 0, 0]);
            }
        }
        for height_index in 0..self.height {
            for width_index in 0..self.width {
                let source_color = self.pixels[width_index as usize][height_index as usize];
                let destination_width_index = (self.width - 1) - width_index;
                let destination_height_index = height_index;
                pixels[destination_width_index as usize][destination_height_index as usize] = source_color;
            }
        }
        ImageFragment {
            pixels: pixels,
            width: self.width,
            height: self.height
        }
    }
}

struct Canvas {
    width: u32,
    height: u32
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        Canvas {
            width: width,
            height: height
        }
    }
    fn get_wave_function(&self, source_image_file_path: &str, fragment_width: u32, fragment_height: u32, is_reflection_permitted: bool, is_rotation_permitted: bool, is_periodic: bool, contains_ground: bool) -> WaveFunction<ImageFragment> {

        // get all of the possible image fragments from the original image
        let mut image_reader = ImageReader::open(source_image_file_path).expect("The source image file should exist at the provided file path.");
        image_reader.set_format(ImageFormat::Bmp);
        let image = image_reader.decode().unwrap();
        let image_width = image.width();
        let image_height = image.height();

        let mut image_fragments: HashSet<ImageFragment> = HashSet::new();
        let mut image_fragment_duplicates_total_per_image_fragment: HashMap<ImageFragment, f32> = HashMap::new();
        let mut ground_image_fragments: HashSet<ImageFragment> = HashSet::new();

        for image_height_index in 0..(image_height - (fragment_height - 1)) {
            for image_width_index in 0..(image_width - (fragment_width - 1)) {
                let mut oriented_image_fragments: Vec<ImageFragment> = Vec::new();
                let mut image_fragment = ImageFragment::new_from_image(&image, image_width_index, image_height_index, fragment_width, fragment_height);

                if image_height_index + 1 == (image_height - (fragment_height - 1)) {
                    ground_image_fragments.insert(image_fragment.clone());
                }

                oriented_image_fragments.push(image_fragment.clone());

                if is_reflection_permitted {
                    if is_rotation_permitted {
                        image_fragment = image_fragment.rotate();
                        oriented_image_fragments.push(image_fragment.clone());
                        image_fragment = image_fragment.rotate();
                        oriented_image_fragments.push(image_fragment.clone());
                        image_fragment = image_fragment.rotate();
                        oriented_image_fragments.push(image_fragment.clone());
                        image_fragment = image_fragment.flip();
                        oriented_image_fragments.push(image_fragment.clone());
                        image_fragment = image_fragment.rotate();
                        oriented_image_fragments.push(image_fragment.clone());
                        image_fragment = image_fragment.rotate();
                        oriented_image_fragments.push(image_fragment.clone());
                        image_fragment = image_fragment.rotate();
                        oriented_image_fragments.push(image_fragment.clone());
                    }
                    else {
                        image_fragment = image_fragment.flip();
                        oriented_image_fragments.push(image_fragment.clone());
                    }
                }
                else if is_rotation_permitted {
                    image_fragment = image_fragment.rotate();
                    oriented_image_fragments.push(image_fragment.clone());
                    image_fragment = image_fragment.rotate();
                    oriented_image_fragments.push(image_fragment.clone());
                    image_fragment = image_fragment.rotate();
                    oriented_image_fragments.push(image_fragment.clone());
                }
                
                for image_fragment in oriented_image_fragments.into_iter() {
                    if !image_fragment_duplicates_total_per_image_fragment.contains_key(&image_fragment) {
                        image_fragment_duplicates_total_per_image_fragment.insert(image_fragment.clone(), 1.0);
                    }
                    else {
                        image_fragment_duplicates_total_per_image_fragment.insert(image_fragment.clone(), image_fragment_duplicates_total_per_image_fragment.get(&image_fragment).unwrap() + 1.0);
                    }

                    image_fragments.insert(image_fragment);
                }
            }
        }

        // construct node state collections such that only those image fragments that overlap can be next to each other
        let mut node_state_collections: Vec<NodeStateCollection<ImageFragment>> = Vec::new();

        // starting from the top-left, only permit those image fragments that would overlap in the remaining eight pixels
        // root         pixel 1     pixel 2
        // pixel 3      pixel 4     pixel 5
        // pixel 6      pixel 7     pixel 8

        let mut permitted_node_states_per_height_offset_per_width_offset_per_node_state: HashMap<&ImageFragment, HashMap<i8, HashMap<i8, Vec<ImageFragment>>>> = HashMap::new();
        for root_image_fragment in image_fragments.iter() {
            //println!("====================");
            //println!("Root:");
            //root_image_fragment.print();
            let mut permitted_node_states_per_height_offset_per_width_offset: HashMap<i8, HashMap<i8, Vec<ImageFragment>>> = HashMap::new();
            for width_offset in -1..=1 as i8 {
                let mut permitted_node_states_per_height_offset: HashMap<i8, Vec<ImageFragment>> = HashMap::new();
                for height_offset in -1..=1 as i8 {
                    // do not setup node state collection for root overlapping root
                    if !(height_offset == 0 && width_offset == 0 ||
                        height_offset.abs() == 1 && width_offset.abs() == 1) {
                        let mut permitted_node_states: Vec<ImageFragment> = Vec::new();
                        for other_image_fragment in image_fragments.iter() {
                            if root_image_fragment.is_overlapping(other_image_fragment, width_offset, height_offset) {
                                //println!("overlapping at {} {}", width_offset, height_offset);
                                //other_image_fragment.print();
                                permitted_node_states.push(other_image_fragment.clone());
                            }
                        }
                        permitted_node_states_per_height_offset.insert(height_offset, permitted_node_states);
                    }
                }
                permitted_node_states_per_height_offset_per_width_offset.insert(width_offset, permitted_node_states_per_height_offset);
            }
            permitted_node_states_per_height_offset_per_width_offset_per_node_state.insert(root_image_fragment, permitted_node_states_per_height_offset_per_width_offset);
        }

        // create distinct node state collections per offset height per offset width
        let mut node_state_collection_ids_per_height_offset_per_width_offset: HashMap<i8, HashMap<i8, Vec<String>>> = HashMap::new();
        for (from_node_state, permitted_node_states_per_height_offset_per_width_offset) in permitted_node_states_per_height_offset_per_width_offset_per_node_state.into_iter() {
            for (width_offset, permitted_node_states_per_height_offset) in permitted_node_states_per_height_offset_per_width_offset.into_iter() {
                node_state_collection_ids_per_height_offset_per_width_offset.entry(width_offset).or_insert(HashMap::new());
                for (height_offset, permitted_node_states) in permitted_node_states_per_height_offset.into_iter() {
                    node_state_collection_ids_per_height_offset_per_width_offset.get_mut(&width_offset).unwrap().entry(height_offset).or_insert(Vec::new());

                    let node_state_collection_id = Uuid::new_v4().to_string();
                    let node_state_collection: NodeStateCollection<ImageFragment> = NodeStateCollection::new(node_state_collection_id.clone(), from_node_state.clone(), permitted_node_states);
                    node_state_collection_ids_per_height_offset_per_width_offset.get_mut(&width_offset).unwrap().get_mut(&height_offset).unwrap().push(node_state_collection_id);
                    node_state_collections.push(node_state_collection);
                }
            }
        }

        // construct nodes
        let mut nodes: Vec<Node<ImageFragment>> = Vec::new();

        // create grid of node IDs cooresponding to each image fragment's top-left corner
        let mut node_id_per_height_index_per_width_index: HashMap<usize, HashMap<usize, String>> = HashMap::new();
        for node_width_index in 0..(self.width - (fragment_width - 1)) as usize {
            let mut node_id_per_height_index: HashMap<usize, String> = HashMap::new();
            for node_height_index in 0..(self.height - (fragment_height - 1)) as usize {
                let node_id: String = format!("node_{}_{}", node_width_index, node_height_index);
                node_id_per_height_index.insert(node_height_index, node_id);
            }
            node_id_per_height_index_per_width_index.insert(node_width_index, node_id_per_height_index);
        }

        // create each node such that its relative node state collections are specified
        for node_width_index in 0..(self.width - (fragment_width - 1)) as i8 {
            for node_height_index in 0..(self.height - (fragment_height - 1)) as i8 {
                let node_id: &String = node_id_per_height_index_per_width_index.get(&(node_width_index as usize)).unwrap().get(&(node_height_index as usize)).unwrap();
                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                for neighbor_width_offset in -1..=1 as i8 {
                    for neighbor_height_offset in -1..=1 as i8 {
                        if !(neighbor_width_offset == 0 && neighbor_height_offset == 0 ||
                            neighbor_width_offset.abs() == 1 && neighbor_height_offset.abs() == 1) {
                            let mut neighbor_width_index = node_width_index + neighbor_width_offset;
                            let mut neighbor_height_index = node_height_index + neighbor_height_offset;
                            if is_periodic {
                                if neighbor_width_index < 0 {
                                    neighbor_width_index += (self.width - (fragment_width - 1)) as i8;
                                }
                                else if neighbor_width_index >= (self.width - (fragment_width - 1)) as i8 {
                                    neighbor_width_index -= (self.width - (fragment_width - 1)) as i8;
                                }
                                if neighbor_height_index < 0 {
                                    neighbor_height_index += (self.height - (fragment_height - 1)) as i8;
                                }
                                else if neighbor_height_index >= (self.height - (fragment_height - 1)) as i8 {
                                    neighbor_height_index -= (self.height - (fragment_height - 1)) as i8;
                                }
                            }
                            if neighbor_width_index >= 0 &&
                                neighbor_width_index < (self.width - (fragment_width - 1)) as i8 &&
                                neighbor_height_index >= 0 &&
                                neighbor_height_index < (self.height - (fragment_height - 1)) as i8 {

                                let neighbor_node_id = node_id_per_height_index_per_width_index.get(&(neighbor_width_index as usize)).unwrap().get(&(neighbor_height_index as usize)).unwrap();
                                let node_state_collection_ids = node_state_collection_ids_per_height_offset_per_width_offset.get(&neighbor_width_offset).unwrap().get(&neighbor_height_offset).unwrap();
                                node_state_collection_ids_per_neighbor_node_id.insert(neighbor_node_id.clone(), node_state_collection_ids.clone());
                            }
                        }
                    }
                }

                let mut node_state_ratio_per_node_state_id: HashMap<ImageFragment, f32> = HashMap::new();
                if contains_ground {
                    if node_height_index + 1 == (self.height - (fragment_height - 1)) as i8 {
                        for (image_fragment, ratio) in image_fragment_duplicates_total_per_image_fragment.iter() {
                            if ground_image_fragments.contains(image_fragment) {
                                node_state_ratio_per_node_state_id.insert(image_fragment.clone(), *ratio);
                            }
                        }
                    }
                    else {
                        for (image_fragment, ratio) in image_fragment_duplicates_total_per_image_fragment.iter() {
                            if !ground_image_fragments.contains(image_fragment) {
                                node_state_ratio_per_node_state_id.insert(image_fragment.clone(), *ratio);
                            }
                        }
                    }
                }
                else {
                    node_state_ratio_per_node_state_id = image_fragment_duplicates_total_per_image_fragment.clone();
                }

                let node: Node<ImageFragment> = Node::new(node_id.clone(), node_state_ratio_per_node_state_id, node_state_collection_ids_per_neighbor_node_id);
                nodes.push(node);
            }
        }

        WaveFunction::new(nodes, node_state_collections)
    }
    fn print(&self, collapsed_wave_function: CollapsedWaveFunction<ImageFragment>, fragment_width: u32, fragment_height: u32) {
        let mut node_state_per_height_index_per_width_index: HashMap<usize, HashMap<usize, Option<ImageFragment>>> = HashMap::new();
        for width_index in 0..self.width as usize {
            let mut node_state_per_height_index: HashMap<usize, Option<ImageFragment>> = HashMap::new();
            for height_index in 0..self.height as usize {
                node_state_per_height_index.insert(height_index, None);
            }
            node_state_per_height_index_per_width_index.insert(width_index, node_state_per_height_index);
        }

        for (node_id, node_state) in collapsed_wave_function.node_state_per_node.into_iter() {
            let node_id_split = node_id.split("_").collect::<Vec<&str>>();
            let node_width_index = node_id_split[1].parse::<usize>().unwrap();
            let node_height_index = node_id_split[2].parse::<usize>().unwrap();
            node_state_per_height_index_per_width_index.get_mut(&node_width_index).unwrap().insert(node_height_index, Some(node_state));
        }

        let mut pixels: Vec<Vec<[u8; 4]>> = Vec::new();
        for _ in 0..self.width {
            let mut vec = Vec::new();
            for _ in 0..self.height {
                vec.push([0 as u8, 0, 128, 0]);
            }
            pixels.push(vec);
        }

        for width_index in 0..(self.width - (fragment_width - 1)) as usize {
            for height_index in 0..(self.height - (fragment_height - 1)) as usize {
                let node_state = node_state_per_height_index_per_width_index.get(&width_index).unwrap().get(&height_index).unwrap().as_ref().unwrap();
                
                if width_index + 1 == (self.width - (fragment_width - 1)) as usize || height_index + 1 == (self.height - (fragment_height - 1)) as usize {
                    for pixel_height_index in 0..node_state.height as usize {
                        for pixel_width_index in 0..node_state.width as usize {
                            pixels[width_index + pixel_width_index][height_index + pixel_height_index] = node_state.pixels[pixel_width_index][pixel_height_index];
                        }
                    }
                }
                else {
                    pixels[width_index][height_index] = node_state.pixels[0][0];
                }
            }
        }

        for height_index in 0..self.height as usize {
            for width_index in 0..self.width as usize {
                let color = pixels[width_index][height_index];
                print_pixel(&color);
            }
            println!("");
        }
    }
    fn print_step(&self, collapsed_node_states: &Vec<CollapsedNodeState<ImageFragment>>, step_index: usize) {
        let mut pixels: Vec<Vec<[u8; 4]>> = Vec::new();
        for _ in 0..self.width {
            let mut vec = Vec::new();
            for _ in 0..self.height {
                vec.push([0 as u8, 0, 128, 0]);
            }
            pixels.push(vec);
        }

        for current_step_index in 0..=step_index {
            let collapsed_node_state = collapsed_node_states.get(current_step_index).unwrap();
            let node_id_split = collapsed_node_state.node_id.split("_").collect::<Vec<&str>>();
            let node_width_index = node_id_split[1].parse::<usize>().unwrap();
            let node_height_index = node_id_split[2].parse::<usize>().unwrap();
            if let Some(pixel_color) = &collapsed_node_state.node_state_id {
                if node_width_index + pixel_color.width as usize == self.width as usize || node_height_index + pixel_color.height as usize == self.height as usize {
                    for pixel_height_index in 0..pixel_color.height as usize {
                        for pixel_width_index in 0..pixel_color.width as usize{
                            pixels[node_width_index + pixel_width_index][node_height_index + pixel_height_index] = pixel_color.pixels[pixel_width_index][pixel_height_index];
                        }
                    }
                }
                else {
                    pixels[node_width_index][node_height_index] = pixel_color.pixels[0][0];
                }
            }
            else {
                pixels[node_width_index][node_height_index] = [255, 0, 0, 255];
            }
        }

        println!("Step {step_index} ======================================");
        for height_index in 0..self.height as usize {
            for width_index in 0..self.width as usize {
                print_pixel(&pixels[width_index][height_index]);
            }
            println!("");
        }
    }
}

#[allow(dead_code)]
enum Image {
    Plant,
    Rooms,
    Houses
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    //pretty_env_logger::init();

    let is_reflection_permitted: bool;
    let is_rotation_permitted: bool;
    let is_periodic: bool;
    let image_base64: String;
    let contains_ground: bool;

    let plant_image_base64: String = String::from("Qk2uBgAAAAAAADYAAAAoAAAAFwAAABcAAAABABgAAAAAAHgGAAAAAAAAAAAAAAAAAAAAAAAAV3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5AAAAV3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5AKoAV3q5V3q5V3q5V3q5V3q5V3q5V3q5AKoAV3q5V3q5V3q5V3q5V3q5V3q5AAAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoA8ui/AKoAAKoA8ui/8ui/8ui/8ui/AKoA8ui/AKoAAKoA8ui/8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/AKoAAKoA8ui/8ui/8ui/APL/8ui/8ui/AKoAAKoA8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/AKoA8ui/8ui/APL/AKoAAPL/8ui/8ui/AKoA8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/AKoAAKoAAKoA8ui/8ui/APL/8ui/8ui/8ui/AKoA8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/8ui/8ui/AKoA8ui/AKoAAKoA8ui/8ui/8ui/8ui/AKoAAKoAAKoA8ui/8ui/AAAA8ui/8ui/8ui/AKoAAKoAAKoA8ui/8ui/8ui/8ui/AKoA8ui/8ui/AKoAAKoA8ui/8ui/AKoAAKoA8ui/AKoA8ui/8ui/AAAA8ui/8ui/8ui/AKoA8ui/AKoAAKoA8ui/8ui/8ui/APL/8ui/8ui/8ui/AKoA8ui/8ui/AKoA8ui/8ui/APL/8ui/8ui/AAAA8ui/8ui/AKoAAKoA8ui/8ui/AKoAAKoA8ui/APL/AKoAAPL/8ui/8ui/AKoA8ui/AKoAAKoA8ui/APL/AKoAAPL/8ui/AAAA8ui/8ui/AKoA8ui/8ui/8ui/8ui/AKoA8ui/8ui/APL/8ui/8ui/AKoAAKoA8ui/AKoA8ui/8ui/8ui/APL/8ui/8ui/AAAA8ui/8ui/APL/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/AKoAAKoA8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/AAAA8ui/APL/AKoAAPL/8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/AKoA8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/AAAA8ui/8ui/APL/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoA8ui/8ui/8ui/8ui/APL/8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/APL/8ui/8ui/8ui/8ui/8ui/8ui/8ui/APL/8ui/8ui/8ui/APL/AKoAAPL/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/APL/AKoAAPL/8ui/8ui/8ui/8ui/8ui/APL/AKoAAPL/8ui/8ui/8ui/APL/8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/APL/8ui/8ui/8ui/8ui/8ui/8ui/8ui/APL/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AAAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AAAA");
    let rooms_image_base64: String = String::from("Qk02AwAAAAAAADYAAAAoAAAAEAAAABAAAAABABgAAAAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAA4ODg4ODg4ODg4ODgAAAAAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAA4ODg4ODg4ODg4ODgAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAA4ODg4ODg4ODg4ODg4ODgAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODg4ODgAAAA4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAA4ODg4ODg4ODg4ODg4ODgAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAA4ODg4ODg4ODg4ODg4ODgAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAA4ODg4ODg4ODg4ODg4ODg4ODgAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4ODgAAAAAAAAAAAA");
    let houses_image_base64: String = String::from("Qk02AwAAAAAAADYAAAAoAAAAEAAAABAAAAABABgAAAAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFWqAFWqAAAAAFWqAFWqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFWqAFWqAAAAAFWqAFWqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACqAACqAFWqAFWqAFWqAACqAACqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACqAACqAACqAACqAACqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACqAACqAACqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFWqAFWqAAAAAFWqAFWqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFWqAFWqAAAAAFWqAFWqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACqAACqAFWqAFWqAFWqAACqAACqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACqAACqAACqAACqAACqAAAAAAAAAFWqAFWqAAAAAFWqAFWqAAAAAAAAAAAAAAAAAAAAAACqAACqAACqAAAAAAAAAAAAAFWqAFWqAAAAAFWqAFWqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACqAACqAFWqAFWqAFWqAACqAACqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACqAACqAACqAACqAACqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACqAACqAACqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");

    //================================================
    // NOTE: This can be change for different examples
    let chosen_image = Image::Rooms;
    let draw_each_frame: bool = false;
    //================================================

    match chosen_image {
        Image::Plant => {
            is_reflection_permitted = true;
            is_rotation_permitted = false;
            is_periodic = false;
            image_base64 = plant_image_base64;
            contains_ground = true;
        },
        Image::Rooms => {
            is_reflection_permitted = true;
            is_rotation_permitted = true;
            is_periodic = true;
            image_base64 = rooms_image_base64;
            contains_ground = false;
        },
        Image::Houses => {
            is_reflection_permitted = false;
            is_rotation_permitted = false;
            is_periodic = true;
            image_base64 = houses_image_base64;
            contains_ground = false;
        }
    }

    let mut file = tempfile::NamedTempFile::new().unwrap();
    let bytes = base64::decode(image_base64).unwrap();
    file.write(bytes.as_slice()).unwrap();
    let file_path: &str = file.path().to_str().unwrap();

    let canvas = Canvas::new(40, 40);
    let fragment_width: u32 = 3;
    let fragment_height: u32 = 3;
    let wave_function = canvas.get_wave_function(file_path, fragment_width, fragment_height, is_reflection_permitted, is_rotation_permitted, is_periodic, contains_ground);

    file.close().unwrap();

    wave_function.validate().unwrap();

    let mut random_instance = fastrand::Rng::new();
    let random_seed = Some(random_instance.u64(..));

    let mut collapsable_wave_function = wave_function.get_collapsable_wave_function::<EntropicCollapsableWaveFunction<ImageFragment>>(random_seed);
    
    let start = Instant::now();
    let duration: Duration;

    if draw_each_frame {
        let collapsed_node_states = collapsable_wave_function.collapse_into_steps().unwrap();
        duration = start.elapsed();

        for step_index in 0..collapsed_node_states.len() {
            canvas.print_step(&collapsed_node_states, step_index);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    else {
        let collapsed_wave_function = collapsable_wave_function.collapse().unwrap();
        duration = start.elapsed();

        canvas.print(collapsed_wave_function, fragment_width, fragment_height);
    }

    println!("Duration: {:?}", duration);
}