use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use wave_function_collapse::wave_function::{WaveFunction, Node, NodeStateCollection, AnonymousNodeStateCollection};



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
struct UniquePiece {
    piece: Piece,
    index: usize
}

impl Puzzle {
    fn get_wave_function(&self) -> WaveFunction<UniquePiece, NodeState> {

        let mut nodes: Vec<Node<UniquePiece, NodeState>> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection<UniquePiece, NodeState>> = Vec::new();

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

                                                            if !node_state.is_overlapping(&other_node_state) {
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
        }

        let mut node_id_per_index: Vec<String> = Vec::new();
        for _ in self.pieces.iter() {
            let node_id: String = Uuid::new_v4().to_string();
            node_id_per_index.push(node_id);
        }


        // construct all possible node_state_collection instances
        {
            let mut existing_node_state_collections: HashSet<AnonymousNodeStateCollection<NodeState>> = HashSet::new();
            for (node_index, permitted_node_states_per_neighbor_node_index_per_node_state) in permitted_node_states_per_neighbor_node_index_per_node_state_per_node_index.into_iter() {
                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                for (node_state, permitted_node_states_per_neighbor_node_index) in permitted_node_states_per_neighbor_node_index_per_node_state.into_iter() {
                    for (neighbor_node_index, permitted_node_states) in permitted_node_states_per_neighbor_node_index.into_iter() {
                        let anonymous_node_state_collection = AnonymousNodeStateCollection {
                            when_node_state: node_state.clone(),
                            then_node_states: permitted_node_states
                        };
                    }
                }
            }
        }


        let wave_function = WaveFunction::new(nodes, node_state_collections);

        wave_function.validate().unwrap();

        wave_function
    }
}

fn main() {


}