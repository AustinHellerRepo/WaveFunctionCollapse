use std::collections::{HashMap, HashSet};
use log::debug;
use rand::Rng;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{WaveFunction, Node, NodeStateCollection, AnonymousNodeStateCollection, NodeStateProbability, collapsable_wave_function::{accommodating_collapsable_wave_function::AccommodatingCollapsableWaveFunction, collapsable_wave_function::CollapsableWaveFunction}};

#[derive(Clone, Hash, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
enum Piece {
    // OOOO
    ExtraLongStraight,

    // OOO
    //   O
    LongCorner,

    // OOOO
    //    O
    ExtraLongCorner,

    // OO
    //  O
    ShortCorner,

    // OO
    // OO
    ShortSquare,

    // OO
    //  OO
    ShortZigZag,

    // O O
    // OOO
    ShortCup,

    //  O
    // OOO
    ShortSpike
}

impl Piece {
    fn get_cell_locations(&self) -> Vec<(usize, usize)> {
        match self {
            Piece::ExtraLongStraight => {
                vec![(0, 0), (1, 0), (2, 0), (3, 0)]
            },
            Piece::LongCorner => {
                vec![(0, 0), (1, 0), (2, 0), (2, 1)]
            },
            Piece::ExtraLongCorner => {
                vec![(0, 0), (1, 0), (2, 0), (3, 0), (3, 1)]
            },
            Piece::ShortCorner => {
                vec![(0, 0), (1, 0), (1, 1)]
            },
            Piece::ShortSquare => {
                vec![(0, 0), (1, 0), (0, 1), (1, 1)]
            },
            Piece::ShortZigZag => {
                vec![(0, 0), (1, 0), (1, 1), (2, 1)]
            },
            Piece::ShortCup => {
                vec![(0, 0), (2, 0), (0, 1), (1, 1), (2, 1)]
            },
            Piece::ShortSpike => {
                vec![(1, 0), (0, 1), (1, 1), (2, 1)]
            }
        }
    }
    fn rotate_cell_locations(cell_locations: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        let mut rotated_cell_locations: Vec<(usize, usize)> = Vec::new();
        let mut largest_height: usize = 0;
        for cell_location in cell_locations.iter() {
            if cell_location.1 > largest_height {
                largest_height = cell_location.1;
            }
        }
        for cell_location in cell_locations.iter() {
            let rotated_cell_location = (largest_height - cell_location.1, cell_location.0);
            rotated_cell_locations.push(rotated_cell_location);
        }
        rotated_cell_locations
    }
    fn flip_cell_locations(cell_locations: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        let mut flipped_cell_locations: Vec<(usize, usize)> = Vec::new();
        let mut largest_width: usize = 0;
        for cell_location in cell_locations.iter() {
            if cell_location.0 > largest_width {
                largest_width = cell_location.0;
            }
        }
        for cell_location in cell_locations.iter() {
            let flipped_cell_location = (largest_width - cell_location.0, cell_location.1);
            flipped_cell_locations.push(flipped_cell_location);
        }
        flipped_cell_locations
    }
}

#[derive(Clone, Hash, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
struct NodeState {
    piece: Piece,
    rotation_index: u8,
    is_flipped: bool,
    location: (usize, usize)
}

impl NodeState {
    fn is_overlapping(&self, other_node_state: &Self) -> bool {
        let mut is_at_least_one_cell_overlapping: bool = false;
        let mut cell_locations: Vec<(usize, usize)> = self.piece.get_cell_locations();
        for _ in 0..self.rotation_index {
            cell_locations = Piece::rotate_cell_locations(cell_locations);
        }
        if self.is_flipped {
            cell_locations = Piece::flip_cell_locations(cell_locations);
        }
        for cell_location_index in 0..cell_locations.len() {
            cell_locations[cell_location_index].0 += self.location.0;
            cell_locations[cell_location_index].1 += self.location.1;
        }
        let mut other_cell_locations: Vec<(usize, usize)> = other_node_state.piece.get_cell_locations();
        for _ in 0..other_node_state.rotation_index {
            other_cell_locations = Piece::rotate_cell_locations(other_cell_locations);
        }
        if other_node_state.is_flipped {
            other_cell_locations = Piece::flip_cell_locations(other_cell_locations);
        }
        for other_cell_location_index in 0..other_cell_locations.len() {
            other_cell_locations[other_cell_location_index].0 += other_node_state.location.0;
            other_cell_locations[other_cell_location_index].1 += other_node_state.location.1;
        }
        'cell_search: {
            for cell_location in cell_locations.iter() {
                for other_cell_location in other_cell_locations.iter() {
                    if cell_location == other_cell_location {
                        is_at_least_one_cell_overlapping = true;
                        break 'cell_search;
                    }
                }
            }
        }
        is_at_least_one_cell_overlapping
    }
    fn fits_within_bounds(&self, size: (usize, usize)) -> bool {
        let mut is_at_least_one_cell_out_of_bounds: bool = false;
        let mut cell_locations: Vec<(usize, usize)> = self.piece.get_cell_locations();
        for _ in 0..self.rotation_index {
            cell_locations = Piece::rotate_cell_locations(cell_locations);
        }
        if self.is_flipped {
            cell_locations = Piece::flip_cell_locations(cell_locations);
        }
        for cell_location_index in 0..cell_locations.len() {
            cell_locations[cell_location_index].0 += self.location.0;
            cell_locations[cell_location_index].1 += self.location.1;
        }
        for cell_location in cell_locations.iter() {
            if cell_location.0 >= size.0 ||
                cell_location.1 >= size.1 {

                is_at_least_one_cell_out_of_bounds = true;
                break;
            }
        }
        !is_at_least_one_cell_out_of_bounds
    }
}

