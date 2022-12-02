use std::{collections::{HashSet, HashMap}, io::Write, time::Instant};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{WaveFunction, NodeStateCollection, Node, collapsable_wave_function::{accommodating_collapsable_wave_function::AccommodatingCollapsableWaveFunction, collapsable_wave_function::{CollapsableWaveFunction, CollapsedWaveFunction}, sequential_collapsable_wave_function::SequentialCollapsableWaveFunction, spreading_collapsable_wave_function::SpreadingCollapsableWaveFunction}};
use image::{io::Reader as ImageReader, GenericImageView, DynamicImage, ImageFormat};
use colored::{Colorize, ColoredString};
use std::cmp;

fn print_pixel(color: &[u8; 4]) {
    let character = "\u{2588}";
    print!("{}", character.truecolor(color[0], color[1], color[2]));
    print!("{}", character.truecolor(color[0], color[1], color[2]));
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
struct ImageFragment {
    // the RGBA color per height per width
    pixels: [[[u8; 4]; 3]; 3]
}

impl ImageFragment {
    fn new_from_image(image: &DynamicImage, width_index: u32, height_index: u32) -> ImageFragment {
        let mut pixels = [[[0; 4]; 3]; 3];
        for pixel_height_offset in 0..3 {
            for pixel_width_offset in 0..3 {
                let pixel_width_index = width_index + pixel_width_offset;
                let pixel_height_index = height_index + pixel_height_offset;
                let pixel = image.get_pixel(pixel_width_index, pixel_height_index);
                pixels[pixel_width_offset as usize][pixel_height_offset as usize] = pixel.0;
            }
        }
        ImageFragment {
            pixels: pixels
        }
    }
    fn is_overlapping(&self, other_image_fragment: &ImageFragment, width_offset: i8, height_offset: i8) -> bool {
        let mut is_at_least_one_pixel_nonoverlapping: bool = false;
        for self_height_index in cmp::max(0, height_offset)..cmp::min(3, height_offset + 3) {
            let other_height_index = self_height_index - height_offset;
            for self_width_index in cmp::max(0, width_offset)..cmp::min(3, width_offset + 3) {
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
    fn print(&self) {
        for height_index in 0..3 as usize {
            for width_index in 0..3 as usize {
                let color = self.pixels[width_index][height_index];
                print_pixel(&color);
            }
            println!("");
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
    fn get_wave_function(&self, source_image_file_path: &str) -> WaveFunction<ImageFragment> {

        // get all of the possible image fragments from the original image
        let mut image_reader = ImageReader::open(source_image_file_path).expect("The source image file should exist at the provided file path.");
        image_reader.set_format(ImageFormat::Bmp);
        let image = image_reader.decode().unwrap();
        let image_width = image.width();
        let image_height = image.height();

        let mut image_fragments: HashSet<ImageFragment> = HashSet::new();
        let mut image_fragment_duplicates_total_per_image_fragment: HashMap<ImageFragment, f32> = HashMap::new();

        for image_height_index in 0..(image_height - 2) {
            for image_width_index in 0..(image_width - 2) {
                let image_fragment = ImageFragment::new_from_image(&image, image_width_index, image_height_index);
                if !image_fragment_duplicates_total_per_image_fragment.contains_key(&image_fragment) {
                    image_fragment_duplicates_total_per_image_fragment.insert(image_fragment.clone(), 1.0);
                }
                else {
                    let image_fragment_duplicates_total: &f32 = image_fragment_duplicates_total_per_image_fragment.get(&image_fragment).unwrap();
                    image_fragment_duplicates_total_per_image_fragment.insert(image_fragment.clone(), image_fragment_duplicates_total + 1.0);
                }

                image_fragments.insert(image_fragment);
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
            println!("====================");
            println!("Root:");
            root_image_fragment.print();
            let mut permitted_node_states_per_height_offset_per_width_offset: HashMap<i8, HashMap<i8, Vec<ImageFragment>>> = HashMap::new();
            for width_offset in -2..3 {
                let mut permitted_node_states_per_height_offset: HashMap<i8, Vec<ImageFragment>> = HashMap::new();
                for height_offset in -2..3 {
                    // do not setup node state collection for root overlapping root
                    if !(height_offset == 0 && width_offset == 0) {
                        let mut permitted_node_states: Vec<ImageFragment> = Vec::new();
                        for other_image_fragment in image_fragments.iter() {
                            if root_image_fragment.is_overlapping(&other_image_fragment, width_offset, height_offset) {
                                println!("overlapping at {} {}", width_offset, height_offset);
                                other_image_fragment.print();
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
                if !node_state_collection_ids_per_height_offset_per_width_offset.contains_key(&width_offset) {
                    node_state_collection_ids_per_height_offset_per_width_offset.insert(width_offset, HashMap::new());
                }
                for (height_offset, permitted_node_states) in permitted_node_states_per_height_offset.into_iter() {
                    if !node_state_collection_ids_per_height_offset_per_width_offset.get(&width_offset).unwrap().contains_key(&height_offset) {
                        node_state_collection_ids_per_height_offset_per_width_offset.get_mut(&width_offset).unwrap().insert(height_offset, Vec::new());
                    }
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
        for node_width_index in 0..(self.width - 2) as usize {
            let mut node_id_per_height_index: HashMap<usize, String> = HashMap::new();
            for node_height_index in 0..(self.height - 2) as usize {
                let node_id: String = format!("node_{}_{}", node_width_index, node_height_index);
                node_id_per_height_index.insert(node_height_index, node_id);
            }
            node_id_per_height_index_per_width_index.insert(node_width_index, node_id_per_height_index);
        }

        // create each node such that its relative node state collections are specified
        for node_width_index in 0..(self.width - 2) as i8 {
            for node_height_index in 0..(self.height - 2) as i8 {
                let node_id: &String = node_id_per_height_index_per_width_index.get(&(node_width_index as usize)).unwrap().get(&(node_height_index as usize)).unwrap();
                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                for neighbor_width_offset in -2..3 as i8 {
                    for neighbor_height_offset in -2..3 as i8 {
                        if !(neighbor_width_offset == 0 && neighbor_height_offset == 0) {
                            let neighbor_width_index = node_width_index + neighbor_width_offset;
                            let neighbor_height_index = node_height_index + neighbor_height_offset;
                            if neighbor_width_index >= 0 &&
                                neighbor_width_index < (self.width - 2) as i8 &&
                                neighbor_height_index >= 0 &&
                                neighbor_height_index < (self.height - 2) as i8 {

                                let neighbor_node_id = node_id_per_height_index_per_width_index.get(&(neighbor_width_index as usize)).unwrap().get(&(neighbor_height_index as usize)).unwrap();
                                let node_state_collection_ids = node_state_collection_ids_per_height_offset_per_width_offset.get(&neighbor_width_offset).unwrap().get(&neighbor_height_offset).unwrap();
                                node_state_collection_ids_per_neighbor_node_id.insert(neighbor_node_id.clone(), node_state_collection_ids.clone());
                            }
                        }
                    }
                }

                let node: Node<ImageFragment> = Node::new(node_id.clone(), image_fragment_duplicates_total_per_image_fragment.clone(), node_state_collection_ids_per_neighbor_node_id);
                nodes.push(node);
            }
        }

        WaveFunction::new(nodes, node_state_collections)
    }
    fn print(&self, collapsed_wave_function: CollapsedWaveFunction<ImageFragment>) {
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

        let mut color_per_height_index_per_width_index: HashMap<usize, HashMap<usize, [u8; 4]>> = HashMap::new();
        for width_index in 0..(self.width - 2) as usize {
            let mut color_per_height_index: HashMap<usize, [u8; 4]> = HashMap::new();
            for height_index in 0..(self.height - 2) as usize {
                let node_state = node_state_per_height_index_per_width_index.get(&width_index).unwrap().get(&height_index).unwrap().as_ref().unwrap();
                
                color_per_height_index.insert(height_index, node_state.pixels[0][0]);

                // TODO get right and bottom
            }
            color_per_height_index_per_width_index.insert(width_index, color_per_height_index);
        }

        for height_index in 0..(self.height - 2) as usize {
            for width_index in 0..(self.width - 2) as usize {
                let color = color_per_height_index_per_width_index.get(&width_index).unwrap().get(&height_index).unwrap();
                print_pixel(color);
            }
            println!("");
        }
    }
}

fn main() {
    let plant_image_base64: String = String::from("Qk1eGQAAAAAAADYAAAAoAAAALgAAAC4AAAABABgAAAAAACgZAAAAAAAAAAAAAAAAAAAAAAAAV3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5AABXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerlXerkAAFd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uQCqAACqAFd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uQCqAACqAFd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uVd6uQAAV3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5AKoAAKoAV3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5AKoAAKoAV3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5V3q5AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/LovwDy/wDy/wCqAACqAADy/wDy//Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/APL/APL/AKoAAKoAAPL/APL/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAACqAACqAPLov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/LovwCqAACqAACqAACqAPLov/LovwCqAACqAPLov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/APL/APL/AKoAAKoAAPL/APL/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/APL/APL/AKoAAKoAAPL/APL/8ui/8ui/AADy6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L8A8v8A8v8AqgAAqgAA8v8A8v/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L8A8v8A8v8AqgAAqgAA8v8A8v/y6L/y6L8AAPLov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/LovwCqAACqAPLov/Lov/Lov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/APL/APL/AKoAAKoAAPL/APL/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AADy6L/y6L8A8v8A8v8AqgAAqgAA8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoAAKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgAAqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAACqAACqAPLov/Lov/Lov/Lov/Lov/Lov/Lov/LovwCqAACqAPLov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AKoAAKoA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AqgAAqgDy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/Lov/Lov/LovwDy/wDy/wCqAACqAADy/wDy//Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/8ui/8ui/APL/APL/AKoAAKoAAPL/APL/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8A8v8A8v8AqgAAqgAA8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8A8v8A8v8AqgAAqgAA8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwDy/wDy/wCqAACqAADy/wDy//Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwDy/wDy/wCqAACqAADy/wDy//Lov/Lov/Lov/Lov/Lov/LovwDy/wDy//Lov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/APL/APL/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AADy6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8A8v8A8v/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L/y6L8AAPLov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/Lov/LovwAA8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/8ui/AAA=");
    let mut file = tempfile::NamedTempFile::new().unwrap();
    let bytes = base64::decode(plant_image_base64).unwrap();
    println!("Image bytes: {}", bytes.len());
    file.write(bytes.as_slice()).unwrap();
    let file_path: &str = file.path().to_str().unwrap();

    let canvas = Canvas::new(20, 20);
    let wave_function = canvas.get_wave_function(file_path);

    file.close().unwrap();

    wave_function.validate().unwrap();

    println!("validated");

    let mut collapsable_wave_function = wave_function.get_collapsable_wave_function::<SpreadingCollapsableWaveFunction<ImageFragment>>(None);
    
    let start = Instant::now();
    let collapsed_wave_function = collapsable_wave_function.collapse().unwrap();

    // pull out the root pixels from every node's image fragment along with the right and bottom walls
    canvas.print(collapsed_wave_function);
    let duration = start.elapsed();
    println!("Duration: {:?}", duration);
}