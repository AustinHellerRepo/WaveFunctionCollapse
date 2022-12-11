use std::collections::{VecDeque, HashMap, HashSet};
use austinhellerrepo_common_rust::segment_container::{Segment, SegmentContainer};
use colored::Colorize;
use log::debug;
use rand::Rng;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{Node, NodeStateCollection, WaveFunction, collapsable_wave_function::{entropic_collapsable_wave_function::EntropicCollapsableWaveFunction, collapsable_wave_function::CollapsableWaveFunction}};
use std::iter::zip;

fn print_pixel(color: &[u8; 4]) {
    let character = "\u{2588}";
    print!("{}{}", character.truecolor(color[0], color[1], color[2]), character.truecolor(color[0], color[1], color[2]));
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
enum TileType {
    Empty,
    Solid,
    JumpThrough
}

impl TileType {
    fn get_color(&self) -> [u8; 4] {
        match self {
            TileType::Solid => {
                [0, 0, 255, 255]
            },
            TileType::JumpThrough => {
                [150, 75, 0, 255]
            },
            TileType::Empty => {
                [0, 0, 0, 255]
            }
        }
    }
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Empty
    }
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
enum ElementType {
    Seaweed,
    Bomb,
    Sproinger
}

impl ElementType {
    fn get_color(&self) -> [u8; 4] {
        match self {
            ElementType::Seaweed => {
                [0, 255, 0, 255]
            },
            ElementType::Bomb => {
                [128, 128, 128, 255]
            },
            ElementType::Sproinger => {
                [200, 0, 0, 255]
            }
        }
    }
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
enum Placable {
    Tile(TileType),
    Element(ElementType)
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
struct PlacedPlacable {
    placable: Placable,
    location: (usize, usize)
}

impl PlacedPlacable {
    fn new(placable: Placable, x: usize, y: usize) -> Self {
        PlacedPlacable {
            placable: placable,
            location: (x, y)
        }
    }
    fn get_color(&self) -> [u8; 4] {
        match &self.placable {
            Placable::Element(element_type) => {
                element_type.get_color()
            },
            Placable::Tile(tile_type) => {
                tile_type.get_color()
            }
        }
    }
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
struct PlacedPlacableCollection {
    placed_placables: Vec<PlacedPlacable>
}

impl Default for PlacedPlacableCollection {
    fn default() -> Self {
        PlacedPlacableCollection {
            placed_placables: Vec::new()
        }
    }
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
struct NodeState {
    location: (usize, usize)
}

struct Level {
    width: usize,
    height: usize,
    placed_placables: Vec<PlacedPlacable>
}

impl Level {
    fn new(width: usize, height: usize, placed_placables: Vec<PlacedPlacable>) -> Self {
        Level {
            width: width,
            height: height,
            placed_placables: placed_placables
        }
    }
    fn default() -> Self {
        let width: usize = 50;
        let height: usize = 30;
        let mut placed_placables: Vec<PlacedPlacable> = Vec::new();
        for width_index in ((width as f32 * 0.25) as usize)..((width as f32 * 0.4) as usize) {
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::Solid), width_index, 0));
        }
        for width_index in ((width as f32 * 0.6) as usize)..((width as f32 * 0.75) as usize) {
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::Solid), width_index, 0));
        }
        for height_index in 0..height {
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::Solid), 0, height_index));
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::Solid), width - 1, height_index));
        }
        for width_index in 1..(width - 1) {
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::Solid), width_index, height - 1));
        }
        for width_index in 1..(width as f32 * 0.25) as usize {
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::Solid), width_index, (height as f32 * 0.4) as usize));
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::Solid), width - width_index - 1, (height as f32 * 0.65) as usize));
        }
        for width_index in 1..((width as f32 * 0.75) as usize) {
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::JumpThrough), width_index, (height as f32 * 0.25) as usize));
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::JumpThrough), width - width_index - 1, (height as f32 * 0.5) as usize));
            placed_placables.push(PlacedPlacable::new(Placable::Tile(TileType::JumpThrough), width_index, (height as f32 * 0.75) as usize));
        }
        placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Sproinger), 1, height - 2));
        placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Sproinger), (width as f32 * 0.25) as usize, (height as f32 * 0.75) as usize - 1));
        placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Sproinger), (width as f32 * 0.5) as usize, (height as f32 * 0.5) as usize - 1));
        placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Bomb), (width as f32 * 0.75) as usize - 1, (height as f32 * 0.75) as usize - 1));
        placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Seaweed), (width as f32 * 0.25) as usize, height - 2));
        placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Seaweed), (width as f32 * 0.5) as usize, height - 2));
        placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Seaweed), (width as f32 * 0.75) as usize, height - 2));
        for width_index in 0..3 {
            placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Seaweed), (width_index as f32 * (width as f32 * 0.25) / 3.0) as usize + 1, (height as f32 * 0.4) as usize - 1));
            placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Seaweed), width - (width_index as f32 * (width as f32 * 0.25) / 3.0) as usize - 2, (height as f32 * 0.65) as usize - 1));
        }
        placed_placables.push(PlacedPlacable::new(Placable::Element(ElementType::Bomb), (2.0 * (width as f32 * 0.25) / 3.0) as usize + 3, (height as f32 * 0.4) as usize - 1));

        Level {
            width: width,
            height: height,
            placed_placables: placed_placables
        }
    }
    fn print(&self) {
        let mut level_grid: Vec<Vec<[u8; 4]>> = Vec::new();
        for width_index in 0..self.width {
            level_grid.push(Vec::new());
            for _ in 0..self.height {
                level_grid[width_index].push([0, 0, 0, 255]);
            }
        }

        for placed_placable in self.placed_placables.iter() {
            level_grid[placed_placable.location.0][placed_placable.location.1] = placed_placable.get_color();
        }

        for height_index in 0..self.height {
            for width_index in 0..self.width {
                print_pixel(&level_grid[width_index][height_index]);
            }
            println!("");
        }
    }
    pub fn get_similar_level(&self) -> Self {

        let mut placed_placable_per_location: HashMap<(usize, usize), &PlacedPlacable> = HashMap::new();
        for placed_placable in self.placed_placables.iter() {
            placed_placable_per_location.insert(placed_placable.location, placed_placable);
        }

        let mut nodes: Vec<Node<NodeState>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<NodeState>> = Vec::new();

        // collect PlacedPlacableCollection instances representing the walls
        let mut top_walls: Vec<PlacedPlacableCollection> = Vec::new();
        let mut right_walls: Vec<PlacedPlacableCollection> = Vec::new();
        let mut bottom_walls: Vec<PlacedPlacableCollection> = Vec::new();
        let mut left_walls: Vec<PlacedPlacableCollection> = Vec::new();

        let mut placed_placable_per_location: HashMap<(usize, usize), &PlacedPlacable> = HashMap::new();
        for placed_placable in self.placed_placables.iter() {
            placed_placable_per_location.insert(placed_placable.location, placed_placable);
        }

        let mut current_wall: Option<PlacedPlacableCollection> = None;
        let mut is_end_of_wall_found: bool = false;

        // search along the top and bottom
        for height_index in [0, self.height - 1] {
            for width_index in 0..self.width {
                let location = (width_index, height_index);
                if placed_placable_per_location.contains_key(&location) {
                    let placed_placable: &PlacedPlacable = placed_placable_per_location.get(&location).unwrap();
                    match placed_placable.placable {
                        Placable::Tile(_) => {
                            if current_wall.is_none() {
                                current_wall = Some(PlacedPlacableCollection::default());
                            }
                            if let Some(ref mut placed_placable_collection) = current_wall {
                                placed_placable_collection.placed_placables.push(placed_placable.clone());
                            }
                        },
                        Placable::Element(_) => {
                            is_end_of_wall_found = true;
                        }
                    }
                }
                else {
                    is_end_of_wall_found = true;
                }

                if is_end_of_wall_found {
                    if current_wall.is_some() {
                        if height_index == 0 {
                            top_walls.push(current_wall.unwrap());
                        }
                        else if height_index == self.height - 1 {
                            bottom_walls.push(current_wall.unwrap());
                        }
                        else {
                            panic!("Unexpected height when trying to collect walls.");
                        }
                        current_wall = None;
                    }
                    is_end_of_wall_found = false;
                }
            }

            if current_wall.is_some() {
                if height_index == 0 {
                    top_walls.push(current_wall.unwrap());
                }
                else if height_index == self.height - 1 {
                    bottom_walls.push(current_wall.unwrap());
                }
                else {
                    panic!("Unexpected height when trying to collect walls.");
                }
                current_wall = None;
            }
        }

        // search along the left and right
        for width_index in [0, self.width - 1] {
            for height_index in 0..self.height {
                let location = (width_index, height_index);
                if placed_placable_per_location.contains_key(&location) {
                    let placed_placable: &PlacedPlacable = placed_placable_per_location.get(&location).unwrap();
                    match placed_placable.placable {
                        Placable::Tile(_) => {
                            if current_wall.is_none() {
                                current_wall = Some(PlacedPlacableCollection::default());
                            }
                            if let Some(ref mut placed_placable_collection) = current_wall {
                                placed_placable_collection.placed_placables.push(placed_placable.clone());
                            }
                        },
                        Placable::Element(_) => {
                            is_end_of_wall_found = true;
                        }
                    }
                }
                else {
                    is_end_of_wall_found = true;
                }

                if is_end_of_wall_found {
                    if current_wall.is_some() {
                        if width_index == 0 {
                            left_walls.push(current_wall.unwrap());
                        }
                        else if width_index == self.width - 1 {
                            right_walls.push(current_wall.unwrap());
                        }
                        else {
                            panic!("Unexpected height when trying to collect walls.");
                        }
                        current_wall = None;
                    }
                }
            }

            if current_wall.is_some() {
                if width_index == 0 {
                    left_walls.push(current_wall.unwrap());
                }
                else if width_index == self.width - 1 {
                    right_walls.push(current_wall.unwrap());
                }
                else {
                    panic!("Unexpected height when trying to collect walls.");
                }
                current_wall = None;
            }
        }

        /*let wall_node_ids: Vec<String> = Vec::new();
        for wall in walls.iter() {
            wall_node_ids.push(format!("wall_node_{}_{}", wall.placed_placables[0].location.0, wall.placed_placables[0].location.1));
        }*/

        // determine possible locations per wall
        let mut current_wall_index: usize = 0;
        let mut possible_locations_per_wall_index: HashMap<usize, HashSet<(usize, usize)>> = HashMap::new();
        let mut other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index: HashMap<usize, HashMap<(usize, usize), HashMap<usize, Vec<(usize, usize)>>>> = HashMap::new();
        
        // get the possible locations for each wall
        for ((walls, is_horizontal), width_or_height) in zip(zip([top_walls, bottom_walls, left_walls, right_walls], [true, true, false, false]), [0, self.height - 1, 0, self.width - 1]) {
            debug!("trying walls {:?} which are located at {}", walls, width_or_height);
            if !walls.is_empty() {

                let mut segments: Vec<Segment<usize>> = Vec::new();

                for wall in walls.iter() {

                    debug!("examining wall index {} as {:?}", current_wall_index, wall);

                    // ensure that this wall is not stuck to another wall
                    if wall.placed_placables[0].location.0 == 0 && is_horizontal {
                        // the wall is stuck in the top-left or bottom-left corner
                        debug!("found wall is either in top-left or bottom-left corner");

                        let mut possible_locations: HashSet<(usize, usize)> = HashSet::new();
                        possible_locations.insert(wall.placed_placables[0].location);
                        possible_locations_per_wall_index.insert(current_wall_index, possible_locations);
                    }
                    else if wall.placed_placables[0].location.1 == 0 && !is_horizontal {
                        // the wall is stuck in the top-left or top-right corner
                        debug!("found wall is either in top-left or top-right corner");

                        let mut possible_locations: HashSet<(usize, usize)> = HashSet::new();
                        possible_locations.insert(wall.placed_placables[0].location);
                        possible_locations_per_wall_index.insert(current_wall_index, possible_locations);
                    }
                    else if wall.placed_placables[wall.placed_placables.len() - 1].location.0 == self.width - 1 && is_horizontal {
                        // the wall is stuck in the top-right or bottom-right corner
                        debug!("found wall is either in top-right or bottom-right corner");

                        let mut possible_locations: HashSet<(usize, usize)> = HashSet::new();
                        possible_locations.insert(wall.placed_placables[0].location);
                        possible_locations_per_wall_index.insert(current_wall_index, possible_locations);
                    }
                    else if wall.placed_placables[wall.placed_placables.len() - 1].location.1 == self.height - 1 && !is_horizontal {
                        // the wall is stuck in the bottom-left or bottom-right corner
                        debug!("found wall is either in bottom-left or bottom-right corner");

                        let mut possible_locations: HashSet<(usize, usize)> = HashSet::new();
                        possible_locations.insert(wall.placed_placables[0].location);
                        possible_locations_per_wall_index.insert(current_wall_index, possible_locations);
                    }
                    else {
                        debug!("found wall unstuck from any corners");

                        let wall_start_location = wall.placed_placables[0].location;
                        let wall_end_location = wall.placed_placables[wall.placed_placables.len() - 1].location;

                        let segment_length: usize;
                        if is_horizontal {
                            segment_length = wall_end_location.0 - wall_start_location.0 + 1;
                        }
                        else {
                            segment_length = wall_end_location.1 - wall_start_location.1 + 1;
                        }

                        segments.push(Segment::new(current_wall_index, segment_length));
                    }

                    current_wall_index += 1;
                }

                if !segments.is_empty() {
                    // there is at least one open wall segment that does not touch either corner

                    // get the left-most and right-most a wall can travel based on the existence (or not) of any corner walls
                    let left_most_length: usize;
                    if walls[0].placed_placables[0].location.0 == 0 && is_horizontal {
                        left_most_length = walls[0].placed_placables[walls[0].placed_placables.len() - 1].location.0 + 2;  // +2 spaces away is the next valid location
                    }
                    else if walls[0].placed_placables[0].location.1 == 0 && !is_horizontal {
                        left_most_length = walls[0].placed_placables[walls[0].placed_placables.len() - 1].location.1 + 2;  // +2 spaces away is the next valid location
                    }
                    else {
                        left_most_length = 1;  // 1 space away from 0 is the next valid location
                    }
                    
                    let right_most_length: usize;
                    if is_horizontal {
                        if walls[walls.len() - 1].placed_placables[walls[walls.len() - 1].placed_placables.len() - 1].location.0 == self.width - 1 {
                            right_most_length = walls[walls.len() - 1].placed_placables[0].location.0 - 2;  // -2 spaces away to the left is the next valid location
                        }
                        else {
                            right_most_length = self.width - 2;  // 1 space to the left of the last index is the next valid location
                        }
                    }
                    else {
                        if walls[walls.len() - 1].placed_placables[walls[walls.len() - 1].placed_placables.len() - 1].location.1 == self.height - 1 {
                            right_most_length = walls[walls.len() - 1].placed_placables[0].location.1 - 2;  // -2 spaces away to the left is the next valid location
                        }
                        else {
                            right_most_length = self.height - 2;  // 1 space up from the last index is the next valid location
                        }
                    }

                    let left_most_to_right_most_length = right_most_length - left_most_length + 1;

                    debug!("left_most_to_right_most_length: {}", left_most_to_right_most_length);
                    
                    let segment_container: SegmentContainer<usize> = SegmentContainer::new(segments);
                    let permutations = segment_container.get_segment_location_permutations_within_bounding_length(left_most_to_right_most_length, 1);

                    for permutation in permutations.into_iter() {
                        for (located_segment_index, located_segment) in permutation.iter().enumerate() {
                            let location: (usize, usize);
                            
                            if is_horizontal {
                                location = (located_segment.position + left_most_length, width_or_height);
                            }
                            else {
                                location = (width_or_height, located_segment.position + left_most_length);
                            }

                            if !possible_locations_per_wall_index.contains_key(&located_segment.id) {
                                possible_locations_per_wall_index.insert(located_segment.id.clone(), HashSet::new());
                            }
                            possible_locations_per_wall_index.get_mut(&located_segment.id).unwrap().insert(location.clone());

                            for (other_located_segment_index, other_located_segment) in permutation.iter().enumerate() {
                                if located_segment_index != other_located_segment_index {
                                    let other_location:(usize, usize);
                                    
                                    if is_horizontal {
                                        other_location = (other_located_segment.position + left_most_length, width_or_height);
                                    }
                                    else {
                                        other_location = (width_or_height, other_located_segment.position + left_most_length);
                                    }

                                    if !other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index.contains_key(&located_segment.id) {
                                        other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index.insert(located_segment.id.clone(), HashMap::new());
                                    }
                                    if !other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index.get(&located_segment.id).unwrap().contains_key(&location) {
                                        other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index.get_mut(&located_segment.id).unwrap().insert(location.clone(), HashMap::new());
                                    }
                                    if !other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index.get(&located_segment.id).unwrap().get(&location).unwrap().contains_key(&other_located_segment.id) {
                                        other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index.get_mut(&located_segment.id).unwrap().get_mut(&location).unwrap().insert(other_located_segment.id.clone(), Vec::new());
                                    }
                                    other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index.get_mut(&located_segment.id).unwrap().get_mut(&location).unwrap().get_mut(&other_located_segment.id).unwrap().push(other_location);
                                }
                            }
                        }
                    }
                }
            }
        }

        debug!("possible_locations_per_wall_index: {:?}", possible_locations_per_wall_index);
        debug!("other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index: {:?}", other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index);

        // collect PlacedPlacableCollection instances representing the wall-adjacents

        // determine which wall(s) are adjacent to every wall-adjacent

        // determine the possible locations of every wall-adjectent along with which locations the walls can and cannot be

        // collect PlacedPlacableCollection instances representing the floaters

        // determine the possible locations of every floater compared to every wall-adjacent

        // determine the subset of possible locations of every floater compared to every other floater



        let wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        let mut rng = rand::thread_rng();
        let random_seed = Some(rng.gen::<u64>());

        let mut collapsable_wave_function = wave_function.get_collapsable_wave_function::<EntropicCollapsableWaveFunction<NodeState>>(random_seed);

        let collapsed_wave_function = collapsable_wave_function.collapse().unwrap();

        //println!("collapsed wave function: {:?}", collapsed_wave_function.node_state_per_node);

        /*let mut node_state_per_height_index_per_width_index: HashMap<usize, HashMap<usize, Option<NodeState>>> = HashMap::new();
        for width_index in 0..self.width as usize {
            let mut node_state_per_height_index: HashMap<usize, Option<NodeState>> = HashMap::new();
            for height_index in 0..self.height as usize {
                node_state_per_height_index.insert(height_index, None);
            }
            node_state_per_height_index_per_width_index.insert(width_index, node_state_per_height_index);
        }

        for (node_id, node_state) in collapsed_wave_function.node_state_per_node.into_iter() {
            let node_id_split = node_id.split("_").collect::<Vec<&str>>();
            let node_width_index = node_id_split[node_id_split.len() - 2].parse::<usize>().unwrap();
            let node_height_index = node_id_split[node_id_split.len() - 1].parse::<usize>().unwrap();
            node_state_per_height_index_per_width_index.get_mut(&node_width_index).unwrap().insert(node_height_index, Some(node_state));
        }

        let mut pixels: Vec<Vec<[u8; 4]>> = Vec::new();
        for _ in 0..self.width {
            let mut vec = Vec::new();
            for _ in 0..self.height {
                vec.push([255, 255, 255, 255]);
            }
            pixels.push(vec);
        }

        for width_index in 1..=(self.width - 2) as usize {
            for height_index in 1..=(self.height - 2) as usize {
                let node_state = node_state_per_height_index_per_width_index.get(&width_index).unwrap().get(&height_index).unwrap().as_ref().unwrap();
                
                if width_index == (self.width - 2) as usize || height_index == (self.height - 2) as usize {
                    for pixel_height_index in 0..3 as usize {
                        for pixel_width_index in 0..3 as usize {
                            pixels[width_index + pixel_width_index - 1][height_index + pixel_height_index - 1] = node_state.get_color(pixel_width_index, pixel_height_index);
                        }
                    }
                }
                else {
                    pixels[width_index - 1][height_index - 1] = node_state.get_color(0, 0);
                }
            }
        }

        for height_index in 0..self.height as usize {
            for width_index in 0..self.width as usize {
                let color = pixels[width_index][height_index];
                print_pixel(&color);
            }
            println!("");
        }*/

        todo!();
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    let level = Level::default();
    level.print();

    let similar_level = level.get_similar_level();
}