struct Puzzle {
    pieces: Vec<Piece>,
    size: (usize, usize)
}

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
enum Identifier {
    Node(UniquePiece),
    NodeStateCollection(String)
}

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
struct UniquePiece {
    piece: Piece,
    index: usize
}

impl UniquePiece {
    fn get_color(&self) -> [u8; 4] {
        let piece_seed: u64;
        match self.piece {
            Piece::ExtraLongStraight => {
                piece_seed = 0;
            },
            Piece::LongCorner => {
                piece_seed = 1;
            },
            Piece::ExtraLongCorner => {
                piece_seed = 2;
            },
            Piece::ShortCorner => {
                piece_seed = 3;
            },
            Piece::ShortSquare => {
                piece_seed = 4;
            },
            Piece::ShortZigZag => {
                piece_seed = 5;
            },
            Piece::ShortCup => {
                piece_seed = 6;
            },
            Piece::ShortSpike => {
                piece_seed = 7;
            }
        }

        let mut rng = rand::thread_rng();
        let random_seed = Some(rng.gen::<u64>());

        let random_value = 
    }
}

impl Puzzle {
    fn new(pieces: Vec<Piece>, size: (usize, usize)) -> Self {
        Puzzle {
            pieces: pieces,
            size: size
        }
    }
    fn get_wave_function(&self) -> WaveFunction<Identifier, NodeState> {

        let mut nodes: Vec<Node<Identifier, NodeState>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<Identifier, NodeState>> = Vec::new();

        let mut permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index: HashMap<usize, HashMap<NodeState, HashMap<usize, Vec<NodeState>>>> = HashMap::new();

        // get all of the permitted piece orientations and locations given each possible location for all other pieces
        {
            for (piece_index, piece) in self.pieces.iter().enumerate() {
                let mut possible_node_states: Vec<NodeState> = Vec::new();

                for (other_piece_index, other_piece) in self.pieces.iter().enumerate() {
                    if other_piece_index != piece_index {
                        for height_index in 0..self.size.1 {
                            for width_index in 0..self.size.0 {
                                for rotation_index in 0..4 as u8 {
                                    for is_flipped in [false, true] {
                                        let node_state = NodeState {
                                            piece: piece.clone(),
                                            rotation_index: rotation_index,
                                            is_flipped: is_flipped,
                                            location: (width_index, height_index)
                                        };

                                        if node_state.fits_within_bounds(self.size) {

                                            for other_height_index in 0..self.size.1 {
                                                for other_width_index in 0..self.size.0 {
                                                    for other_rotation_index in 0..4 as u8 {
                                                        for other_is_flipped in [false, true] {
                                                            let other_node_state = NodeState {
                                                                piece: other_piece.clone(),
                                                                rotation_index: other_rotation_index,
                                                                is_flipped: other_is_flipped,
                                                                location: (other_width_index, other_height_index)
                                                            };

                                                            if other_node_state.fits_within_bounds(self.size) && !node_state.is_overlapping(&other_node_state) {
                                                                if !permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.contains_key(&piece_index) {
                                                                    permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.insert(piece_index.clone(), HashMap::new());
                                                                }
                                                                if !permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.get(&piece_index).unwrap().contains_key(&node_state) {
                                                                    permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.get_mut(&piece_index).unwrap().insert(node_state.clone(), HashMap::new());
                                                                }
                                                                if !permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.get(&piece_index).unwrap().get(&node_state).unwrap().contains_key(&other_piece_index) {
                                                                    permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.get_mut(&piece_index).unwrap().get_mut(&node_state).unwrap().insert(other_piece_index.clone(), Vec::new());
                                                                }
                                                                permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.get_mut(&piece_index).unwrap().get_mut(&node_state).unwrap().get_mut(&other_piece_index).unwrap().push(other_node_state.clone());
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            possible_node_states.push(node_state.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            debug!("permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index: {:?}", permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index);
        }

        let mut node_id_per_index: Vec<Identifier> = Vec::new();
        let mut current_index_per_piece: HashMap<&Piece, usize> = HashMap::new();
        for piece in self.pieces.iter() {
            if !current_index_per_piece.contains_key(piece) {
                current_index_per_piece.insert(piece, 0);
            }
            let index = current_index_per_piece.get(piece).unwrap().clone();
            current_index_per_piece.insert(piece, index + 1);
            let node_id: Identifier = Identifier::Node(UniquePiece {
                piece: piece.clone(),
                index: index
            });
            node_id_per_index.push(node_id);
        }

        debug!("node_id_per_index: {:?}", node_id_per_index);

        // construct all possible node_state_collection instances
        {
            let mut id_per_anonymous_node_state_collection: HashMap<AnonymousNodeStateCollection<NodeState>, Identifier> = HashMap::new();
            let mut existing_node_state_collections: HashSet<AnonymousNodeStateCollection<NodeState>> = HashSet::new();
            for (node_index, permitted_node_states_per_neighbor_node_index_per_node_state) in permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.into_iter() {
                let node_id = node_id_per_index.get(node_index).unwrap().clone();
                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<Identifier, Vec<Identifier>> = HashMap::new();
                let mut node_states: Vec<NodeState> = Vec::new();
                for (node_state, permitted_node_states_per_neighbor_node_index) in permitted_node_states_per_neighbor_node_index_per_node_state.into_iter() {
                    node_states.push(node_state.clone());
                    for (neighbor_node_index, permitted_node_states) in permitted_node_states_per_neighbor_node_index.into_iter() {
                        let neighbor_node_id = node_id_per_index.get(neighbor_node_index).unwrap().clone();
                        let anonymous_node_state_collection = AnonymousNodeStateCollection {
                            when_node_state: node_state.clone(),
                            then_node_states: permitted_node_states
                        };
                        if !existing_node_state_collections.contains(&anonymous_node_state_collection) {
                            existing_node_state_collections.insert(anonymous_node_state_collection.clone());
                            let node_state_collection_id: Identifier = Identifier::NodeStateCollection(Uuid::new_v4().to_string());
                            id_per_anonymous_node_state_collection.insert(anonymous_node_state_collection.clone(), node_state_collection_id.clone());
                            let node_state_collection = NodeStateCollection::new_from_anonymous(node_state_collection_id, anonymous_node_state_collection.clone());
                            node_state_collections.push(node_state_collection);
                        }
                        let node_state_collection_id: Identifier = id_per_anonymous_node_state_collection.get(&anonymous_node_state_collection).unwrap().clone();
                        if !node_state_collection_ids_per_neighbor_node_id.contains_key(&neighbor_node_id) {
                            node_state_collection_ids_per_neighbor_node_id.insert(neighbor_node_id.clone(), Vec::new());
                        }
                        node_state_collection_ids_per_neighbor_node_id.get_mut(&neighbor_node_id).unwrap().push(node_state_collection_id);
                    }
                }
                debug!("node_id: {:?}", node_id);
                debug!("node_state_collection_ids_per_neighbor_node_id: {:?}", node_state_collection_ids_per_neighbor_node_id);
                let node = Node::new(node_id, NodeStateProbability::get_equal_probability(node_states), node_state_collection_ids_per_neighbor_node_id);
                nodes.push(node);
            }
        }

        let wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        wave_function
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    let pieces: Vec<Piece> = vec![
        Piece::ShortSquare,
        Piece::ShortSquare
    ];
    let size: (usize, usize) = (4, 2);

    let puzzle = Puzzle::new(pieces, size);
    let wave_function = puzzle.get_wave_function();
    let mut collapsable_wave_function = wave_function.get_collapsable_wave_function::<AccommodatingCollapsableWaveFunction<Identifier, NodeState>>(None);
    let collapsed_wave_function = collapsable_wave_function.collapse().unwrap();

    let mut pixels: Vec<Vec<[u8; 4]>> = Vec::new();
    for height_index in 0..size.1 {
        pixels.push(Vec::new());
        for _ in 0..size.0 {
            pixels[height_index].push([0, 0, 0, 0]);
        }
    }

    for (node_id, node_state) in collapsed_wave_function.node_state_per_node.iter() {
        let color = node_id
    }
}