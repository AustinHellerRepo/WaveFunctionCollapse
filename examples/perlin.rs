// This example demonstrates the ProximityGraph abstraction
//  Quest locations must be adjacent to quest givers
//  Dangerous locations must be far from beginner locations

use std::collections::{HashMap, HashSet, VecDeque};

use colored::Colorize;
use perlin2d::PerlinNoise2D;
use serde::{Deserialize, Serialize};
use wave_function_collapse::abstractions::proximity_graph::{Distance, HasProximity, Proximity, ProximityGraph, ProximityGraphNode};

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
enum Color {
    Black,
    Purple,
    Blue,
    Green,
    Yellow,
    Orange,
    Red,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
struct QuestDestination {
    id: usize,
    name: String,
    color: Color,
}

impl PartialEq for QuestDestination {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for QuestDestination {
    
}

impl PartialOrd for QuestDestination {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for QuestDestination {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

fn get_quests() -> Vec<QuestDestination> {
    vec![
        QuestDestination {
            id: 0,
            name: String::from("Unvisited"),
            color: Color::Black,
        },
        QuestDestination {
            id: 1,
            name: String::from("Player house"),
            color: Color::Purple,
        },
        QuestDestination {
            id: 2,
            name: String::from("Neighbor house"),
            color: Color::Blue,
        },
        QuestDestination {
            id: 3,
            name: String::from("Abandoned vehicle"),
            color: Color::Green,
        },
        QuestDestination {
            id: 4,
            name: String::from("Known zombie horde"),
            color: Color::Yellow,
        },
        QuestDestination {
            id: 5,
            name: String::from("Abandoned warehouse"),
            color: Color::Orange,
        },
        QuestDestination {
            id: 6,
            name: String::from("Enemy base"),
            color: Color::Red,
        },
    ]
}

impl HasProximity for QuestDestination {
    fn get_proximity(&self, other: &Self) -> Proximity where Self: Sized {
        let (smallest_id, largest_id) = if self.id < other.id {
            (self.id, other.id)
        }
        else {
            (other.id, self.id)
        };
        //println!("getting proximity between {} and {}", smallest_id, largest_id);
        let scalar = 4.0;
        match smallest_id {
            0 => {
                Proximity::InAnotherDimensionEntirely
            },
            1 => {
                match largest_id {
                    1 => Proximity::ExclusiveExistence,
                    2 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                2.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    3 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                3.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    4 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                8.0 * scalar,
                                0.5,
                            )
                        }
                    },
                    5 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                4.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                6.0 * scalar,
                                0.0,
                            )
                        }
                    },
                    _ => {
                        panic!("Unexpected quest.");
                    },
                }
            },
            2 => {
                match largest_id {
                    2 => Proximity::ExclusiveExistence,
                    3 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                3.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    4 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                9.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    5 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                4.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                6.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    _ => {
                        panic!("Unexpected quest.");
                    }
                }
            },
            3 => {
                match largest_id {
                    3 => Proximity::ExclusiveExistence,
                    4 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                10.0 * scalar,
                                2.0,
                            )
                        }
                    },
                    5 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                3.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                9.0 * scalar,
                                1.0,
                            )
                        }
                    },
                    _ => {
                        panic!("Unexpected quest.");
                    }
                }
            },
            4 => {
                match largest_id {
                    4 => Proximity::ExclusiveExistence,
                    5 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                14.0 * scalar,
                                2.0,
                            )
                        }
                    },
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                20.0 * scalar,
                                2.0,
                            )
                        }
                    },
                    _ => {
                        panic!("Unexpected quest.");
                    }
                }
            },
            5 => {
                match largest_id {
                    5 => Proximity::ExclusiveExistence,
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                12.0 * scalar,
                                4.0,
                            )
                        }
                    },
                    _ => {
                        panic!("Unexpected quest.");
                    }
                }
            },
            6 => {
                match largest_id {
                    6 => Proximity::ExclusiveExistence,
                    _ => {
                        panic!("Unexpected quest.");
                    }
                }
            },
            _ => {
                panic!("Unexpected quest.");
            }
        }
    }
}

trait ToVecProximityGraphNode {
    type TTag: Clone;

    fn to_vec_proximity_graph_node(value: Self, nodes_length: usize, random_seed: Option<u64>) -> Vec<ProximityGraphNode<Self::TTag>>;
}

impl ToVecProximityGraphNode for Vec<Vec<bool>> {
    type TTag = (usize, usize);

