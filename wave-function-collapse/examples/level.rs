use std::collections::{VecDeque, HashMap, HashSet};
use austinhellerrepo_common_rust::{segment_container::{Segment, SegmentContainer}, index_incrementer::IndexIncrementer};
use colored::Colorize;
use log::debug;
use rand::Rng;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{Node, NodeStateCollection, WaveFunction, collapsable_wave_function::{entropic_collapsable_wave_function::EntropicCollapsableWaveFunction, collapsable_wave_function::CollapsableWaveFunction, accommodating_collapsable_wave_function::AccommodatingCollapsableWaveFunction}, AnonymousNodeStateCollection, NodeStateProbability};
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

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
enum ComponentType {
    Wall,
    WallAdjacent,
    Floater
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
struct NodeIdentifier {
    component_type: ComponentType,
    index: usize
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
struct NodeStateCollectionIdentifier {
    uuid: String
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
enum Identifier {
    Node(NodeIdentifier),
    NodeStateCollection(NodeStateCollectionIdentifier)
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
    fn get_wave_function(&self) -> WaveFunction<Identifier, NodeState> {

        // cache useful data

        let mut placed_placable_per_location: HashMap<(usize, usize), &PlacedPlacable> = HashMap::new();

        {
            for placed_placable in self.placed_placables.iter() {
                placed_placable_per_location.insert(placed_placable.location, placed_placable);
            }
        }

        // collect PlacedPlacableCollection instances representing the walls

        let mut top_walls: Vec<PlacedPlacableCollection> = Vec::new();
        let mut right_walls: Vec<PlacedPlacableCollection> = Vec::new();
        let mut bottom_walls: Vec<PlacedPlacableCollection> = Vec::new();
        let mut left_walls: Vec<PlacedPlacableCollection> = Vec::new();

        {
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
        }

        /*let wall_node_ids: Vec<String> = Vec::new();
        for wall in walls.iter() {
            wall_node_ids.push(format!("wall_node_{}_{}", wall.placed_placables[0].location.0, wall.placed_placables[0].location.1));
        }*/

        // determine possible locations per wall

        let mut top_wall_indexes: Vec<usize> = Vec::new();
        let mut right_wall_indexes: Vec<usize> = Vec::new();
        let mut bottom_wall_indexes: Vec<usize> = Vec::new();
        let mut left_wall_indexes: Vec<usize> = Vec::new();
        let mut walls: Vec<PlacedPlacableCollection> = Vec::new();
        // This contains the possible locations that each wall could exist at
        let mut raw_locations_per_wall_index: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
        // This contains the permutation-based walls and the possible locations of other walls since they can move around each other and are dependent upon each other
        let mut other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index: HashMap<usize, HashMap<(usize, usize), HashMap<usize, Vec<(usize, usize)>>>> = HashMap::new();
        
        {
            let mut current_wall_index: usize = 0;

            // iterate over each wall
            for ((edge_walls, is_horizontal), width_or_height) in zip(zip([top_walls.clone(), bottom_walls.clone(), left_walls.clone(), right_walls.clone()], [true, true, false, false]), [0, self.height - 1, 0, self.width - 1]) {
                debug!("trying edge_walls {:?} which are located at {}", edge_walls, width_or_height);

                if is_horizontal {
                    if width_or_height == 0 {
                        top_wall_indexes.push(current_wall_index);
                    }
                    else {
                        bottom_wall_indexes.push(current_wall_index);
                    }
                }
                else {
                    if width_or_height == 0 {
                        left_wall_indexes.push(current_wall_index);
                    }
                    else {
                        right_wall_indexes.push(current_wall_index);
                    }
                }

                if !edge_walls.is_empty() {

                    let mut segments: Vec<Segment<usize>> = Vec::new();

                    for wall in edge_walls.iter() {

                        debug!("examining wall index {} as {:?}", current_wall_index, wall);
                        walls.push(wall.clone());

                        // ensure that this wall is not stuck to another wall
                        if wall.placed_placables[0].location.0 == 0 && is_horizontal {
                            // the wall is stuck in the top-left or bottom-left corner
                            debug!("found wall is either in top-left or bottom-left corner");

                            let mut possible_locations: Vec<(usize, usize)> = Vec::new();
                            possible_locations.push(wall.placed_placables[0].location);
                            raw_locations_per_wall_index.insert(current_wall_index, possible_locations);
                        }
                        else if wall.placed_placables[0].location.1 == 0 && !is_horizontal {
                            // the wall is stuck in the top-left or top-right corner
                            debug!("found wall is either in top-left or top-right corner");

                            let mut possible_locations: Vec<(usize, usize)> = Vec::new();
                            possible_locations.push(wall.placed_placables[0].location);
                            raw_locations_per_wall_index.insert(current_wall_index, possible_locations);
                        }
                        else if wall.placed_placables[wall.placed_placables.len() - 1].location.0 == self.width - 1 && is_horizontal {
                            // the wall is stuck in the top-right or bottom-right corner
                            debug!("found wall is either in top-right or bottom-right corner");

                            let mut possible_locations: Vec<(usize, usize)> = Vec::new();
                            possible_locations.push(wall.placed_placables[0].location);
                            raw_locations_per_wall_index.insert(current_wall_index, possible_locations);
                        }
                        else if wall.placed_placables[wall.placed_placables.len() - 1].location.1 == self.height - 1 && !is_horizontal {
                            // the wall is stuck in the bottom-left or bottom-right corner
                            debug!("found wall is either in bottom-left or bottom-right corner");

                            let mut possible_locations: Vec<(usize, usize)> = Vec::new();
                            possible_locations.push(wall.placed_placables[0].location);
                            raw_locations_per_wall_index.insert(current_wall_index, possible_locations);
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
                        if edge_walls[0].placed_placables[0].location.0 == 0 && is_horizontal {
                            left_most_length = edge_walls[0].placed_placables[edge_walls[0].placed_placables.len() - 1].location.0 + 2;  // +2 spaces away is the next valid location
                        }
                        else if edge_walls[0].placed_placables[0].location.1 == 0 && !is_horizontal {
                            left_most_length = edge_walls[0].placed_placables[edge_walls[0].placed_placables.len() - 1].location.1 + 2;  // +2 spaces away is the next valid location
                        }
                        else {
                            left_most_length = 1;  // 1 space away from 0 is the next valid location
                        }
                        
                        let right_most_length: usize;
                        if is_horizontal {
                            if edge_walls[edge_walls.len() - 1].placed_placables[edge_walls[edge_walls.len() - 1].placed_placables.len() - 1].location.0 == self.width - 1 {
                                right_most_length = edge_walls[edge_walls.len() - 1].placed_placables[0].location.0 - 2;  // -2 spaces away to the left is the next valid location
                            }
                            else {
                                right_most_length = self.width - 2;  // 1 space to the left of the last index is the next valid location
                            }
                        }
                        else {
                            if edge_walls[edge_walls.len() - 1].placed_placables[edge_walls[edge_walls.len() - 1].placed_placables.len() - 1].location.1 == self.height - 1 {
                                right_most_length = edge_walls[edge_walls.len() - 1].placed_placables[0].location.1 - 2;  // -2 spaces away to the left is the next valid location
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

                                if !raw_locations_per_wall_index.contains_key(&located_segment.id) {
                                    raw_locations_per_wall_index.insert(located_segment.id.clone(), Vec::new());
                                }
                                if !raw_locations_per_wall_index.get(&located_segment.id).unwrap().contains(&location) {
                                    raw_locations_per_wall_index.get_mut(&located_segment.id).unwrap().push(location.clone());
                                }

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

            debug!("raw_locations_per_wall_index: {:?}", raw_locations_per_wall_index);
            debug!("other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index: {:?}", other_wall_possible_locations_per_other_wall_index_per_location_per_wall_index);
        }

        // collect PlacedPlacableCollection instances representing the wall-adjacents
        
        let mut wall_adjacents: Vec<PlacedPlacableCollection> = Vec::new();

        {
            let mut traveled_locations: HashSet<(usize, usize)> = HashSet::new();
            for height_index in [1, self.height - 2] {
                for width_index in 1..(self.width - 2) {
                    let location = (width_index, height_index);
                    if placed_placable_per_location.contains_key(&location) && !traveled_locations.contains(&location) {

                        let mut is_next_to_at_least_one_wall: bool = false;
                        let mut wall_adjacent: PlacedPlacableCollection = PlacedPlacableCollection::default();
                        let mut wall_adjacent_locations: VecDeque<(usize, usize)> = VecDeque::new();
                        wall_adjacent_locations.push_back(location);

                        while !wall_adjacent_locations.is_empty() {
                            let location = wall_adjacent_locations.pop_front().unwrap();

                            if location.0 != 1 {
                                let left_location = (location.0 - 1, location.1);
                                if placed_placable_per_location.contains_key(&left_location) && !traveled_locations.contains(&left_location) {
                                    wall_adjacent_locations.push_back(left_location);
                                }
                            }
                            else {
                                let potential_wall_location = (location.0 - 1, location.1);
                                'search_walls: {
                                    for wall in left_walls.iter() {
                                        for placed_placable in wall.placed_placables.iter() {
                                            if placed_placable.location == potential_wall_location {
                                                is_next_to_at_least_one_wall = true;
                                                break 'search_walls;
                                            }
                                        }
                                    }
                                }
                            }
                            if location.1 != 1 {
                                let top_location = (location.0, location.1 - 1);
                                if placed_placable_per_location.contains_key(&top_location) && !traveled_locations.contains(&top_location) {
                                    wall_adjacent_locations.push_back(top_location);
                                }
                            }
                            else {
                                let potential_wall_location = (location.0, location.1 - 1);
                                'search_walls: {
                                    for wall in top_walls.iter() {
                                        for placed_placable in wall.placed_placables.iter() {
                                            if placed_placable.location == potential_wall_location {
                                                is_next_to_at_least_one_wall = true;
                                                break 'search_walls;
                                            }
                                        }
                                    }
                                }
                            }
                            if location.0 != self.width - 2 {
                                let right_location = (location.0 + 1, location.1);
                                if placed_placable_per_location.contains_key(&right_location) && !traveled_locations.contains(&right_location) {
                                    wall_adjacent_locations.push_back(right_location);
                                }
                            }
                            else {
                                let potential_wall_location = (location.0 + 1, location.1);
                                'search_walls: {
                                    for wall in right_walls.iter() {
                                        for placed_placable in wall.placed_placables.iter() {
                                            if placed_placable.location == potential_wall_location {
                                                is_next_to_at_least_one_wall = true;
                                                break 'search_walls;
                                            }
                                        }
                                    }
                                }
                            }
                            if location.1 != self.height - 2 {
                                let bottom_location = (location.0, location.1 + 1);
                                if placed_placable_per_location.contains_key(&bottom_location) && !traveled_locations.contains(&bottom_location) {
                                    wall_adjacent_locations.push_back(bottom_location);
                                }
                            }
                            else {
                                let potential_wall_location = (location.0, location.1 + 1);
                                'search_walls: {
                                    for wall in bottom_walls.iter() {
                                        for placed_placable in wall.placed_placables.iter() {
                                            if placed_placable.location == potential_wall_location {
                                                is_next_to_at_least_one_wall = true;
                                                break 'search_walls;
                                            }
                                        }
                                    }
                                }
                            }

                            wall_adjacent.placed_placables.push((*placed_placable_per_location.get(&location).unwrap()).clone());
                            traveled_locations.insert(location);
                        }

                        if is_next_to_at_least_one_wall {
                            wall_adjacents.push(wall_adjacent);
                        }
                    }
                }
            }
            for height_index in 2..(self.height - 3) {
                for width_index in [1, self.width - 2] {
                    let location = (width_index, height_index);
                    if placed_placable_per_location.contains_key(&location) && !traveled_locations.contains(&location) {

                        let mut is_next_to_at_least_one_wall: bool = false;
                        let mut wall_adjacent: PlacedPlacableCollection = PlacedPlacableCollection::default();
                        let mut wall_adjacent_locations: VecDeque<(usize, usize)> = VecDeque::new();
                        wall_adjacent_locations.push_back(location);

                        while !wall_adjacent_locations.is_empty() {
                            let location = wall_adjacent_locations.pop_front().unwrap();

                            if location.0 != 1 {
                                let left_location = (location.0 - 1, location.1);
                                if placed_placable_per_location.contains_key(&left_location) && !traveled_locations.contains(&left_location) {
                                    wall_adjacent_locations.push_back(left_location);
                                }
                            }
                            else {
                                let potential_wall_location = (location.0 - 1, location.1);
                                'search_walls: {
                                    for wall in left_walls.iter() {
                                        for placed_placable in wall.placed_placables.iter() {
                                            if placed_placable.location == potential_wall_location {
                                                is_next_to_at_least_one_wall = true;
                                                break 'search_walls;
                                            }
                                        }
                                    }
                                }
                            }
                            if location.1 != 1 {
                                let top_location = (location.0, location.1 - 1);
                                if placed_placable_per_location.contains_key(&top_location) && !traveled_locations.contains(&top_location) {
                                    wall_adjacent_locations.push_back(top_location);
                                }
                            }
                            else {
                                let potential_wall_location = (location.0, location.1 - 1);
                                'search_walls: {
                                    for wall in top_walls.iter() {
                                        for placed_placable in wall.placed_placables.iter() {
                                            if placed_placable.location == potential_wall_location {
                                                is_next_to_at_least_one_wall = true;
                                                break 'search_walls;
                                            }
                                        }
                                    }
                                }
                            }
                            if location.0 != self.width - 2 {
                                let right_location = (location.0 + 1, location.1);
                                if placed_placable_per_location.contains_key(&right_location) && !traveled_locations.contains(&right_location) {
                                    wall_adjacent_locations.push_back(right_location);
                                }
                            }
                            else {
                                let potential_wall_location = (location.0 + 1, location.1);
                                'search_walls: {
                                    for wall in right_walls.iter() {
                                        for placed_placable in wall.placed_placables.iter() {
                                            if placed_placable.location == potential_wall_location {
                                                is_next_to_at_least_one_wall = true;
                                                break 'search_walls;
                                            }
                                        }
                                    }
                                }
                            }
                            if location.1 != self.height - 2 {
                                let bottom_location = (location.0, location.1 + 1);
                                if placed_placable_per_location.contains_key(&bottom_location) && !traveled_locations.contains(&bottom_location) {
                                    wall_adjacent_locations.push_back(bottom_location);
                                }
                            }
                            else {
                                let potential_wall_location = (location.0, location.1 + 1);
                                'search_walls: {
                                    for wall in bottom_walls.iter() {
                                        for placed_placable in wall.placed_placables.iter() {
                                            if placed_placable.location == potential_wall_location {
                                                is_next_to_at_least_one_wall = true;
                                                break 'search_walls;
                                            }
                                        }
                                    }
                                }
                            }

                            wall_adjacent.placed_placables.push((*placed_placable_per_location.get(&location).unwrap()).clone());
                            traveled_locations.insert(location);
                        }

                        if is_next_to_at_least_one_wall {
                            wall_adjacents.push(wall_adjacent);
                        }
                    }
                }
            }

