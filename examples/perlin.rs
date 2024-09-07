// This example demonstrates the ProximityGraph abstraction
//  Quest locations must be adjacent to quest givers
//  Dangerous locations must be far from beginner locations

use std::collections::{HashMap, VecDeque};

use colored::{Colorize, ColoredString};
use perlin2d::PerlinNoise2D;
use serde::{Deserialize, Serialize};
use wave_function_collapse::abstractions::proximity_graph::{Distance, HasProximity, Proximity, ProximityGraph, ProximityGraphNode};

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
enum Color {
    Purple,
    Blue,
    Green,
    Yellow,
    Orange,
    Red,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
struct Quest {
    id: usize,
    name: String,
    color: Color,
}

impl PartialEq for Quest {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Quest {
    
}

impl PartialOrd for Quest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Quest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

fn get_quests() -> Vec<Quest> {
    vec![
        Quest {
            id: 1,
            name: String::from("Introduce yourself to neighbor"),
            color: Color::Purple,
        },
        Quest {
            id: 2,
            name: String::from("Find working vehicle"),
            color: Color::Blue,
        },
        Quest {
            id: 3,
            name: String::from("Pick up neighbor"),
            color: Color::Green,
        },
        Quest {
            id: 4,
            name: String::from("Take neighbor to zombie horde"),
            color: Color::Yellow,
        },
        Quest {
            id: 5,
            name: String::from("Get neighbor corpse from zombie pit"),
            color: Color::Orange,
        },
        Quest {
            id: 6,
            name: String::from("Take neighbor's corpse to cage"),
            color: Color::Red,
        },
    ]
}

impl HasProximity for Quest {
    fn get_proximity(&self, other: &Self) -> Proximity where Self: Sized {
        let (smallest_id, largest_id) = if self.id < other.id {
            (self.id, other.id)
        }
        else {
            (other.id, self.id)
        };
        match smallest_id {
            1 => {
                match largest_id {
                    2 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                2.0,
                                0.0,
                            )
                        }
                    },
                    3 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                0.0,
                                0.0,
                            )
                        }
                    },
                    4 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                8.0,
                                0.5,
                            )
                        }
                    },
                    5 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                8.0,
                                0.5,
                            )
                        }
                    },
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                6.0,
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
                    3 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                2.0,
                                0.0,
                            )
                        }
                    },
                    4 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                6.0,
                                0.0,
                            )
                        }
                    },
                    5 => {
                        Proximity::InAnotherDimensionEntirely
                    },
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                1.0,
                                0.0,
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
                    4 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                8.0,
                                0.5,
                            )
                        }
                    },
                    5 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                8.0,
                                0.5,
                            )
                        }
                    },
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                6.0,
                                0.0,
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
                    5 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                0.1,
                                0.0,
                            )
                        }
                    },
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                12.0,
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
                    6 => {
                        Proximity::SomeDistanceAway {
                            distance: Distance::new(
                                12.0,
                                2.0,
                            )
                        }
                    },
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
    fn to_vec_proximity_graph_node(value: Self) -> Vec<ProximityGraphNode>;
}

impl ToVecProximityGraphNode for Vec<Vec<bool>> {
    fn to_vec_proximity_graph_node(value: Self) -> Vec<ProximityGraphNode> {
        let mut proximity_graph_nodes = Vec::new();
        let distances = compute_all_pairs_shortest_paths(&value);
        for from_x in 0..distances.len() {
            for from_y in 0..distances[from_x].len() {
                let from_proximity_graph_node_id = format!("({}, {})", from_x, from_y);
                let mut distance_per_proximity_graph_node_id = HashMap::new();
                for to_x in 0..distances[from_x][from_y].len() {
                    for to_y in 0..distances[from_x][from_y][to_x].len() {
                        if let Some(distance) = distances[from_x][from_y][to_x][to_y] {
                            let to_proximity_graph_node_id = format!("({}, {})", to_x, to_y);
                            distance_per_proximity_graph_node_id.insert(to_proximity_graph_node_id, distance as f32);
                        }
                    }
                }
                let proximity_graph_node = ProximityGraphNode::new(
                    from_proximity_graph_node_id,
                    distance_per_proximity_graph_node_id,
                );
                proximity_graph_nodes.push(proximity_graph_node);
            }
        }
        //for y_from in 0..value.len() {
        //    for x_from in 0..value[y_from].len() {
        //        let from_proximity_graph_node_id = format!("({}, {})", x_from, y_from);
        //        let mut distance_per_proximity_graph_node_id = HashMap::new();
        //        for y_to in 0..value.len() {
        //            for x_to in 0..value[y_to].len() {
        //                if y_to > y_from || y_to == y_from && x_to > x_from {
        //                    println!("comparing ({}, {}) to ({}, {})", y_from, x_from, y_to, x_to);
        //                    let start = (x_from, y_from);
        //                    let destination = (x_to, y_to);
        //                    if let Some(distance) = find_distance(&value, start, destination) {
        //                        let to_proximity_graph_node_id = format!("({}, {})", x_to, y_to);
        //                        distance_per_proximity_graph_node_id.insert(to_proximity_graph_node_id, distance as f32);
        //                    }
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
        proximity_graph_nodes
    }
}

