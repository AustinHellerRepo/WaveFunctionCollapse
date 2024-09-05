// This example demonstrates the ProximityGraph abstraction
//  Quest locations must be adjacent to quest givers
//  Dangerous locations must be far from beginner locations

use colored::{Colorize, ColoredString};
use serde::{Deserialize, Serialize};
use wave_function_collapse::abstractions::proximity_graph::{Distance, HasProximity, Proximity};

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

fn main() {
    
    let quests = get_quests();

}