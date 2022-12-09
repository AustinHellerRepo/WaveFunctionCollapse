use std::collections::{VecDeque, HashMap, HashSet};
use colored::Colorize;
use rand::Rng;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{Node, NodeStateCollection, WaveFunction, collapsable_wave_function::{entropic_collapsable_wave_function::EntropicCollapsableWaveFunction, collapsable_wave_function::CollapsableWaveFunction}};

fn print_pixel(color: &[u8; 4]) {
    let character = "\u{2588}";
    print!("{}{}", character.truecolor(color[0], color[1], color[2]), character.truecolor(color[0], color[1], color[2]));
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
enum TileType {
    Outside,
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
            TileType::Outside => {
                panic!("Outside should never attempt to be drawn");
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
struct TileSet {
    tile_type_grid: [[TileType; 3]; 3]
}

impl TileSet {
    fn is_overlapping(&self, other_tile_set: &TileSet, width_offset: i8, height_offset: i8) -> bool {
        let mut is_at_least_one_tile_type_nonoverlapping: bool = false;
        for self_height_index in std::cmp::max(0, height_offset)..std::cmp::min(3, height_offset + 3 as i8) {
            let other_height_index = self_height_index - height_offset;
            for self_width_index in std::cmp::max(0, width_offset)..std::cmp::min(3, width_offset + 3 as i8) {
                let other_width_index = self_width_index - width_offset;
                let other_tile_type = &other_tile_set.tile_type_grid[other_width_index as usize][other_height_index as usize];
                let self_tile_type = &self.tile_type_grid[self_width_index as usize][self_height_index as usize];
                if other_tile_type != self_tile_type {
                    is_at_least_one_tile_type_nonoverlapping = true;
                    break;
                }
            }
        }
        !is_at_least_one_tile_type_nonoverlapping
    }
    fn is_top(&self) -> bool {
        self.tile_type_grid[0][0] == TileType::Outside &&
        self.tile_type_grid[1][0] == TileType::Outside &&
        self.tile_type_grid[2][0] == TileType::Outside
    }
    fn is_right(&self) -> bool {
        self.tile_type_grid[2][0] == TileType::Outside &&
        self.tile_type_grid[2][1] == TileType::Outside &&
        self.tile_type_grid[2][2] == TileType::Outside
    }
    fn is_bottom(&self) -> bool {
        self.tile_type_grid[0][2] == TileType::Outside &&
        self.tile_type_grid[1][2] == TileType::Outside &&
        self.tile_type_grid[2][2] == TileType::Outside
    }
    fn is_left(&self) -> bool {
        self.tile_type_grid[0][0] == TileType::Outside &&
        self.tile_type_grid[0][1] == TileType::Outside &&
        self.tile_type_grid[0][2] == TileType::Outside
    }
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
struct ElementTileShell {
    placed_placables: Vec<PlacedPlacable>
}

#[derive(Clone, Hash, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
enum NodeState {
    TileSet(TileSet),
    ElementTileShell(ElementTileShell)
}

impl NodeState {
    fn get_color(&self, width_index: usize, height_index: usize) -> [u8; 4] {
        match self {
            NodeState::TileSet(tile_set) => {
                tile_set.tile_type_grid[width_index][height_index].get_color()
            },
            NodeState::ElementTileShell(element_tile_shell) => {
                let location = (width_index, height_index);
                for placed_placable in element_tile_shell.placed_placables.iter() {
                    if placed_placable.location == location {
                        return placed_placable.get_color();
                    }
                }
                [0, 0, 0, 255]
            }
        }
    }
}

struct Level {
    width: usize,
    height: usize,
    placed_placables: Vec<PlacedPlacable>
}

impl Level {
    fn default() -> Self {
        let width: usize = 50;
        let height: usize = 30;
        let mut placed_placables: Vec<PlacedPlacable> = Vec::new();
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
    fn get_similar_level(&self) -> Self {

        let mut placed_placable_per_location: HashMap<(usize, usize), &PlacedPlacable> = HashMap::new();
        for placed_placable in self.placed_placables.iter() {
            placed_placable_per_location.insert(placed_placable.location, placed_placable);
        }

        let mut nodes: Vec<Node<NodeState>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<NodeState>> = Vec::new();

        // construct tile set nodes
        let mut tile_set_node_id_per_location: HashMap<(usize, usize), String> = HashMap::new();
        let mut ratio_per_tile_set: HashMap<TileSet, u32> = HashMap::new();

        // This is what the tile set nodes look like with Outside padding if the world was 3x3 in size
        //  and the result should match the 3x3 since the Outside is ignored after being generated.
        //
        //  Outside Outside Outside Outside Outside
        //  Outside Solid   Empty   Solid   Outside
        //  Outside Solid   Empty   Solid   Outside
        //  Outside Solid   Solid   Solid   Outside
        //  Outside Outside Outside Outside Outside

        for height_index in 0..=(self.height - 1) {
            for width_index in 0..=(self.width - 1) {

                tile_set_node_id_per_location.insert((width_index, height_index), format!("tile_set_node_{}_{}", width_index, height_index));

                // start from the current (width_index, height_index) and piece together a tileset
                // treat 0 and max as TileType::Outside

                let mut tile_type_grid: [[TileType; 3]; 3] = Default::default();
                for height_offset in 0..3 {
                    for width_offset in 0..3 {
                        let tile_type: TileType;
                        let calculated_height_index = height_index + height_offset;
                        let calculated_width_index = width_index + width_offset;
                        if calculated_width_index == 0 || calculated_height_index == 0 ||
                            calculated_width_index == self.width + 1 || calculated_height_index == self.height + 1 {

                            tile_type = TileType::Outside;
                        }
                        else {
                            let location = (calculated_width_index - 1, calculated_height_index - 1);
                            if placed_placable_per_location.contains_key(&location) {
                                let placed_placable: &PlacedPlacable = placed_placable_per_location.get(&location).unwrap();
                                match &placed_placable.placable {
                                    Placable::Tile(tile_type_value) => {
                                        tile_type = tile_type_value.clone();
                                    },
                                    Placable::Element(_) => {
                                        tile_type = TileType::Empty;
                                    }
                                }
                            }
                            else {
                                tile_type = TileType::Empty;
                            }
                        }

                        tile_type_grid[width_offset][height_offset] = tile_type;
                    }
                }

                let tile_set: TileSet = TileSet {
                    tile_type_grid: tile_type_grid
                };

                let mut ratio = 0;
                if ratio_per_tile_set.contains_key(&tile_set) {
                    ratio = *ratio_per_tile_set.get(&tile_set).unwrap();
                }
                ratio_per_tile_set.insert(tile_set, ratio + 1);
            }
        }

        println!("ratio_per_tile_set: {:?}", ratio_per_tile_set);

        // construct permitted overlapping tile sets
        let mut permitted_tile_sets_per_height_offset_per_width_offset_per_tile_set: HashMap<&TileSet, HashMap<i8, HashMap<i8, Vec<NodeState>>>> = HashMap::new();
        for root_tile_set in ratio_per_tile_set.keys() {
            let mut permitted_tile_sets_per_height_offset_per_width_offset: HashMap<i8, HashMap<i8, Vec<NodeState>>> = HashMap::new();
            for width_offset in -1..=1 as i8 {
                let mut permitted_tile_sets_per_height_offset: HashMap<i8, Vec<NodeState>> = HashMap::new();
                for height_offset in -1..=1 as i8 {
                    // do not setup node state collection for root overlapping root
                    if !(height_offset == 0 && width_offset == 0 ||
                        height_offset.abs() == 1 && width_offset.abs() == 1) {
                        
                        let mut permitted_tile_sets: Vec<NodeState> = Vec::new();
                        for other_tile_set in ratio_per_tile_set.keys() {
                            if root_tile_set.is_overlapping(other_tile_set, width_offset, height_offset) {
                                permitted_tile_sets.push(NodeState::TileSet(other_tile_set.clone()));
                            }
                        }
                        permitted_tile_sets_per_height_offset.insert(height_offset, permitted_tile_sets);
                    }
                }
                permitted_tile_sets_per_height_offset_per_width_offset.insert(width_offset, permitted_tile_sets_per_height_offset);
            }
            permitted_tile_sets_per_height_offset_per_width_offset_per_tile_set.insert(root_tile_set, permitted_tile_sets_per_height_offset_per_width_offset);
        }

        // construct the distinct node state collections per offset height per offset width
        let mut tile_set_node_state_collection_ids_per_height_offset_per_width_offset: HashMap<i8, HashMap<i8, Vec<String>>> = HashMap::new();
        for (from_tile_set, permitted_tile_sets_per_height_offset_per_width_offset) in permitted_tile_sets_per_height_offset_per_width_offset_per_tile_set.into_iter() {
            for (width_offset, permitted_tile_sets_per_height_offset) in permitted_tile_sets_per_height_offset_per_width_offset.into_iter() {
                if !tile_set_node_state_collection_ids_per_height_offset_per_width_offset.contains_key(&width_offset) {
                    tile_set_node_state_collection_ids_per_height_offset_per_width_offset.insert(width_offset, HashMap::new());
                }
                for (height_offset, permitted_tile_sets) in permitted_tile_sets_per_height_offset.into_iter() {
                    if !tile_set_node_state_collection_ids_per_height_offset_per_width_offset.get(&width_offset).unwrap().contains_key(&height_offset) {
                        tile_set_node_state_collection_ids_per_height_offset_per_width_offset.get_mut(&width_offset).unwrap().insert(height_offset, Vec::new());
                    }
                    let node_state_collection_id = Uuid::new_v4().to_string();
                    let node_state_collection: NodeStateCollection<NodeState> = NodeStateCollection::new(node_state_collection_id.clone(), NodeState::TileSet(from_tile_set.clone()), permitted_tile_sets);
                    tile_set_node_state_collection_ids_per_height_offset_per_width_offset.get_mut(&width_offset).unwrap().get_mut(&height_offset).unwrap().push(node_state_collection_id);
                    node_state_collections.push(node_state_collection);
                }
            }
        }

        // construct tile set nodes

        // create each node such that its relative node state collections are specified
        for node_height_index in 0..=(self.height - 1) {
            for node_width_index in 0..=(self.width - 1) {
                let location = (node_width_index, node_height_index);
                let node_id: &String = tile_set_node_id_per_location.get(&location).unwrap();
                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                for neighbor_height_offset in -1..=1 as i8 {
                    for neighbor_width_offset in -1..=1 as i8 {
                        if !(neighbor_width_offset == 0 && neighbor_height_offset == 0 ||
                            neighbor_width_offset.abs() == 1 && neighbor_height_offset.abs() == 1) {
                            
                            let neighbor_width_index = node_width_index as i8 + neighbor_width_offset;
                            let neighbor_height_index = node_height_index as i8 + neighbor_height_offset;

                            if neighbor_width_index >= 0 &&
                                neighbor_width_index <= (self.width - 1) as i8 &&
                                neighbor_height_index >= 0 &&
                                neighbor_height_index <= (self.height - 1) as i8 {

                                let neighbor_location = (neighbor_width_index as usize, neighbor_height_index as usize);
                                let neighbor_node_id = tile_set_node_id_per_location.get(&neighbor_location).unwrap();
                                let node_state_collection_ids = tile_set_node_state_collection_ids_per_height_offset_per_width_offset.get(&neighbor_width_offset).unwrap().get(&neighbor_height_offset).unwrap();
                                node_state_collection_ids_per_neighbor_node_id.insert(neighbor_node_id.clone(), node_state_collection_ids.clone());
                            }
                        }
                    }
                }

                let mut node_state_ratio_per_node_state_id: HashMap<NodeState, f32> = HashMap::new();
 
                // only permit the node at this index to be top/right/bottom/left as appropriate
                for (tile_set, ratio) in ratio_per_tile_set.iter() {
                    if (node_width_index == 0 && tile_set.is_left() || node_width_index != 0) &&
                        (node_height_index == 0 && tile_set.is_top() || node_height_index != 0) &&
                        (node_width_index == self.width - 1 && tile_set.is_right() || node_width_index != self.width - 1) &&
                        (node_height_index == self.height - 1 && tile_set.is_bottom() || node_height_index != self.height - 1) {
                        
                        node_state_ratio_per_node_state_id.insert(NodeState::TileSet(tile_set.clone()), *ratio as f32);
                    }
                }

                let node: Node<NodeState> = Node::new(node_id.clone(), node_state_ratio_per_node_state_id, node_state_collection_ids_per_neighbor_node_id);
                nodes.push(node);
            }
        }

        // create element relationships and tile shells
        let mut element_tile_shells: Vec<Vec<PlacedPlacable>> = Vec::new();

        let mut remaining_elements: VecDeque<PlacedPlacable> = VecDeque::new();
        for placed_placable in self.placed_placables.iter() {
            match placed_placable.placable {
                Placable::Element(_) => {
                    remaining_elements.push_back(placed_placable.clone());
                },
                Placable::Tile(_) => {
                    // Not storing tiles
                }
            }
        }

        while !remaining_elements.is_empty() {
            let popped_element = remaining_elements.pop_front().unwrap();

            let mut calculated_minimum_width_index = self.width;
            let mut calculated_minimum_height_index = self.height;

            // look around the popped element for any and all tiles and other elements
            let tile_look_distance = 1;
            let element_look_distance = 2;

            let mut group_of_nearby_elements: Vec<PlacedPlacable> = Vec::new();

            let mut searching_nearby_placed_placable_elements: VecDeque<PlacedPlacable> = VecDeque::new();
            searching_nearby_placed_placable_elements.push_back(popped_element);
            while !searching_nearby_placed_placable_elements.is_empty() {
                let popped_searching_nearby_placed_placable_element = searching_nearby_placed_placable_elements.pop_front().unwrap();

                let min_height_look_distance = std::cmp::max(0, popped_searching_nearby_placed_placable_element.location.1 - element_look_distance);
                let max_height_look_distance = std::cmp::min(self.height - 1, popped_searching_nearby_placed_placable_element.location.1 + element_look_distance);
                let min_width_look_distance = std::cmp::max(0, popped_searching_nearby_placed_placable_element.location.0 - element_look_distance);
                let max_width_look_distance = std::cmp::min(self.width - 1, popped_searching_nearby_placed_placable_element.location.0 + element_look_distance);
                for height_index in min_height_look_distance..=max_height_look_distance {
                    for width_index in min_width_look_distance..=max_width_look_distance {
                        if !(height_index == 0 && width_index == 0) {
                            let location = (width_index, height_index);
                            if placed_placable_per_location.contains_key(&location) {
                                let placed_placable: &PlacedPlacable = placed_placable_per_location.get(&location).unwrap();
                                match placed_placable.placable {
                                    Placable::Element(_) => {
                                        if !group_of_nearby_elements.contains(placed_placable) && !searching_nearby_placed_placable_elements.contains(placed_placable) {
                                            searching_nearby_placed_placable_elements.push_back(placed_placable.clone());
                                        }
                                    },
                                    Placable::Tile(_) => {
                                        // not searching for nearby tiles yet
                                    }
                                }
                            }
                        }
                    }
                }

                // remove searched element from outer loop's search
                remaining_elements.retain(|element| element != &popped_searching_nearby_placed_placable_element);

                if popped_searching_nearby_placed_placable_element.location.0 < calculated_minimum_width_index {
                    calculated_minimum_width_index = popped_searching_nearby_placed_placable_element.location.0;
                }
                if popped_searching_nearby_placed_placable_element.location.1 < calculated_minimum_height_index {
                    calculated_minimum_height_index = popped_searching_nearby_placed_placable_element.location.1;
                }

                group_of_nearby_elements.push(popped_searching_nearby_placed_placable_element);
            }
            
            // the group of nearby elements is now known, so tiles nearby these elements must now be discovered

            // search for tiles around each element in group_of_nearby_elements
            let mut group_of_nearby_tiles: Vec<PlacedPlacable> = Vec::new();
            for nearby_element in group_of_nearby_elements.iter() {
                let min_height_look_distance = std::cmp::max(0, nearby_element.location.1 - tile_look_distance);
                let max_height_look_distance = std::cmp::min(self.height - 1, nearby_element.location.1 + tile_look_distance);
                let min_width_look_distance = std::cmp::max(0, nearby_element.location.0 - tile_look_distance);
                let max_width_look_distance = std::cmp::min(self.width - 1, nearby_element.location.0 + tile_look_distance);
                for height_index in min_height_look_distance..=max_height_look_distance {
                    for width_index in min_width_look_distance..=max_width_look_distance {
                        if !(height_index == 0 && width_index == 0) {
                            let location = (width_index, height_index);
                            if placed_placable_per_location.contains_key(&location) {
                                let placed_placable: &PlacedPlacable = placed_placable_per_location.get(&location).unwrap();
                                match placed_placable.placable {
                                    Placable::Element(_) => {
                                        // not searching for nearby elements
                                    },
                                    Placable::Tile(_) => {
                                        if !group_of_nearby_tiles.contains(placed_placable) {

                                            if placed_placable.location.0 < calculated_minimum_width_index {
                                                calculated_minimum_width_index = placed_placable.location.0;
                                            }
                                            if placed_placable.location.1 < calculated_minimum_height_index {
                                                calculated_minimum_height_index = placed_placable.location.1;
                                            }

                                            group_of_nearby_tiles.push(placed_placable.clone());
                                        }
                                    }
                                }
                            }
                            else {
                                group_of_nearby_tiles.push(PlacedPlacable::new(Placable::Tile(TileType::Empty), width_index, height_index));
                            }
                        }
                    }
                }
            }

            let mut element_tile_shell: Vec<PlacedPlacable> = Vec::new();
            for nearby_element in group_of_nearby_elements.into_iter().chain(group_of_nearby_tiles.into_iter()) {
                let placed_placable = PlacedPlacable::new(nearby_element.placable, nearby_element.location.0 - calculated_minimum_width_index, nearby_element.location.1 - calculated_minimum_height_index);
                element_tile_shell.push(placed_placable);
            }

            element_tile_shells.push(element_tile_shell);
        }

        // all of the element tile shells are now known

        // TODO construct the node layer which the element tile shells would be resolved on

        // find smallest width and height in order to know how many nodes are required
        let mut smallest_element_tile_shell_width = self.width;
        let mut smallest_element_tile_shell_height = self.height;

        for element_tile_shell in element_tile_shells.iter() {
            let mut element_tile_shell_width: usize = 0;
            let mut element_tile_shell_height: usize = 0;
            for placed_placable in element_tile_shell {
                if placed_placable.location.0 > element_tile_shell_width {
                    element_tile_shell_width = placed_placable.location.0;
                }
                if placed_placable.location.1 > element_tile_shell_height {
                    element_tile_shell_height = placed_placable.location.1;
                }
            }
            if element_tile_shell_width < smallest_element_tile_shell_width {
                smallest_element_tile_shell_width = element_tile_shell_width;
            }
            if element_tile_shell_height < smallest_element_tile_shell_height {
                smallest_element_tile_shell_height = element_tile_shell_height;
            }
        }

        let mut element_tile_shell_node_id_per_location: HashMap<(usize, usize), String> = HashMap::new();
        for element_tile_shell_node_height_index in 0..=(self.height - smallest_element_tile_shell_height) {
            for element_tile_shell_node_width_index in 0..=(self.width - smallest_element_tile_shell_width) {
                let location = (element_tile_shell_node_width_index, element_tile_shell_node_height_index);
                element_tile_shell_node_id_per_location.insert(location, Uuid::new_v4().to_string());
            }
        }

        let wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        let mut rng = rand::thread_rng();
        let random_seed = Some(rng.gen::<u64>());

        let mut collapsable_wave_function = wave_function.get_collapsable_wave_function::<EntropicCollapsableWaveFunction<NodeState>>(random_seed);

        let collapsed_wave_function = collapsable_wave_function.collapse().unwrap();

        //println!("collapsed wave function: {:?}", collapsed_wave_function.node_state_per_node);

        let mut node_state_per_height_index_per_width_index: HashMap<usize, HashMap<usize, Option<NodeState>>> = HashMap::new();
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
        }

        todo!();
    }
}

fn main() {

    let level = Level::default();
    level.print();

    let similar_level = level.get_similar_level();
}