/// Initializes the distance matrix with None or 0 for each cell to itself
fn initialize_distance_matrix(grid: &Vec<Vec<bool>>) -> DistanceMatrix {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut distances = vec![vec![vec![vec![None; cols]; rows]; cols]; rows];

    for x in 0..rows {
        for y in 0..cols {
            distances[x][y][x][y] = Some(0); // Distance from a cell to itself is 0
        }
    }

    distances
}

/// Runs a BFS from a given start point and fills in the distance matrix
fn bfs_fill_distances(
    grid: &Vec<Vec<bool>>,
    start: (usize, usize),
    distances: &mut DistanceMatrix,
) {
    let (start_x, start_y) = start;
    let rows = grid.len();
    let cols = grid[0].len();
    let target_state = grid[start_x][start_y];

    let directions = [(0, 1), (1, 0), (0, isize::wrapping_neg(1)), (isize::wrapping_neg(1), 0)];

    let mut visited = vec![vec![false; cols]; rows];
    visited[start_x][start_y] = true;

    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y, 0));

    while let Some((x, y, dist)) = queue.pop_front() {
        for (dx, dy) in &directions {
            let new_x = x.wrapping_add(*dx as usize);
            let new_y = y.wrapping_add(*dy as usize);

            if new_x < rows
                && new_y < cols
                && !visited[new_x][new_y]
                && grid[new_x][new_y] == target_state
            {
                visited[new_x][new_y] = true;
                distances[start_x][start_y][new_x][new_y] = Some(dist + 1);
                queue.push_back((new_x, new_y, dist + 1));
            }
        }
    }
}

type DistanceMatrix = Vec<Vec<Vec<Vec<Option<usize>>>>>;

/// Computes the shortest paths between all pairs of cells in the grid
fn compute_all_pairs_shortest_paths(grid: &Vec<Vec<bool>>) -> DistanceMatrix {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut distances = initialize_distance_matrix(grid);

    // Initialize direct distances using BFS
    for x in 0..rows {
        for y in 0..cols {
            bfs_fill_distances(grid, (x, y), &mut distances);
        }
    }

    // Floyd-Warshall like update for all-pairs shortest paths
    for k_x in 0..rows {
        for k_y in 0..cols {
            println!("trying ({}, {})", k_x, k_y);
            for x1 in 0..rows {
                for y1 in 0..cols {
                    for x2 in 0..rows {
                        for y2 in 0..cols {
                            if let (Some(d1), Some(d2)) = (
                                distances[x1][y1][k_x][k_y],
                                distances[k_x][k_y][x2][y2],
                            ) {
                                let new_dist = d1 + d2;
                                if distances[x1][y1][x2][y2].is_none()
                                    || new_dist < distances[x1][y1][x2][y2].unwrap()
                                {
                                    distances[x1][y1][x2][y2] = Some(new_dist);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    distances
}

fn main() {
    
    let width: usize = 60;
    let height: usize = 60;
    let quests = get_quests();
    let perlin = PerlinNoise2D::new(
        6,
        1.0,
        1.0,
        1.0,
        2.0,
        (1.0, 1.0),
        0.5,
        101,
    );

    let mut grid = Vec::with_capacity(height);
    let character = "\u{2588}";
    for y in 0..height {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            let noise = perlin.get_noise(x as f64 / width as f64, y as f64 / height as f64);
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
    let nodes = Vec::to_vec_proximity_graph_node(grid);
    println!("created nodes.");
    let proximity_graph = ProximityGraph::new(
        nodes,
    );
    let maximum_acceptable_distance_variance_factor = 10.0;
    let acceptable_distance_variance_factor_difference = 0.1;
    println!("solving proximity graph...");
    let value_per_proximity_graph_node_id = proximity_graph.get_value_per_proximity_graph_node_id(
        quests,
        maximum_acceptable_distance_variance_factor,
        acceptable_distance_variance_factor_difference,
        false,
    )
        .expect("Failed to get values from proximity graph.");
    println!("solved proximity graph.");
    
    for y in 0..height {
        for x in 0..width {
            let proximity_graph_node_id = format!("({}, {})", x, y);
            let noise = perlin.get_noise(x as f64 / width as f64, y as f64 / height as f64);

            //println!("({}, {}) = {}", x, y, noise);
            let colored_character;
            if noise < 0.0 {
                if let Some(value) = value_per_proximity_graph_node_id.get(&proximity_graph_node_id) {
                    colored_character = match value.color {
                        Color::Orange => character.bright_red(),
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