    fn to_vec_proximity_graph_node(value: Self, nodes_length: usize, random_seed: Option<u64>) -> Vec<ProximityGraphNode<Self::TTag>> {
        let mut proximity_graph_nodes = Vec::new();

        //let distances = compute_all_pairs_shortest_paths(&value);
        //for from_x in 0..distances.len() {
        //    for from_y in 0..distances[from_x].len() {
        //        let from_proximity_graph_node_id = format!("({}, {})", from_x, from_y);
        //        let mut distance_per_proximity_graph_node_id = HashMap::new();
        //        for to_x in 0..distances[from_x][from_y].len() {
        //            for to_y in 0..distances[from_x][from_y][to_x].len() {
        //                if let Some(distance) = distances[from_x][from_y][to_x][to_y] {
        //                    let to_proximity_graph_node_id = format!("({}, {})", to_x, to_y);
        //                    distance_per_proximity_graph_node_id.insert(to_proximity_graph_node_id, distance as f32);
        //                }
        //            }
        //        }
        //        let proximity_graph_node = ProximityGraphNode::new(
        //            from_proximity_graph_node_id,
        //            distance_per_proximity_graph_node_id,
        //        );
        //        proximity_graph_nodes.push(proximity_graph_node);
        //    }
        //}

        // grab random locations
        //let debug_location = (31, 36);
        let mut excluded_locations = HashSet::new();
        let mut included_locations = Vec::new();
        let mut is_at_least_one_new_location_excluded = true;
        while is_at_least_one_new_location_excluded {
            //println!("starting with {} locations", included_locations.len());
            proximity_graph_nodes.clear();
            is_at_least_one_new_location_excluded = false;
            let locations = {
                if let Some(random_seed) = &random_seed {
                    fastrand::seed(*random_seed);
                }
                let mut locations = included_locations.clone();
                while locations.len() < nodes_length {
                    let y = fastrand::usize(0..value.len());
                    let x = fastrand::usize(0..value[y].len());
                    let location = (x, y);
                    if !excluded_locations.contains(&location) && !locations.contains(&location) {
                        if value[y][x] {
                            locations.push(location);
                        }
                    }
                }
                locations
            };
            for (from_location_index, from_location) in locations.iter().enumerate() {
                let from_proximity_graph_node_id = format!("{}", from_location_index);
                let mut distance_per_proximity_graph_node_id = HashMap::new();
                let mut failed_to_find_locations = Vec::new();
                for (to_location_index, to_location) in locations.iter().enumerate() {
                    if to_location_index != from_location_index {
                        if let Some(distance) = find_distance(&value, *from_location, *to_location) {
                            let to_proximity_graph_node_id = format!("{}", to_location_index);
                            distance_per_proximity_graph_node_id.insert(to_proximity_graph_node_id, distance as f32);
                        }
                        else {
                            failed_to_find_locations.push(*to_location);
                        }
                    }
                }
                if (failed_to_find_locations.len() as f32) > nodes_length as f32 * 0.5 {
                    if !included_locations.contains(from_location) {
                        //println!("excluding current location ({}, {})", from_location.0, from_location.1);
                        excluded_locations.insert(*from_location);
                        is_at_least_one_new_location_excluded = true;
                    }
                    //println!("excluding {} locations", failed_to_find_locations.len());
                    //excluded_locations.extend(failed_to_find_locations.drain(..));
                    //is_at_least_one_new_location_excluded = true;
                }
                //else {
                //    if !included_locations.contains(from_location) {
                //        println!("excluding current location");
                //        excluded_locations.insert(*from_location);
                //        is_at_least_one_new_location_excluded = true;
                //    }
                //}
                let proximity_graph_node = ProximityGraphNode::new(
                    from_proximity_graph_node_id,
                    distance_per_proximity_graph_node_id,
                    *from_location,
                );
                proximity_graph_nodes.push(proximity_graph_node);
            }

            //let original_length = included_locations.len();
            for location in locations.into_iter() {
                if !excluded_locations.contains(&location) {
                    included_locations.push(location);
                }
            }
            //println!("included {} new locations", included_locations.len() - original_length);
        }
        proximity_graph_nodes
    }
}