            debug!("wall adjacents: {:?}", wall_adjacents);
        }

        // determine which wall(s) are adjacent to every wall-adjacent

        let mut wall_indexes_per_wall_adjacent_index: HashMap<usize, HashSet<usize>> = HashMap::new();

        {
            // iterate over every wall-adjacent cell and find each wall it is adjacent to, capturing the wall index
            for (wall_adjacent_index, wall_adjacent) in wall_adjacents.iter().enumerate() {
                let mut wall_indexes: HashSet<usize> = HashSet::new();
                for wall_adjacent_placed_placable in wall_adjacent.placed_placables.iter() {
                    if wall_adjacent_placed_placable.location.0 == 1 {
                        'wall_search: {
                            for (wall_index, wall) in walls.iter().enumerate() {
                                for wall_placed_placable in wall.placed_placables.iter() {
                                    if wall_placed_placable.location.1 == wall_adjacent_placed_placable.location.1 &&
                                        wall_placed_placable.location.0 == wall_adjacent_placed_placable.location.0 - 1 {
                                        
                                        wall_indexes.insert(wall_index);

                                        // no other walls could be to the left of this wall-adjacent cell
                                        break 'wall_search;
                                    }
                                }
                            }
                        }
                    }
                    if wall_adjacent_placed_placable.location.1 == 1 {
                        'wall_search: {
                            for (wall_index, wall) in walls.iter().enumerate() {
                                for wall_placed_placable in wall.placed_placables.iter() {
                                    if wall_placed_placable.location.0 == wall_adjacent_placed_placable.location.0 &&
                                        wall_placed_placable.location.1 == wall_adjacent_placed_placable.location.1 - 1 {
                                        
                                        wall_indexes.insert(wall_index);

                                        // no other walls could be above this wall-adjacent cell
                                        break 'wall_search;
                                    }
                                }
                            }
                        }
                    }
                    if wall_adjacent_placed_placable.location.0 == self.width - 2 {
                        'wall_search: {
                            for (wall_index, wall) in walls.iter().enumerate() {
                                for wall_placed_placable in wall.placed_placables.iter() {
                                    if wall_placed_placable.location.1 == wall_adjacent_placed_placable.location.1 &&
                                        wall_placed_placable.location.0 == wall_adjacent_placed_placable.location.0 + 1 {
                                        
                                        wall_indexes.insert(wall_index);

                                        // no other walls could be to the right of this wall-adjacent cell
                                        break 'wall_search;
                                    }
                                }
                            }
                        }
                    }
                    if wall_adjacent_placed_placable.location.1 == self.height - 2 {
                        'wall_search: {
                            for (wall_index, wall) in walls.iter().enumerate() {
                                for wall_placed_placable in wall.placed_placables.iter() {
                                    if wall_placed_placable.location.0 == wall_adjacent_placed_placable.location.0 &&
                                        wall_placed_placable.location.1 == wall_adjacent_placed_placable.location.1 + 1 {
                                        
                                        wall_indexes.insert(wall_index);

                                        // no other walls could be below this wall-adjacent cell
                                        break 'wall_search;
                                    }
                                }
                            }
                        }
                    }
                }
                if wall_indexes.is_empty() {
                    panic!("Failed to find wall adjacent to wall-adjacent at index {}: {:?}", wall_adjacent_index, wall_adjacent);
                }
                wall_indexes_per_wall_adjacent_index.insert(wall_adjacent_index, wall_indexes);
            }

            debug!("wall_indexes_per_wall_adjacent_index: {:?}", wall_indexes_per_wall_adjacent_index);
        }

        // determine the possible locations of every wall-adjectent along with which locations the walls can and cannot be

        let mut wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index: HashMap<usize, HashMap<(usize, usize), HashMap<usize, HashSet<(usize, usize)>>>> = HashMap::new();
        let mut top_left_location_per_wall_adjacent_index: HashMap<usize, (usize, usize)> = HashMap::new();
        // the wall locations when the possible wall-adjacent locations are considered
        let mut filtered_locations_per_wall_index: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();

        {
            for (wall_adjacent_index, wall_adjacent) in wall_adjacents.iter().enumerate() {

                debug!("wall-adjacent: {}: {:?}", wall_adjacent_index, wall_adjacent);

                let mut is_top: bool = false;
                let mut is_bottom: bool = false;
                let mut is_left: bool = false;
                let mut is_right: bool = false;
                let mut top_left_location: (usize, usize) = (self.width, self.height);
                let mut bottom_right_location: (usize, usize) = (0, 0);

                for placed_placable in wall_adjacent.placed_placables.iter() {
                    if placed_placable.location.0 == 1 {
                        is_left = true;
                    }
                    if placed_placable.location.0 == self.width - 2 {
                        is_right = true;
                    }
                    if placed_placable.location.1 == 1 {
                        is_top = true;
                    }
                    if placed_placable.location.1 == self.height - 2 {
                        is_bottom = true;
                    }

                    if placed_placable.location.0 < top_left_location.0 {
                        top_left_location.0 = placed_placable.location.0;
                    }
                    if placed_placable.location.1 < top_left_location.1 {
                        top_left_location.1 = placed_placable.location.1;
                    }
                    if placed_placable.location.0 > bottom_right_location.0 {
                        bottom_right_location.0 = placed_placable.location.0;
                    }
                    if placed_placable.location.1 > bottom_right_location.1 {
                        bottom_right_location.1 = placed_placable.location.1;
                    }
                }

                top_left_location_per_wall_adjacent_index.insert(wall_adjacent_index, top_left_location);

                // determine how many walls this wall-adjacent is in direct contact with
                let touching_walls_total: usize = [is_left, is_right, is_top, is_bottom].into_iter().filter(|is_touching_wall| is_touching_wall.to_owned()).collect::<Vec<bool>>().len();

                debug!("touching_walls_total: {}", touching_walls_total);

                let mut wall_incrementer_and_wall_adjacent_tuples: Vec<(IndexIncrementer, HashMap<(usize, usize), Vec<(usize, usize)>>, Vec<usize>)> = Vec::new();

                if touching_walls_total == 1 {

                    let first_side_wall_indexes: &Vec<usize>;
                    let main_wall_indexes: &Vec<usize>;
                    let second_side_wall_indexes: &Vec<usize>;
                    let travel_location_delta: (usize, usize);
                    let main_wall_delta: (i8, i8);
                    let inclusive_origin: (usize, usize);
                    let inclusive_destination: (usize, usize);
                    let first_contact_corner: (usize, usize);
                    let second_contact_corner: (usize, usize);

                    // single-walled wall-adjacent
                    if is_left {
                        first_side_wall_indexes = &top_wall_indexes;
                        main_wall_indexes = &left_wall_indexes;
                        second_side_wall_indexes = &bottom_wall_indexes;
                        travel_location_delta = (0, 1);
                        main_wall_delta = (-1, 0);
                        inclusive_origin = (1, 1);
                        inclusive_destination = (1, self.height - 2 - (bottom_right_location.1 - top_left_location.1));
                        first_contact_corner = (1, 1);
                        second_contact_corner = (1, self.height - 2);
                    }
                    else if is_right {
                        first_side_wall_indexes = &top_wall_indexes;
                        main_wall_indexes = &right_wall_indexes;
                        second_side_wall_indexes = &bottom_wall_indexes;
                        travel_location_delta = (0, 1);
                        main_wall_delta = (1, 0);
                        inclusive_origin = (self.width - 2 - (bottom_right_location.0 - top_left_location.0), 1);
                        inclusive_destination = (self.width - 2 - (bottom_right_location.0 - top_left_location.0), self.height - 2 - (bottom_right_location.1 - top_left_location.1));
                        first_contact_corner = (self.width - 2, 1);
                        second_contact_corner = (self.width - 2, self.height - 2);
                    }
                    else if is_top {
                        first_side_wall_indexes = &left_wall_indexes;
                        main_wall_indexes = &top_wall_indexes;
                        second_side_wall_indexes = &right_wall_indexes;
                        travel_location_delta = (1, 0);
                        main_wall_delta = (0, -1);
                        inclusive_origin = (1, 1);
                        inclusive_destination = (self.width - 2 - (bottom_right_location.0 - top_left_location.0), 1);
                        first_contact_corner = (1, 1);
                        second_contact_corner = (self.width - 2, 1);
                    }
                    else if is_bottom {
                        first_side_wall_indexes = &left_wall_indexes;
                        main_wall_indexes = &bottom_wall_indexes;
                        second_side_wall_indexes = &right_wall_indexes;
                        travel_location_delta = (1, 0);
                        main_wall_delta = (0, 1);
                        inclusive_origin = (1, self.height - 2 - (bottom_right_location.1 - top_left_location.1));
                        inclusive_destination = (self.width - 2 - (bottom_right_location.0 - top_left_location.0), self.height - 2 - (bottom_right_location.1 - top_left_location.1));
                        first_contact_corner = (1, self.height - 2);
                        second_contact_corner = (self.width - 2, self.height - 2);
                    }
                    else {
                        panic!("Unexpected lack of walls for wall-adjacent at index {}: {:?}", wall_adjacent_index, wall_adjacent);
                    }
                    
                    // first keep the wall adjacent in contact with the first_side_wall and main_wall
                    //      iterate over every possible location for each wall in the first_side_wall and main_wall
                    // begin iterating over every location from second to near-last along main_wall
                    //      iterate over every possible location for each wall in the main_wall
                    // last keep the wall adjacent in contact with the second_side_wall and main_wall
                    //      iterate over every possible location for each wall in the second_side_wall and main_wall

                    let first_possible_wall_location_index_incrementer: IndexIncrementer;
                    let main_possible_wall_location_index_incrementer: IndexIncrementer;
                    let second_possible_wall_location_index_incrementer: IndexIncrementer;
                    let first_wall_indexes: Vec<usize> = first_side_wall_indexes.iter().cloned().chain(main_wall_indexes.iter().cloned()).collect();
                    let main_wall_indexes: Vec<usize> = main_wall_indexes.clone();
                    let second_wall_indexes: Vec<usize> = second_side_wall_indexes.iter().cloned().chain(main_wall_indexes.iter().cloned()).collect();
                    
                    debug!("initializing first possible wall location index incrementer");

                    // initialize first possible wall location index incrementer
                    {
                        let mut maximum_exclusive_possible_wall_location_indexes: Vec<usize> = Vec::new();
                        for wall_index in first_wall_indexes.iter() {
                            let maximum_exclusive_possible_wall_location_index = raw_locations_per_wall_index.get(wall_index).unwrap().len();
                            maximum_exclusive_possible_wall_location_indexes.push(maximum_exclusive_possible_wall_location_index);
                        }
                        first_possible_wall_location_index_incrementer = IndexIncrementer::new(maximum_exclusive_possible_wall_location_indexes);
                    }

                    debug!("initialized first possible wall location index incrementer");

                    debug!("initializing main possible wall location index incrementer");

                    // initialize main possible wall location index incrementer
                    {
                        let mut maximum_exclusive_possible_wall_location_indexes: Vec<usize> = Vec::new();
                        for wall_index in main_wall_indexes.iter() {
                            let maximum_exclusive_possible_wall_location_index = raw_locations_per_wall_index.get(wall_index).unwrap().len();
                            maximum_exclusive_possible_wall_location_indexes.push(maximum_exclusive_possible_wall_location_index);
                        }
                        main_possible_wall_location_index_incrementer = IndexIncrementer::new(maximum_exclusive_possible_wall_location_indexes);
                    }

                    debug!("initialized main possible wall location index incrementer");

                    debug!("initializing second possible wall location index incrementer");

                    // initialize second possible wall location index incrementer
                    {
                        let mut maximum_exclusive_possible_wall_location_indexes: Vec<usize> = Vec::new();
                        for wall_index in second_wall_indexes.iter() {
                            let maximum_exclusive_possible_wall_location_index = raw_locations_per_wall_index.get(wall_index).unwrap().len();
                            maximum_exclusive_possible_wall_location_indexes.push(maximum_exclusive_possible_wall_location_index);
                        }
                        second_possible_wall_location_index_incrementer = IndexIncrementer::new(maximum_exclusive_possible_wall_location_indexes);
                    }

                    debug!("initialized second possible wall location index incrementer");

                    debug!("collecting first placed_placable locations that would overlap with a nearby wall");

                    let mut first_applicable_wall_adjacent_detection_locations_per_location: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

                    // collect the first placed_placable locations that would overlap with a wall as it exists in the origin
                    {
                        let mut first_applicable_wall_adjacent_detection_locations: Vec<(usize, usize)> = Vec::new();
                        for placed_placable in wall_adjacent.placed_placables.iter() {
                            let calculated_placed_placable_location: (usize, usize) = (placed_placable.location.0 - top_left_location.0 + inclusive_origin.0, placed_placable.location.1 - top_left_location.1 + inclusive_origin.1);

                            if calculated_placed_placable_location.0 == first_contact_corner.0 {
                                // move from the current direction into the direction of the wall (if this axis is main wall adjacent) or into the direction of the first wall (if this axis is first wall adjacent)
                                let detection_location: (usize, usize) = ((calculated_placed_placable_location.0 as i8 + main_wall_delta.0 - travel_location_delta.0 as i8) as usize, calculated_placed_placable_location.1);
                                first_applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                            if calculated_placed_placable_location.1 == first_contact_corner.1 {
                                let detection_location: (usize, usize) = (calculated_placed_placable_location.0, (calculated_placed_placable_location.1 as i8 + main_wall_delta.1 - travel_location_delta.1 as i8) as usize);
                                first_applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                        }
                        first_applicable_wall_adjacent_detection_locations_per_location.insert(inclusive_origin, first_applicable_wall_adjacent_detection_locations);
                    }

                    debug!("collected first placed_placable locations that would overlap with a nearby wall");

                    debug!("collecting main placed_placable locations that would overlap with a nearby wall");

                    let mut main_applicable_wall_adjacent_detection_locations_per_location: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

                    // collect the main placed_placable locations that would overlap with a wall as it moves away from the origin to the destination
                    {
                        let mut current_travel_location: (usize, usize) = (inclusive_origin.0 + travel_location_delta.0, inclusive_origin.1 + travel_location_delta.1);
                        while current_travel_location != inclusive_destination {
                            main_applicable_wall_adjacent_detection_locations_per_location.insert(current_travel_location, Vec::new());
                            for placed_placable in wall_adjacent.placed_placables.iter() {
                                let calculated_placed_placable_location: (usize, usize) = (placed_placable.location.0 - top_left_location.0 + current_travel_location.0, placed_placable.location.1 - top_left_location.1 + current_travel_location.1);

                                // TODO determine how to calculate the main wall detection locations
                                if calculated_placed_placable_location.0 == first_contact_corner.0 ||
                                    calculated_placed_placable_location.1 == first_contact_corner.1 {
                                    
                                    let detection_location: (usize, usize) = ((calculated_placed_placable_location.0 as i8 + main_wall_delta.0) as usize, (calculated_placed_placable_location.1 as i8 + main_wall_delta.1) as usize);
                                    main_applicable_wall_adjacent_detection_locations_per_location.get_mut(&current_travel_location).unwrap().push(detection_location);
                                }
                            }
                            current_travel_location.0 += travel_location_delta.0;
                            current_travel_location.1 += travel_location_delta.1;
                        }
                    }

                    debug!("collected main placed_placable locations that would overlap with a nearby wall");

                    debug!("collecting second placed_placable locations that would overlap with a nearby wall");

                    let mut second_applicable_wall_adjacent_detection_locations_per_location: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

                    // collect the second placed_placable locations that would overlap with a wall as it exists in the destination
                    {
                        let mut second_applicable_wall_adjacent_detection_locations: Vec<(usize, usize)> = Vec::new();
                        for placed_placable in wall_adjacent.placed_placables.iter() {
                            let calculated_placed_placable_location: (usize, usize) = (placed_placable.location.0 - top_left_location.0 + inclusive_destination.0, placed_placable.location.1 - top_left_location.1 + inclusive_destination.1);

                            if calculated_placed_placable_location.0 == second_contact_corner.0 {
                                // move from the current direction into the direction of the wall (if this axis is main wall adjacent) or into the direction of the second wall (if this axis is second wall adjacent)
                                let detection_location: (usize, usize) = ((calculated_placed_placable_location.0 as i8 + main_wall_delta.0 + travel_location_delta.0 as i8) as usize, calculated_placed_placable_location.1);
                                second_applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                            if calculated_placed_placable_location.1 == second_contact_corner.1 {
                                let detection_location: (usize, usize) = (calculated_placed_placable_location.0, (calculated_placed_placable_location.1 as i8 + main_wall_delta.1 + travel_location_delta.1 as i8) as usize);
                                second_applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                        }
                        second_applicable_wall_adjacent_detection_locations_per_location.insert(inclusive_destination, second_applicable_wall_adjacent_detection_locations);
                    }

                    debug!("collected second placed_placable locations that would overlap with a nearby wall");

                    wall_incrementer_and_wall_adjacent_tuples.push((first_possible_wall_location_index_incrementer, first_applicable_wall_adjacent_detection_locations_per_location, first_wall_indexes));
                    wall_incrementer_and_wall_adjacent_tuples.push((main_possible_wall_location_index_incrementer, main_applicable_wall_adjacent_detection_locations_per_location, main_wall_indexes));
                    wall_incrementer_and_wall_adjacent_tuples.push((second_possible_wall_location_index_incrementer, second_applicable_wall_adjacent_detection_locations_per_location, second_wall_indexes));
                }
                else if touching_walls_total == 2 {

                    if is_left && is_right || is_top && is_bottom {
                        // the wall-adjacent stretches across the level

                        todo!();
                    }
                    else {
                        // the wall-adjacent is stuck in the corner

                        let first_side_wall_indexes: &Vec<usize>;
                        let second_side_wall_indexes: &Vec<usize>;
                        let origin: (usize, usize);
                        let contact_corner: (usize, usize);
                        let contact_corner_delta: (i8, i8);

                        // single-walled wall-adjacent
                        if is_left {

                            first_side_wall_indexes = &left_wall_indexes;

                            if is_top {
                                second_side_wall_indexes = &top_wall_indexes;
                                origin = (1, 1);
                                contact_corner = (1, 1);
                                contact_corner_delta = (-1, -1);
                            }
                            else if is_bottom {
                                second_side_wall_indexes = &bottom_wall_indexes;
                                origin = (1, self.height - 2 - (bottom_right_location.1 - top_left_location.1));
                                contact_corner = (1, self.height - 2);
                                contact_corner_delta = (-1, 1);
                            }
                            else {
                                panic!("Unexpected lack of edge-adjacent contact.");
                            }
                        }
                        else if is_right {

                            first_side_wall_indexes = &right_wall_indexes;

                            if is_top {
                                second_side_wall_indexes = &top_wall_indexes;
                                origin = (self.width - 2 - (bottom_right_location.0 - top_left_location.0), 1);
                                contact_corner = (self.width - 2, 1);
                                contact_corner_delta = (1, -1);
                            }
                            else if is_bottom {
                                second_side_wall_indexes = &bottom_wall_indexes;
                                origin = (self.width - 2 - (bottom_right_location.0 - top_left_location.0), self.height - 2 - (bottom_right_location.1 - top_left_location.1));
                                contact_corner = (self.width - 2, self.height - 2);
                                contact_corner_delta = (1, 1);
                            }
                            else {
                                panic!("Unexpected lack of edge-adjacent contact.");
                            }
                        }
                        else {
                            panic!("Unexpected lack of walls for wall-adjacent at index {}: {:?}", wall_adjacent_index, wall_adjacent);
                        }
                        
                        // keep the wall adjacent in contact with the first_side_wall and second_side_wall
                        //      iterate over every possible location for each wall in the first_side_wall and second_side_wall

                        let possible_wall_location_index_incrementer: IndexIncrementer;
                        let wall_indexes: Vec<usize> = first_side_wall_indexes.iter().cloned().chain(second_side_wall_indexes.iter().cloned()).collect();

                        // initialize possible wall location index incrementer
                        {
                            let mut maximum_exclusive_possible_wall_location_indexes: Vec<usize> = Vec::new();
                            for wall_index in wall_indexes.iter() {
                                let maximum_exclusive_possible_wall_location_index = raw_locations_per_wall_index.get(wall_index).unwrap().len();
                                maximum_exclusive_possible_wall_location_indexes.push(maximum_exclusive_possible_wall_location_index);
                            }
                            possible_wall_location_index_incrementer = IndexIncrementer::new(maximum_exclusive_possible_wall_location_indexes);
                        }

                        let mut applicable_wall_adjacent_detection_locations_per_location: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

                        // collect the placed_placable locations that would overlap with a wall as it exists in the contact corner
                        {
                            let mut applicable_wall_adjacent_detection_locations: Vec<(usize, usize)> = Vec::new();
                            for placed_placable in wall_adjacent.placed_placables.iter() {
                                let calculated_placed_placable_location: (usize, usize) = (placed_placable.location.0 - top_left_location.0 + origin.0, placed_placable.location.1 - top_left_location.1 + origin.1);

                                if calculated_placed_placable_location.0 == contact_corner.0 {
                                    // move from the current direction into the direction of the wall (if this axis is main wall adjacent) or into the direction of the first wall (if this axis is first wall adjacent)
                                    let detection_location: (usize, usize) = ((calculated_placed_placable_location.0 as i8 + contact_corner_delta.0) as usize, calculated_placed_placable_location.1);
                                    applicable_wall_adjacent_detection_locations.push(detection_location);
                                }
                                if calculated_placed_placable_location.1 == contact_corner.1 {
                                    let detection_location: (usize, usize) = (calculated_placed_placable_location.0, (calculated_placed_placable_location.1 as i8 + contact_corner_delta.1) as usize);
                                    applicable_wall_adjacent_detection_locations.push(detection_location);
                                }
                            }
                            applicable_wall_adjacent_detection_locations_per_location.insert(origin, applicable_wall_adjacent_detection_locations);
                        }

                        wall_incrementer_and_wall_adjacent_tuples.push((possible_wall_location_index_incrementer, applicable_wall_adjacent_detection_locations_per_location, wall_indexes));
                    }
                }
                else if touching_walls_total == 3 {

                    // the wall-adjacent is stuck in both corners

                    let left_side_wall_indexes: &Vec<usize>;
                    let middle_side_wall_indexes: &Vec<usize>;
                    let right_side_wall_indexes: &Vec<usize>;
                    let origin: (usize, usize);
                    let left_middle_contact_corner: (usize, usize);
                    let left_middle_contact_corner_delta: (i8, i8);
                    let middle_right_contact_corner: (usize, usize);
                    let middle_right_contact_corner_delta: (i8, i8);

                    // single-walled wall-adjacent
                    if !is_left {

                        left_side_wall_indexes = &top_wall_indexes;
                        middle_side_wall_indexes = &right_wall_indexes;
                        right_side_wall_indexes = &bottom_wall_indexes;
                        origin = (self.width - 2 - (bottom_right_location.0 - top_left_location.0), 1);
                        left_middle_contact_corner = (self.width - 2, 1);
                        left_middle_contact_corner_delta = (1, -1);
                        middle_right_contact_corner = (self.width - 2, self.height - 2);
                        middle_right_contact_corner_delta = (1, 1);
                    }
                    else if !is_top {

                        left_side_wall_indexes = &left_wall_indexes;
                        middle_side_wall_indexes = &bottom_wall_indexes;
                        right_side_wall_indexes = &right_wall_indexes;
                        origin = (1, self.height - 2 - (bottom_right_location.1 - top_left_location.1));
                        left_middle_contact_corner = (1, self.height - 2);
                        left_middle_contact_corner_delta = (-1, 1);
                        middle_right_contact_corner = (self.width - 2, self.height - 2);
                        middle_right_contact_corner_delta = (1, 1);
                    }
                    else if !is_right {

                        left_side_wall_indexes = &top_wall_indexes;
                        middle_side_wall_indexes = &left_side_wall_indexes;
                        right_side_wall_indexes = &bottom_wall_indexes;
                        origin = (1, 1);
                        left_middle_contact_corner = (1, 1);
                        left_middle_contact_corner_delta = (-1, -1);
                        middle_right_contact_corner = (1, self.height - 2);
                        middle_right_contact_corner_delta = (-1, 1);
                    }
                    else if !is_bottom {

                        left_side_wall_indexes = &left_wall_indexes;
                        middle_side_wall_indexes = &top_wall_indexes;
                        right_side_wall_indexes = &right_wall_indexes;
                        origin = (1, 1);
                        left_middle_contact_corner = (1, 1);
                        left_middle_contact_corner_delta = (-1, -1);
                        middle_right_contact_corner = (self.width - 2, 1);
                        middle_right_contact_corner_delta = (1, -1);
                    }
                    else {
                        panic!("Unexpected lack of walls for wall-adjacent at index {}: {:?}", wall_adjacent_index, wall_adjacent);
                    }
                    
                    // keep the wall adjacent in contact with the first_side_wall and second_side_wall
                    //      iterate over every possible location for each wall in the first_side_wall and second_side_wall

                    let possible_wall_location_index_incrementer: IndexIncrementer;
                    let wall_indexes: Vec<usize> = left_side_wall_indexes.iter().cloned().chain(middle_side_wall_indexes.iter().cloned().chain(right_side_wall_indexes.iter().cloned())).collect();

                    // initialize possible wall location index incrementer
                    {
                        let mut maximum_exclusive_possible_wall_location_indexes: Vec<usize> = Vec::new();
                        for wall_index in wall_indexes.iter() {
                            let maximum_exclusive_possible_wall_location_index = raw_locations_per_wall_index.get(wall_index).unwrap().len();
                            maximum_exclusive_possible_wall_location_indexes.push(maximum_exclusive_possible_wall_location_index);
                        }
                        possible_wall_location_index_incrementer = IndexIncrementer::new(maximum_exclusive_possible_wall_location_indexes);
                    }

                    let mut applicable_wall_adjacent_detection_locations_per_location: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

                    // collect the placed_placable locations that would overlap with a wall as it exists in both contact corners
                    {
                        let mut applicable_wall_adjacent_detection_locations: Vec<(usize, usize)> = Vec::new();
                        for placed_placable in wall_adjacent.placed_placables.iter() {
                            let calculated_placed_placable_location: (usize, usize) = (placed_placable.location.0 - top_left_location.0 + origin.0, placed_placable.location.1 - top_left_location.1 + origin.1);

                            if calculated_placed_placable_location.0 == left_middle_contact_corner.0 {
                                // move from the current direction into the direction of the wall (if this axis is main wall adjacent) or into the direction of the first wall (if this axis is first wall adjacent)
                                let detection_location: (usize, usize) = ((calculated_placed_placable_location.0 as i8 + left_middle_contact_corner_delta.0) as usize, calculated_placed_placable_location.1);
                                applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                            if calculated_placed_placable_location.1 == left_middle_contact_corner.1 {
                                let detection_location: (usize, usize) = (calculated_placed_placable_location.0, (calculated_placed_placable_location.1 as i8 + left_middle_contact_corner_delta.1) as usize);
                                applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                            if calculated_placed_placable_location.0 == middle_right_contact_corner.0 && left_middle_contact_corner.0 != middle_right_contact_corner.0 {
                                // move from the current direction into the direction of the wall (if this axis is main wall adjacent) or into the direction of the first wall (if this axis is first wall adjacent)
                                let detection_location: (usize, usize) = ((calculated_placed_placable_location.0 as i8 + middle_right_contact_corner_delta.0) as usize, calculated_placed_placable_location.1);
                                applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                            if calculated_placed_placable_location.1 == middle_right_contact_corner.1 && left_middle_contact_corner.1 != middle_right_contact_corner.1 {
                                let detection_location: (usize, usize) = (calculated_placed_placable_location.0, (calculated_placed_placable_location.1 as i8 + middle_right_contact_corner_delta.1) as usize);
                                applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                        }
                        applicable_wall_adjacent_detection_locations_per_location.insert(origin, applicable_wall_adjacent_detection_locations);
                    }

                    wall_incrementer_and_wall_adjacent_tuples.push((possible_wall_location_index_incrementer, applicable_wall_adjacent_detection_locations_per_location, wall_indexes));
                }
                else if touching_walls_total == 4 {

                    // the wall-adjacent is stuck in all four corners
                    let origin: (usize, usize) = (1, 1);
                    let top_left_contact_corner: (usize, usize) = (1, 1);
                    let top_left_contact_corner_delta: (i8, i8) = (-1, -1);
                    let bottom_right_contact_corner: (usize, usize) = (self.width - 2, self.height - 2);
                    let bottom_right_contact_corner_delta: (i8, i8) = (1, 1);

                    // keep the wall adjacent in contact with all four walls, so exactly where it is currently located
                    //      iterate over every possible location for each wall

                    let possible_wall_location_index_incrementer: IndexIncrementer;
                    let wall_indexes: Vec<usize> = left_wall_indexes.iter().cloned().chain(top_wall_indexes.iter().cloned().chain(right_wall_indexes.iter().cloned().chain(bottom_wall_indexes.iter().cloned()))).collect();

                    // initialize possible wall location index incrementer
                    {
                        let mut maximum_exclusive_possible_wall_location_indexes: Vec<usize> = Vec::new();
                        for wall_index in wall_indexes.iter() {
                            let maximum_exclusive_possible_wall_location_index = raw_locations_per_wall_index.get(wall_index).unwrap().len();
                            maximum_exclusive_possible_wall_location_indexes.push(maximum_exclusive_possible_wall_location_index);
                        }
                        possible_wall_location_index_incrementer = IndexIncrementer::new(maximum_exclusive_possible_wall_location_indexes);
                    }

                    let mut applicable_wall_adjacent_detection_locations_per_location: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

                    // collect the placed_placable locations that would overlap with a wall as it exists in the contact corner
                    {
                        let mut applicable_wall_adjacent_detection_locations: Vec<(usize, usize)> = Vec::new();
                        for placed_placable in wall_adjacent.placed_placables.iter() {
                            let calculated_placed_placable_location: (usize, usize) = (placed_placable.location.0 - top_left_location.0 + origin.0, placed_placable.location.1 - top_left_location.1 + origin.1);

                            if calculated_placed_placable_location.0 == top_left_contact_corner.0 {
                                // move from the current direction into the direction of the wall (if this axis is main wall adjacent) or into the direction of the first wall (if this axis is first wall adjacent)
                                let detection_location: (usize, usize) = ((calculated_placed_placable_location.0 as i8 + top_left_contact_corner_delta.0) as usize, calculated_placed_placable_location.1);
                                applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                            if calculated_placed_placable_location.1 == top_left_contact_corner.1 {
                                let detection_location: (usize, usize) = (calculated_placed_placable_location.0, (calculated_placed_placable_location.1 as i8 + top_left_contact_corner_delta.1) as usize);
                                applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                            if calculated_placed_placable_location.0 == bottom_right_contact_corner.0 {
                                // move from the current direction into the direction of the wall (if this axis is main wall adjacent) or into the direction of the first wall (if this axis is first wall adjacent)
                                let detection_location: (usize, usize) = ((calculated_placed_placable_location.0 as i8 + bottom_right_contact_corner_delta.0) as usize, calculated_placed_placable_location.1);
                                applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                            if calculated_placed_placable_location.1 == bottom_right_contact_corner.1 {
                                let detection_location: (usize, usize) = (calculated_placed_placable_location.0, (calculated_placed_placable_location.1 as i8 + bottom_right_contact_corner_delta.1) as usize);
                                applicable_wall_adjacent_detection_locations.push(detection_location);
                            }
                        }
                        applicable_wall_adjacent_detection_locations_per_location.insert(origin, applicable_wall_adjacent_detection_locations);
                    }

                    wall_incrementer_and_wall_adjacent_tuples.push((possible_wall_location_index_incrementer, applicable_wall_adjacent_detection_locations_per_location, wall_indexes));
                }
                else {
                    panic!("Unexpected number of walls in contact with wall-adjacent {}: {:?}", wall_adjacent_index, wall_adjacent);
                }

                debug!("iterating over possible wall location incrementers and such");

                // iterate over the possible wall location incrementers and where wall-adjacents can be located in order to find valid locations for wall-adjacents
                {
                    let mut wall_locations_per_wall_index_per_wall_adjacent_location: HashMap<(usize, usize), HashMap<usize, HashSet<(usize, usize)>>> = HashMap::new();
                    for (mut incrementer, applicable_wall_adjacent_detection_locations_per_location, wall_indexes) in wall_incrementer_and_wall_adjacent_tuples.into_iter() {
                        for (potential_wall_adjacent_location, applicable_wall_adjacent_detection_locations) in applicable_wall_adjacent_detection_locations_per_location.into_iter() {

                            let mut is_wall_iteration_successful: bool = true;
                            while is_wall_iteration_successful {
                                let mut discovered_wall_location_per_wall_index: HashMap<usize, (usize, usize)> = HashMap::new();

                                // get the current location indexes for each wall as a parallel array to wall_indexes
                                let possible_wall_location_indexes = incrementer.get();

                                // iterate over all local walls, storing which wall indexes and their locations are detected next to this wall-adjacent
                                for (wall_index, location_index) in zip(wall_indexes.iter().copied(), possible_wall_location_indexes.into_iter()) {
                                    let wall_location = raw_locations_per_wall_index.get(&wall_index).unwrap()[location_index];
                                    let wall: &PlacedPlacableCollection = &walls[wall_index];
                                    let placed_placable_location_delta: (i8, i8) = (wall_location.0 as i8 - wall.placed_placables[0].location.0 as i8, wall_location.1 as i8 - wall.placed_placables[0].location.1 as i8);
                                    for placed_placable in walls[wall_index].placed_placables.iter() {
                                        let calculated_wall_placed_placable_location = ((placed_placable.location.0 as i8 + placed_placable_location_delta.0) as usize, (placed_placable.location.1 as i8 + placed_placable_location_delta.1) as usize);
                                        if applicable_wall_adjacent_detection_locations.contains(&calculated_wall_placed_placable_location) {
                                            // this wall is detected to be adjacent to this wall-adjacent
                                            discovered_wall_location_per_wall_index.insert(wall_index, wall_location);
                                            break;
                                        }
                                    }
                                }
                                
                                // check that the discovered adjacent wall indexes match the exactly expected wall indexes
                                let original_wall_indexes = wall_indexes_per_wall_adjacent_index.get(&wall_adjacent_index).unwrap();
                                if discovered_wall_location_per_wall_index.len() == original_wall_indexes.len() {
                                    let mut is_all_discovered_wall_indexes_same_as_original_wall_indexes: bool = true;
                                    for wall_index in discovered_wall_location_per_wall_index.keys() {
                                        if !original_wall_indexes.contains(wall_index) {
                                            is_all_discovered_wall_indexes_same_as_original_wall_indexes = false;
                                            break;
                                        }
                                    }
                                    if is_all_discovered_wall_indexes_same_as_original_wall_indexes {
                                        // the wall indexes found at this combination of wall-adjacent location and all relevant wall locations correspond with the original locations

                                        if !wall_locations_per_wall_index_per_wall_adjacent_location.contains_key(&potential_wall_adjacent_location) {
                                            wall_locations_per_wall_index_per_wall_adjacent_location.insert(potential_wall_adjacent_location, HashMap::new());
                                        }
                                        for (wall_index, wall_location) in discovered_wall_location_per_wall_index.into_iter() {
                                            if !wall_locations_per_wall_index_per_wall_adjacent_location.get(&potential_wall_adjacent_location).unwrap().contains_key(&wall_index) {
                                                wall_locations_per_wall_index_per_wall_adjacent_location.get_mut(&potential_wall_adjacent_location).unwrap().insert(wall_index, HashSet::new());
                                            }

                                            // store the wall-adjacent location and wall locations for node state collection purposes
                                            wall_locations_per_wall_index_per_wall_adjacent_location.get_mut(&potential_wall_adjacent_location).unwrap().get_mut(&wall_index).unwrap().insert(wall_location);

                                            // store the filtered wall locations since they are expected to be reduced as part of this process
                                            if !filtered_locations_per_wall_index.contains_key(&wall_index) {
                                                filtered_locations_per_wall_index.insert(wall_index, Vec::new());
                                            }
                                            if !filtered_locations_per_wall_index.get(&wall_index).unwrap().contains(&wall_location) {
                                                filtered_locations_per_wall_index.get_mut(&wall_index).unwrap().push(wall_location);
                                            }
                                        }
                                    }
                                }

                                is_wall_iteration_successful = incrementer.try_increment();
                            }
                        }
                    }
                    wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index.insert(wall_adjacent_index, wall_locations_per_wall_index_per_wall_adjacent_location);
                }

                debug!("iterated over possible wall location incrementers and such");

            }

            debug!("wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index: {:?}", wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index);
        }

        // collect possible wall-adjacent locations per wall-adjacent location

        let mut possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index: HashMap<usize, HashMap<(usize, usize), HashMap<usize, Vec<(usize, usize)>>>> = HashMap::new();

        {
            for (wall_adjacent_index, wall_adjacent) in wall_adjacents.iter().enumerate() {
                
                possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.insert(wall_adjacent_index, HashMap::new());
                let wall_adjacent_top_left_location = top_left_location_per_wall_adjacent_index.get(&wall_adjacent_index).unwrap();

                for (other_wall_adjacent_index, other_wall_adjacent) in wall_adjacents.iter().enumerate() {

                    let other_wall_adjacent_top_left_location = top_left_location_per_wall_adjacent_index.get(&other_wall_adjacent_index).unwrap();

                    if wall_adjacent_index != other_wall_adjacent_index {
                        
                        // check that when the wall_adjacent is at each possible location, that the other_wall_adjacent is not overlapping or directly adjacent

                        for wall_adjacent_location in wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index.get(&wall_adjacent_index).unwrap().keys() {

                            if !possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(&wall_adjacent_index).unwrap().contains_key(wall_adjacent_location) {
                                possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get_mut(&wall_adjacent_index).unwrap().insert(wall_adjacent_location.to_owned(), HashMap::new());
                            }
                            if !possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(&wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().contains_key(&other_wall_adjacent_index) {
                                possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get_mut(&wall_adjacent_index).unwrap().get_mut(wall_adjacent_location).unwrap().insert(other_wall_adjacent_index, Vec::new());
                            }

                            // collect detection locations that the other wall-adjacent cannot exist at
                            let mut wall_adjacent_detection_locations: HashSet<(usize, usize)> = HashSet::new();
                            for placed_placable in wall_adjacent.placed_placables.iter() {
                                let calculated_placed_placable_location = (placed_placable.location.0 - wall_adjacent_top_left_location.0 + wall_adjacent_location.0, placed_placable.location.1 - wall_adjacent_top_left_location.1 + wall_adjacent_location.1);
                                wall_adjacent_detection_locations.insert(calculated_placed_placable_location);
                                wall_adjacent_detection_locations.insert((calculated_placed_placable_location.0 - 1, calculated_placed_placable_location.1));
                                wall_adjacent_detection_locations.insert((calculated_placed_placable_location.0 + 1, calculated_placed_placable_location.1));
                                wall_adjacent_detection_locations.insert((calculated_placed_placable_location.0, calculated_placed_placable_location.1 - 1));
                                wall_adjacent_detection_locations.insert((calculated_placed_placable_location.0, calculated_placed_placable_location.1 + 1));
                            }

                            for other_wall_adjacent_location in wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index.get(&other_wall_adjacent_index).unwrap().keys() {
                                let mut is_other_wall_adjacent_permitted_at_location: bool = true;
                                for placed_placable in other_wall_adjacent.placed_placables.iter() {
                                    let calculated_placed_placable_location = (placed_placable.location.0 - other_wall_adjacent_top_left_location.0 + other_wall_adjacent_location.0, placed_placable.location.1 - other_wall_adjacent_top_left_location.1 + other_wall_adjacent_location.1);
                                    if wall_adjacent_detection_locations.contains(&calculated_placed_placable_location) {
                                        is_other_wall_adjacent_permitted_at_location = false;
                                        break;
                                    }
                                }
                                if is_other_wall_adjacent_permitted_at_location {
                                    possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get_mut(&wall_adjacent_index).unwrap().get_mut(wall_adjacent_location).unwrap().get_mut(&other_wall_adjacent_index).unwrap().push(other_wall_adjacent_location.to_owned());
                                }
                            }
                        }
                    }
                }
            }
        }

        // limit wall-adjacent locations to only those that do not over-restrict another wall-adjacent

        let mut unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index: HashMap<usize, HashMap<(usize, usize), HashMap<usize, Vec<(usize, usize)>>>> = HashMap::new();

        {
            let mut restricted_wall_adjacent_locations_per_wall_adjacent_index: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
            for wall_adjacent_index in possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.keys() {
                for wall_adjacent_location in possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().keys() {
                    let mut is_at_least_one_other_wall_adjacent_fully_restricted: bool = false;
                    for other_wall_adjacent_index in possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().keys() {
                        if possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().get(other_wall_adjacent_index).unwrap().is_empty() {
                            is_at_least_one_other_wall_adjacent_fully_restricted = true;
                            break;
                        }
                    }
                    if is_at_least_one_other_wall_adjacent_fully_restricted {
                        if !restricted_wall_adjacent_locations_per_wall_adjacent_index.contains_key(wall_adjacent_index) {
                            restricted_wall_adjacent_locations_per_wall_adjacent_index.insert(wall_adjacent_index.to_owned(), Vec::new());
                        }
                        restricted_wall_adjacent_locations_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().push(wall_adjacent_location.clone());
                    }
                }
            }

            for wall_adjacent_index in possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.keys() {
                for wall_adjacent_location in possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().keys() {
                    if restricted_wall_adjacent_locations_per_wall_adjacent_index.contains_key(wall_adjacent_index) &&
                        restricted_wall_adjacent_locations_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().contains(wall_adjacent_location) {

                        // this wall-adjacent location for this wall-adjacent is too restrictive for at least one other wall-adjacent
                    }
                    else {
                        if !unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.contains_key(wall_adjacent_index) {
                            unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.insert(wall_adjacent_index.to_owned(), HashMap::new());
                        }
                        if !unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().contains_key(wall_adjacent_location) {
                            unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().insert(wall_adjacent_location.to_owned(), HashMap::new());
                        }
                        for other_wall_adjacent_index in possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().keys() {
                            for other_wall_adjacent_location in possible_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().get(other_wall_adjacent_index).unwrap().iter() {
                                if restricted_wall_adjacent_locations_per_wall_adjacent_index.contains_key(other_wall_adjacent_index) &&
                                    restricted_wall_adjacent_locations_per_wall_adjacent_index.get(other_wall_adjacent_index).unwrap().contains(other_wall_adjacent_location) {

                                    // this wall-adjacent location for this wall-adjacent is too restrictive for at least one other wall-adjacent
                                }
                                else {
                                    if !unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().contains_key(other_wall_adjacent_index) {
                                        unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().get_mut(wall_adjacent_location).unwrap().insert(other_wall_adjacent_index.to_owned(), Vec::new());
                                    }
                                    unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().get_mut(wall_adjacent_location).unwrap().get_mut(other_wall_adjacent_index).unwrap().push(other_wall_adjacent_location.to_owned());
                                }
                            }
                        }
                    }
                }
            }

            debug!("unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index: 2.(20, 28).3 length {:?}", unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(&2).unwrap().get(&(20, 28)).unwrap().get(&3).unwrap().len());
            debug!("unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index: 2.(20, 28).3 {:?}", unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(&2).unwrap().get(&(20, 28)).unwrap().get(&3).unwrap());
        }

        // collect PlacedPlacableCollection instances representing the floaters

        // determine the possible locations of every floater compared to every wall-adjacent

        // determine the subset of possible locations of every floater compared to every other floater

        // determine the wall-adjacent locations that are possible now that they have been further restricted by large floaters

        let mut permitted_wall_adjacent_locations_per_wall_adjacent_index: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();

        {
            // TODO consider the influence of large floaters
            
            for wall_adjacent_index in unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.keys() {
                permitted_wall_adjacent_locations_per_wall_adjacent_index.insert(wall_adjacent_index.to_owned(), Vec::new());
                for wall_adjacent_location in unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().keys() {
                    permitted_wall_adjacent_locations_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().push(wall_adjacent_location.to_owned());
                }
            }
        }

        // determine the wall locations that are possible now that they have been further restricted by wall-adjacents which were restricted by large floaters

        let mut permitted_wall_locations_per_wall_index: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();

        {
            // TODO use the latest wall-adjacent locations once the large floaters logic is implemented
            for (wall_index, wall_locations) in raw_locations_per_wall_index.iter() {
                permitted_wall_locations_per_wall_index.insert(wall_index.to_owned(), Vec::new());
                for wall_location in wall_locations.iter() {
                    let mut is_wall_location_permitted_by_every_wall_adjacent: bool = true;
                    for wall_adjacent_index in unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.keys() {
                        let mut is_found_for_at_least_one_wall_adjacent_location: bool = false;
                        for wall_adjacent_location in unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().keys() {
                            if wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().contains_key(wall_index) &&
                                wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().get(wall_index).unwrap().contains(wall_location) {
                                
                                is_found_for_at_least_one_wall_adjacent_location = true;
                                break;
                            }
                        }
                        if !is_found_for_at_least_one_wall_adjacent_location {
                            is_wall_location_permitted_by_every_wall_adjacent = false;
                        }
                    }
                    if is_wall_location_permitted_by_every_wall_adjacent {
                        permitted_wall_locations_per_wall_index.get_mut(wall_index).unwrap().push(wall_location.to_owned());
                    }
                }
            }
        }

        // construct the node state collections

        let mut node_state_collection_ids_per_wall_index_per_wall_adjacent_index: HashMap<usize, HashMap<usize, Vec<NodeStateCollectionIdentifier>>> = HashMap::new();
        let mut node_state_collection_ids_per_wall_adjacent_index_per_wall_adjacent_index: HashMap<usize, HashMap<usize, Vec<NodeStateCollectionIdentifier>>> = HashMap::new();
        let mut node_state_collection_id_per_anonymous_node_state_collection: HashMap<AnonymousNodeStateCollection<NodeState>, NodeStateCollectionIdentifier> = HashMap::new();
        let mut node_state_collections: Vec<NodeStateCollection<Identifier, NodeState>> = Vec::new();

        {
            for wall_adjacent_index in unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.keys() {
                node_state_collection_ids_per_wall_index_per_wall_adjacent_index.insert(wall_adjacent_index.to_owned(), HashMap::new());
                node_state_collection_ids_per_wall_adjacent_index_per_wall_adjacent_index.insert(wall_adjacent_index.to_owned(), HashMap::new());
                for wall_adjacent_location in unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().keys() {

                    // store the permitted wall locations for this wall-adjacent
                    for wall_index in wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().keys() {
                        // create an anonymous node state collection for when this wall adjacent is at this location that this wall is permitted at these locations
                        let anonymous_node_state_collection = AnonymousNodeStateCollection {
                            when_node_state: NodeState {
                                location: wall_adjacent_location.to_owned()
                            },
                            then_node_states: wall_locations_per_wall_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().get(wall_index).unwrap().iter().cloned().map(|location| NodeState { location: location }).collect()
                        };

                        let node_state_collection_id: NodeStateCollectionIdentifier;
                        if !node_state_collection_id_per_anonymous_node_state_collection.contains_key(&anonymous_node_state_collection) {
                            let uuid: String = Uuid::new_v4().to_string();
                            node_state_collection_id = NodeStateCollectionIdentifier {
                                uuid: uuid
                            };
                            node_state_collection_id_per_anonymous_node_state_collection.insert(anonymous_node_state_collection, node_state_collection_id.clone());
                        }
                        else {
                            node_state_collection_id = node_state_collection_id_per_anonymous_node_state_collection.get(&anonymous_node_state_collection).unwrap().clone();
                        }

                        if !node_state_collection_ids_per_wall_index_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().contains_key(wall_index) {
                            node_state_collection_ids_per_wall_index_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().insert(wall_index.to_owned(), Vec::new());
                        }
                        node_state_collection_ids_per_wall_index_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().get_mut(wall_index).unwrap().push(node_state_collection_id);
                    }

                    // store the permitted wall-adjacent locations for this wall-adjacent
                    for other_wall_adjacent_index in unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().keys() {
                        // create an anonymous node state collection for when this wall-adjacent is at this location that this other wall-adjacent is permitted at these locations
                        let anonymous_node_state_collection = AnonymousNodeStateCollection {
                            when_node_state: NodeState {
                                location: wall_adjacent_location.to_owned()
                            },
                            then_node_states: unrestricted_wall_adjacent_locations_per_wall_adjacent_index_per_wall_adjacent_location_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().get(wall_adjacent_location).unwrap().get(other_wall_adjacent_index).unwrap().iter().cloned().map(|location| NodeState { location: location }).collect()
                        };

                        let node_state_collection_id: NodeStateCollectionIdentifier;
                        if !node_state_collection_id_per_anonymous_node_state_collection.contains_key(&anonymous_node_state_collection) {
                            let uuid: String = Uuid::new_v4().to_string();
                            node_state_collection_id = NodeStateCollectionIdentifier {
                                uuid: uuid
                            };
                            node_state_collection_id_per_anonymous_node_state_collection.insert(anonymous_node_state_collection, node_state_collection_id.clone());
                        }
                        else {
                            node_state_collection_id = node_state_collection_id_per_anonymous_node_state_collection.get(&anonymous_node_state_collection).unwrap().clone();
                        }

                        if !node_state_collection_ids_per_wall_adjacent_index_per_wall_adjacent_index.get(wall_adjacent_index).unwrap().contains_key(other_wall_adjacent_index) {
                            node_state_collection_ids_per_wall_adjacent_index_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().insert(other_wall_adjacent_index.to_owned(), Vec::new());
                        }
                        node_state_collection_ids_per_wall_adjacent_index_per_wall_adjacent_index.get_mut(wall_adjacent_index).unwrap().get_mut(other_wall_adjacent_index).unwrap().push(node_state_collection_id);
                    }
                }
            }

            for (anonymous_node_state_collection, node_state_collection_id) in node_state_collection_id_per_anonymous_node_state_collection.into_iter() {
                node_state_collections.push(NodeStateCollection::new_from_anonymous(Identifier::NodeStateCollection(node_state_collection_id), anonymous_node_state_collection));
            }
        }

        // construct the nodes

        let mut node_id_per_wall_index: HashMap<usize, Identifier> = HashMap::new();
        let mut node_id_per_wall_adjacent_index: HashMap<usize, Identifier> = HashMap::new();
        let mut nodes: Vec<Node<Identifier, NodeState>> = Vec::new();

        {
            for wall_index in 0..walls.len() {
                node_id_per_wall_index.insert(wall_index, Identifier::Node(NodeIdentifier { component_type: ComponentType::Wall, index: wall_index }));
            }
            for wall_adjacent_index in 0..wall_adjacents.len() {
                node_id_per_wall_adjacent_index.insert(wall_adjacent_index, Identifier::Node(NodeIdentifier { component_type: ComponentType::WallAdjacent, index: wall_adjacent_index }));
            }
            // TODO floaters

            for wall_index in 0..walls.len() {
                let node_id = node_id_per_wall_index.get(&wall_index).unwrap();
                let mut node_states: Vec<NodeState> = Vec::new();

                for wall_location in permitted_wall_locations_per_wall_index.get(&wall_index).unwrap().iter() {
                    node_states.push(NodeState {
                        location: wall_location.to_owned()
                    });
                }

                let node_state_collection_ids_per_neighbor_node_id: HashMap<Identifier, Vec<Identifier>> = HashMap::new();

                // TODO tie the walls to the wall-adjacents

                let node = Node::new(node_id.clone(), NodeStateProbability::get_equal_probability(node_states), node_state_collection_ids_per_neighbor_node_id);
                nodes.push(node);
            }

            for wall_adjacent_index in 0..wall_adjacents.len() {
                let node_id = node_id_per_wall_adjacent_index.get(&wall_adjacent_index).unwrap();
                let mut node_states: Vec<NodeState> = Vec::new();

                for wall_adjacent_location in permitted_wall_adjacent_locations_per_wall_adjacent_index.get(&wall_adjacent_index).unwrap().iter() {
                    node_states.push(NodeState {
                        location: wall_adjacent_location.to_owned()
                    });
                }

                let node_state_collection_ids_per_neighbor_node_id: HashMap<Identifier, Vec<Identifier>> = HashMap::new();

                // TODO tie the wall-adjacents to the walls

                // TODO tie the wall-adjacents to the floaters

                let node = Node::new(node_id.clone(), NodeStateProbability::get_equal_probability(node_states), node_state_collection_ids_per_neighbor_node_id);
                nodes.push(node);
            }

            // TODO create a similar loop for creating the nodes and node state collections, but for the floaters

        }

        let wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        wave_function
    }
    pub fn get_similar_level(&self) -> Self {

        let wave_function = self.get_wave_function();

        let mut rng = rand::thread_rng();
        let random_seed = Some(rng.gen::<u64>());

        let mut collapsable_wave_function = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<Identifier, NodeState>>(random_seed);

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