fn find_distance(
    grid: &Vec<Vec<bool>>,
    start: (usize, usize),
    destination: (usize, usize),
) -> Option<usize> {
    // Check if start or destination is out of bounds
    let rows = grid.len();
    let cols = grid[0].len();
    let (x1, y1) = start;
    let (x2, y2) = destination;

    if y1 >= rows || x1 >= cols || y2 >= rows || x2 >= cols {
        return None;
    }

    // Get the state of the start cell
    let target_state = grid[y1][x1];

    // Ensure the destination cell is of the same state
    if grid[y2][x2] != target_state {
        return None;
    }

    // Directions for moving up, down, left, and right
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    // Track visited cells
    let mut visited = vec![vec![false; cols]; rows];
    visited[y1][x1] = true;

    // Queue for BFS: stores the point and the distance from the start
    let mut queue = VecDeque::new();
    queue.push_back((y1, x1, 0));

    // Perform BFS
    while let Some((y, x, distance)) = queue.pop_front() {
        // If we've reached the destination, return the distance
        if (x, y) == (x2, y2) {
            return Some(distance);
        }

        // Explore all 4 possible directions
        for (dx, dy) in &directions {
            let new_x = (x as isize + dx) as usize;
            let new_y = (y as isize + dy) as usize;

            // Ensure new coordinates are within bounds and match the target state
            if new_x < rows
                && new_y < cols
                && !visited[new_y][new_x]
                && grid[new_y][new_x] == target_state
            {
                visited[new_y][new_x] = true;
                queue.push_back((new_y, new_x, distance + 1));
            }
        }
    }

    // If the queue is exhausted and the destination wasn't reached, return None
    None
}

fn main() {

    println!("The following example showcases how to place locations within a dynamic environment.");
    println!("The player house should be nearby the neighbor house.");
    println!("The abandoned vehicle should be nearby the houses.");
    println!("The warehouse should be somewhat further away.");
    println!("The zombie horde and enemy base should be far from the houses.");
    println!("The enemy base should be very far from the zombie horde.");
    
    let node_sample_length = 18;
    let width: usize = 40;
    let height: usize = 40;
    let quests = get_quests();
    let seed = fastrand::i32(0..10000);
    //let seed = -2074058151;
    println!("seed: {}", seed);
    let perlin = PerlinNoise2D::new(
        6,
        1.0,
        1.0,
        2.0,
        2.0,
        (1.0, 1.0),
        -3.0,
        seed,
    );

    let mut grid = Vec::with_capacity(height);
    let character = "\u{2588}";
    for y in 0..height {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            let noise_x = x as f64 / width as f64;
            let noise_y = y as f64 / height as f64;
            //println!("noise: ({}, {})", noise_x, noise_y);
            let noise = perlin.get_noise(noise_x, noise_y);
            let is_ground = noise < 0.0;

            row.push(is_ground);

            //println!("({}, {}) = {}", x, y, noise);
            let colored_character;
            if noise < 0.0 {
                colored_character = character.black();
            }
            else {
                colored_character = character.white();
            }
            print!("{}{}", colored_character, colored_character);
        }
        grid.push(row);
        println!();
    }

    //let nodes = grid.to_vec_proximity_graph_node();
    println!("creating nodes...");
    let nodes = Vec::to_vec_proximity_graph_node(grid, node_sample_length, None);
    println!("created nodes.");
    let proximity_graph = ProximityGraph::new(
        nodes.clone(),
    );
    let maximum_acceptable_distance_variance_factor = 10.0;
    let acceptable_distance_variance_factor_difference = 1.0;
    println!("solving proximity graph...");
    let value_per_proximity_graph_node_id = proximity_graph.get_value_per_proximity_graph_node_id(
        quests,
        maximum_acceptable_distance_variance_factor,
        acceptable_distance_variance_factor_difference,
        false,
    )
        .expect("Failed to get values from proximity graph.");
    println!("solved proximity graph.");
    
    let color_at_location = {
        let mut color_at_location = HashMap::new();
        for node in nodes.iter() {
            if let Some(value) = value_per_proximity_graph_node_id.get(node.get_id()) {
                let location = node.get_tag();
                match value.color {
                    Color::Black => {
                        // do nothing
                    },
                    _ => {
                        println!("found {:?} {} at {:?}", value.color, value.name, location);
                    }
                }
                color_at_location.insert(*location, value.color);
            }
        }
        color_at_location
    };
    for y in 0..height {
        for x in 0..width {
            let noise = perlin.get_noise(x as f64 / width as f64, y as f64 / height as f64);

            //println!("({}, {}) = {}", x, y, noise);
            let colored_character;
            if noise < 0.0 {
                if let Some(color) = color_at_location.get(&(x, y)) {
                    colored_character = match color {
                        Color::Black => character.black(),
                        Color::Orange => character.custom_color(colored::CustomColor::new(255, 128, 0)),
                        Color::Yellow => character.yellow(),
                        Color::Green => character.green(),
                        Color::Blue => character.blue(),
                        Color::Purple => character.purple(),
                        Color::Red => character.red(),
                    };
                }
                else {
                    colored_character = character.black();
                }
            }
            else {
                colored_character = character.white();
            }
            print!("{}{}", colored_character, colored_character);
        }
        println!();
    }
}