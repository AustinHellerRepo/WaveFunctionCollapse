// this abstraction is a web of nodes that have a center and specific states are expected to be closer to each other than further away
// you can imagine a game needing points of interest that are nearby each other - you would not want quest-adjacent locations to be physically distant

use std::collections::HashMap;
use std::hash::Hash;
use serde::{Deserialize, Serialize};
use crate::wave_function::collapsable_wave_function::collapsable_wave_function::CollapsableWaveFunction;
use crate::wave_function::collapsable_wave_function::sequential_collapsable_wave_function::SequentialCollapsableWaveFunction;
use crate::wave_function::{Node, NodeStateCollection, NodeStateProbability, WaveFunction};

#[derive(Clone, Copy)]
pub enum RangeType {
    // the values are qualitatively the same
    Inclusive,
    // the values are qualitatively different
    Exclusive,
}

pub struct Distance {
    // the center of the point that the values are quantifiable
    center: f32,
    // the distance from the center that they are reasonably still the same
    width: f32,
    // if the region defined by the center and width are inclusive of the two value or if they are explicitly exclusive
    // having an Exclusive distance is helpful if there is a large Inclusive range but one small area that is always negated
    range_type: RangeType,
}

pub enum Proximity {
    // this indicates that more than one cannot exist
    ExclusiveExistence,
    // the values are different from each other in a quantifiable way
    SomeDistanceAway {
        // the list of distances in priority order
        distances: Vec<Distance>,
    },
    // the values are not related at all and are unquantifiably different
    InAnotherDimensionEntirely,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize)]
enum NodeState<TValue>
where
    TValue: HasProximity,
{
    // the states that make up the web states
    Primary {
        state: TValue,
    },
    // the states that ensure that there is exactly one instance of the primary state equivalent
    Secondary {
        state: TValue,
        node_index: usize,
    },
}

impl<'de, TValue> Deserialize<'de> for NodeState<TValue>
where
    TValue: HasProximity, // Ensure TValue implements Deserialize
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a visitor struct that will help us with deserialization
        struct NodeStateVisitor<TValue>(std::marker::PhantomData<TValue>);

        // Implement Visitor for NodeStateVisitor
        impl<'de, TValue> serde::de::Visitor<'de> for NodeStateVisitor<TValue>
        where
            TValue: HasProximity,
        {
            type Value = NodeState<TValue>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid NodeState variant")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut state: Option<TValue> = None;
                let mut node_index: Option<usize> = None;

                // Iterate over the key-value pairs in the map
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "state" => {
                            if state.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state = Some(map.next_value()?);
                        }
                        "node_index" => {
                            if node_index.is_some() {
                                return Err(serde::de::Error::duplicate_field("node_index"));
                            }
                            node_index = Some(map.next_value()?);
                        }
                        _ => {
                            return Err(serde::de::Error::unknown_field(&key, &["state", "node_index"]));
                        }
                    }
                }

                // Determine which variant to return based on the presence of fields
                match (state, node_index) {
                    (Some(state), None) => Ok(NodeState::Primary { state }),
                    (Some(state), Some(node_index)) => Ok(NodeState::Secondary { state, node_index }),
                    _ => Err(serde::de::Error::missing_field("state")),
                }
            }
        }

        // Call the deserializer with the visitor
        deserializer.deserialize_struct(
            "NodeState",
            &["state", "node_index"],
            NodeStateVisitor(std::marker::PhantomData),
        )
    }
}

pub trait HasProximity: Eq + Hash + Clone + std::fmt::Debug + Ord + Serialize + for<'de> Deserialize<'de> {
    fn get_proximity(&self, other: &Self) -> Proximity where Self: Sized;
}

#[derive(std::fmt::Debug)]
pub struct ProximityGraphNode {
    proximity_graph_node_id: String,
    distance_per_proximity_graph_node_id: HashMap<String, f32>,
}

impl ProximityGraphNode {
    pub fn new(proximity_graph_node_id: String, distance_per_proximity_graph_node_id: HashMap<String, f32>) -> Self {
        Self {
            proximity_graph_node_id,
            distance_per_proximity_graph_node_id,
        }
    }
}

#[derive(std::fmt::Debug, Clone)]
pub enum ProximityGraphError {
    FailedToMapValuesToNodesAtAnyDistance,
    TestError,
}

pub struct ProximityGraph {
    nodes: Vec<ProximityGraphNode>,
}

impl ProximityGraph {
    pub fn new(nodes: Vec<ProximityGraphNode>) -> Self {
        Self {
            nodes,
        }
    }
    pub fn get_value_per_proximity_graph_node_id<TValue: HasProximity>(&self, values: Vec<TValue>, maximum_acceptable_distance_variance_factor: f32, acceptable_distance_variance_factor_difference: f32, is_distance_normalized: bool) -> Result<HashMap<String, TValue>, ProximityGraphError> {

        // iterate over the construction and collapsing of the wave function until the best solution is found
        // first start with the maximum distance being acceptable to ensure that the values can collapse at all
        // if they can collapse, then begin to binary-search for the optimal configuration by restricting what is an acceptable maximum proximity
        //      ex: divide in half first, too low? then make it 75% of original maximum, still too low? make it between 75%-100%, etc.

        let maximum_value_proximity = {
            if is_distance_normalized {
                let mut maximum_value_proximity = maximum_acceptable_distance_variance_factor;
                for i in 0..(values.len() - 1) {
                    for j in (i + 1)..values.len() {
                        maximum_value_proximity += match values[i].get_proximity(&values[j]) {
                            Proximity::ExclusiveExistence => 0.0,
                            Proximity::SomeDistanceAway { distances } => {
                                let mut max_distance = 0.0;
                                for distance in distances.into_iter() {
                                    let current_distance = distance.center + distance.width;
                                    if current_distance > max_distance {
                                        max_distance = current_distance;
                                    }
                                }
                                max_distance
                            },
                            Proximity::InAnotherDimensionEntirely => 0.0,
                        };
                    }
                }
                Some(maximum_value_proximity)
            }
            else {
                None
            }
        };

        let maximum_node_distance = {
            if is_distance_normalized {
                let mut maximum_node_distance = 0.0;
                for node in self.nodes.iter() {
                    for (_, distance) in node.distance_per_proximity_graph_node_id.iter() {
                        let distance = *distance;
                        if distance > maximum_node_distance {
                            maximum_node_distance = distance;
                        }
                    }
                }
                Some(maximum_node_distance)
            }
            else {
                None
            }
        };

        let mut distance_variance_factor = 0.0;
        let mut distance_variance_factor_minimum = 0.0;
        let mut distance_variance_factor_maximum = 0.0;
        let mut best_collapsed_wave_function = None;
        let mut is_distance_variance_factor_acceptable = false;
        let mut iterations = 0;
        while best_collapsed_wave_function.is_none() || !is_distance_variance_factor_acceptable {
            {
                let best_is_what = if best_collapsed_wave_function.is_some() {
                    "some"
                }
                else {
                    "none"
                };
                println!("best is {} from {} to {} while at {}", best_is_what, distance_variance_factor_minimum, distance_variance_factor_maximum, distance_variance_factor);
            }
            let primary_node_state_ratio_per_node_state_id = {
                let node_state_ids = values.iter()
                    .map(|value| {
                        NodeState::Primary {
                            state: value.clone(),
                        }
                    })
                    .collect::<Vec<NodeState<TValue>>>();
                NodeStateProbability::get_equal_probability(&node_state_ids)
            };

            let (nodes, node_state_collections) = {
                let mut nodes = Vec::new();
                let mut node_state_collections = Vec::new();

                // create primary nodes
                for proximity_graph_node in self.nodes.iter() {
                    // setup the NodeStateCollections per neighbor
                    let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                    for (neighbor_proximity_graph_node_id, neighbor_distance) in proximity_graph_node.distance_per_proximity_graph_node_id.iter() {
                        let normalized_neighbor_distance = if let Some(maximum_node_distance) = maximum_node_distance {
                            *neighbor_distance / maximum_node_distance
                        }
                        else {
                            *neighbor_distance
                        };

                        let mut node_state_collection_ids: Vec<String> = Vec::new();
                        if &proximity_graph_node.proximity_graph_node_id != neighbor_proximity_graph_node_id {
                            // collect up each node state
                            for (current_value_index, current_value) in values.iter().enumerate() {
                                let current_node_state = NodeState::Primary {
                                    state: current_value.clone(),
                                };
                                let mut other_node_states = Vec::new();
                                for other_value in values.iter() {
                                    match current_value.get_proximity(other_value) {
                                        Proximity::ExclusiveExistence => {
                                            // do not add the current node state as being able to be in the same final result as this other node state
                                        },
                                        Proximity::SomeDistanceAway { distances } => {
                                            let mut is_inclusive = None;
                                            'look_for_valid_distance: {
                                                for distance in distances.into_iter() {
                                                    let normalized_distance = {
                                                        if let Some(maximum_value_proximity) = maximum_value_proximity {
                                                            let center = distance.center / maximum_value_proximity;
                                                            let width = distance.width * center;  // scale the width by the same factor as the center
                                                            Distance {
                                                                center,
                                                                width,
                                                                range_type: distance.range_type,
                                                            }
                                                        }
                                                        else {
                                                            distance
                                                        }
                                                    };

                                                    let distance_variance = normalized_distance.center * distance_variance_factor;
                                                    let from_distance = normalized_distance.center - distance_variance - normalized_distance.width;
                                                    let to_distance = normalized_distance.center + distance_variance + normalized_distance.width;

                                                    //println!("checking that {} is between {} and {}", normalized_neighbor_distance, from_distance, to_distance);
                                                    if from_distance <= normalized_neighbor_distance && normalized_neighbor_distance <= to_distance {
                                                        match normalized_distance.range_type {
                                                            RangeType::Inclusive => {
                                                                is_inclusive = Some(true);
                                                                break 'look_for_valid_distance;
                                                            },
                                                            RangeType::Exclusive => {
                                                                is_inclusive = Some(false);
                                                                break 'look_for_valid_distance;
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if let Some(is_inclusive) = is_inclusive {
                                                if is_inclusive {
                                                    // this neighbor is within range of being in this other state
                                                    let other_node_state = NodeState::Primary {
                                                        state: other_value.clone(),
                                                    };
                                                    other_node_states.push(other_node_state);
                                                }
                                            }
                                        },
                                        Proximity::InAnotherDimensionEntirely => {
                                            // this neighbor being in this other state has no affect on the current node's state
                                            let other_node_state = NodeState::Primary {
                                                state: other_value.clone(),
                                            };
                                            other_node_states.push(other_node_state);
                                        },
                                    }
                                }

                                // store the results
                                let node_state_collection_id: String = format!("primary_{}_{}_{}", proximity_graph_node.proximity_graph_node_id, neighbor_proximity_graph_node_id, current_value_index);

                                let node_state_collection = NodeStateCollection::new(
                                    node_state_collection_id.clone(),
                                    current_node_state,
                                    other_node_states,
                                );
                                node_state_collections.push(node_state_collection);

                                node_state_collection_ids.push(node_state_collection_id);
                            }
                        }

                        let neighbor_node_id = format!("primary_{}", neighbor_proximity_graph_node_id);
                        node_state_collection_ids_per_neighbor_node_id.insert(neighbor_node_id, node_state_collection_ids);
                    }

                    let node = Node::new(
                        format!("primary_{}", proximity_graph_node.proximity_graph_node_id),
                        primary_node_state_ratio_per_node_state_id.clone(),
                        node_state_collection_ids_per_neighbor_node_id,
                    );
                    nodes.push(node);
                }

                // create secondary nodes
                for (value_index, value) in values.iter().enumerate() {
                    if let Proximity::ExclusiveExistence = value.get_proximity(&value) {
                        // this value needs to only exist exactly once
                        let secondary_node_state_ratio_per_node_state_id = {
                            let mut node_states = Vec::new();
                            for (node_index, _) in self.nodes.iter().enumerate() {
                                node_states.push(
                                    NodeState::Secondary {
                                        node_index,
                                        state: value.clone(),
                                    }
                                );
                            };
                            NodeStateProbability::get_equal_probability(&node_states)
                        };
                        let node_state_collection_ids_per_neighbor_node_id = {
                            let mut node_state_collection_ids_per_neighbor_node_id = HashMap::new();
                            for (proximity_graph_node_index, proximity_graph_node) in self.nodes.iter().enumerate() {
                                let node_state_collection_id = format!("secondary_{}_{}", value_index, proximity_graph_node.proximity_graph_node_id);
                                let node_state_collection = NodeStateCollection::new(
                                    node_state_collection_id.clone(),
                                    NodeState::Secondary {
                                        node_index: proximity_graph_node_index,
                                        state: value.clone(),
                                    },
                                    vec![NodeState::Primary {
                                        state: value.clone(),
                                    }],
                                );
                                node_state_collections.push(node_state_collection);
                                let neighbor_node_id = format!("primary_{}", proximity_graph_node.proximity_graph_node_id);
                                node_state_collection_ids_per_neighbor_node_id.insert(neighbor_node_id, vec![node_state_collection_id]);
                            }
                            node_state_collection_ids_per_neighbor_node_id
                        };
                        let node = Node::new(
                            format!("secondary_{}", value_index),
                            secondary_node_state_ratio_per_node_state_id,
                            node_state_collection_ids_per_neighbor_node_id,
                        );
                        nodes.push(node);
                    }
                }

                // return results
                (nodes, node_state_collections)
            };

            println!("nodes: {}", nodes.len());
            println!("node_state_collections: {}", node_state_collections.len());

            let wave_function = WaveFunction::new(nodes, node_state_collections);
            let mut collapsable_wave_function = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<NodeState<TValue>>>(None);
            match collapsable_wave_function.collapse() {
                Ok(collapsed_wave_function) => {
                    // store this as the best collapsed wave function
                    best_collapsed_wave_function = Some(collapsed_wave_function);

                    // we need to reduce the variances to better isolate an ideal solution
                    distance_variance_factor_maximum = distance_variance_factor;
                    distance_variance_factor = (distance_variance_factor_maximum + distance_variance_factor_minimum) * 0.5;

                    if distance_variance_factor_maximum - distance_variance_factor_minimum <= acceptable_distance_variance_factor_difference {
                        is_distance_variance_factor_acceptable = true;
                        println!("collapsed and found at ({}-{}) at {}", distance_variance_factor_minimum, distance_variance_factor_maximum, distance_variance_factor);
                    }
                    else {
                        println!("collapsed but {} - {} is not less than {}", distance_variance_factor_maximum, distance_variance_factor_minimum, acceptable_distance_variance_factor_difference);
                    }
                },
                Err(_) => {
                    // expand or retract the distance variance
                    // if the distance variance is beyond some measure of the maximum value proximity versus the maximum node distance, return Err
                    if distance_variance_factor_maximum == 0.0 {
                        // if we haven't expanded yet, let's start at the maximum acceptable variance
                        distance_variance_factor_maximum = maximum_acceptable_distance_variance_factor;
                        distance_variance_factor = maximum_acceptable_distance_variance_factor;
                    }
                    else if distance_variance_factor_maximum == maximum_acceptable_distance_variance_factor {
                        // if we just tried the maximum acceptable distance difference factor, we will never find an acceptable factor
                        return Err(ProximityGraphError::FailedToMapValuesToNodesAtAnyDistance);
                    }
                    else {
                        distance_variance_factor_minimum = distance_variance_factor;
                        distance_variance_factor = (distance_variance_factor_maximum + distance_variance_factor_minimum) * 0.5;
                    }

                    if distance_variance_factor_maximum - distance_variance_factor_minimum <= acceptable_distance_variance_factor_difference {
                        is_distance_variance_factor_acceptable = true;
                        println!("not collapsed and found at ({}-{}) at {}", distance_variance_factor_minimum, distance_variance_factor_maximum, distance_variance_factor);
                    }
                    else {
                        println!("not collapsed but {} - {} is not less than {}", distance_variance_factor_maximum, distance_variance_factor_minimum, acceptable_distance_variance_factor_difference);
                    }
                },
            }

            //return Err(ProximityGraphError::TestError);
        
            iterations += 1;
            if iterations > 10 {
                break;
            }
        }
        
        let best_collapsed_wave_function = best_collapsed_wave_function.expect("We should have already failed when both extremes were tested earlier in the logic.");
        let mut value_per_proximity_graph_node_id = HashMap::new();
        for (node_id, node_state) in best_collapsed_wave_function.node_state_per_node_id {
            match node_state {
                NodeState::Primary { state } => {
                    if let Some(proximity_graph_node_id) = node_id.strip_prefix("primary_") {
                        value_per_proximity_graph_node_id.insert(String::from(proximity_graph_node_id), state);
                    }
                    else {
                        panic!("Unexpected non-primary node ID when node state is in a primary state.");
                    }
                },
                NodeState::Secondary { state: _, node_index: _ } => {
                    if let Some(_) = node_id.strip_prefix("primary_") {
                        panic!("Unexpected secondary node state tied to a primary node.");
                    }
                },
            }
        }
        Ok(value_per_proximity_graph_node_id)
    }
}

#[cfg(test)]
mod proximity_graph_tests {
    // TODO create unit tests

    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use super::{Distance, HasProximity, Proximity, ProximityGraph, ProximityGraphNode, RangeType};

    fn get_x_by_y_grid_proximity_graph(x: usize, y: usize) -> ProximityGraph {
        let mut proximity_graph_nodes = Vec::new();
        for i in 0..x {
            for j in 0..y {
                let mut distance_per_proximity_graph_node_id = HashMap::new();
                for i_other in 0..x {
                    for j_other in 0..y {
                        if i != i_other || j != j_other {
                            let other_proximity_graph_node_id = format!("node_{}_{}", i_other, j_other);
                            let distance = (
                                if i < i_other {
                                    i_other - i
                                }
                                else {
                                    i - i_other
                                } + if j < j_other {
                                    j_other - j
                                }
                                else {
                                    j - j_other
                                }
                            ) as f32;
                            distance_per_proximity_graph_node_id.insert(other_proximity_graph_node_id, distance);
                        }
                    }
                }
                let proximity_graph_node = ProximityGraphNode {
                    proximity_graph_node_id: format!("node_{}_{}", i, j),
                    distance_per_proximity_graph_node_id,
                };
                proximity_graph_nodes.push(proximity_graph_node);
            }
        }
        ProximityGraph::new(proximity_graph_nodes)
    }

    fn get_values(total_values: usize) -> Vec<IceCreamShop> {
        let mut values = Vec::with_capacity(total_values);
        for index in 0..total_values {
            match index {
                0 => values.push(IceCreamShop::AppleCream),
                1 => values.push(IceCreamShop::BananaBoost),
                2 => values.push(IceCreamShop::CaramelJuice),
                3 => values.push(IceCreamShop::DarkDestiny),
                4 => values.push(IceCreamShop::EternalJoy),
                _ => values.push(IceCreamShop::None),
            }
        }
        values
    }

    fn println_value_per_proximity_graph_node_id(x: usize, y: usize, value_per_proximity_graph_node_id: &HashMap<String, IceCreamShop>) {
        let mut character_per_y_per_x = HashMap::new();
        for i in 0..x {
            let mut character_per_y = HashMap::new();
            for j in 0..y {
                character_per_y.insert(j, None);
            }
            character_per_y_per_x.insert(i, character_per_y);
        }
        for (proximity_graph_node_id, ice_cream_shop) in value_per_proximity_graph_node_id.iter() {
            let x_and_y: Vec<&str> = proximity_graph_node_id.strip_prefix("node_")
                .unwrap()
                .split('_')
                .collect();
            let x: usize = x_and_y[0].parse().unwrap();
            let y: usize = x_and_y[1].parse().unwrap();
            let character = match ice_cream_shop {
                IceCreamShop::AppleCream => "A",
                IceCreamShop::BananaBoost => "B",
                IceCreamShop::CaramelJuice => "C",
                IceCreamShop::DarkDestiny => "D",
                IceCreamShop::EternalJoy => "E",
                IceCreamShop::None => "_",
            };
            *character_per_y_per_x.get_mut(&x)
                .unwrap()
                .get_mut(&y)
                .unwrap() = Some(character);
        }

        for j in 0..y {
            let mut line = String::new();
            for i in 0..x {
                let character = character_per_y_per_x.get(&i)
                    .unwrap()
                    .get(&j)
                    .unwrap()
                    .unwrap();
                line.push_str(character);
            }
            println!("{}", line);
        }
    }

    #[derive(Clone, std::fmt::Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    enum IceCreamShop {
        AppleCream,
        BananaBoost,
        CaramelJuice,
        DarkDestiny,
        EternalJoy,
        None,
    }

    impl HasProximity for IceCreamShop {
        fn get_proximity(&self, other: &Self) -> Proximity where Self: Sized {
            match self {
                Self::AppleCream => {
                    match other {
                        Self::AppleCream => Proximity::ExclusiveExistence,
                        Self::BananaBoost => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 4.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            }
                        ]},
                        Self::CaramelJuice => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 8.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::DarkDestiny => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 1.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::EternalJoy => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 4.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::None => Proximity::InAnotherDimensionEntirely,
                    }
                },
                Self::BananaBoost => {
                    match other {
                        Self::AppleCream => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 4.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::BananaBoost => Proximity::ExclusiveExistence,
                        Self::CaramelJuice => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 4.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::DarkDestiny => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 5.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::EternalJoy => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 8.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::None => Proximity::InAnotherDimensionEntirely,
                    }
                },
                Self::CaramelJuice => {
                    match other {
                        Self::AppleCream => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 8.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::BananaBoost => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 4.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::CaramelJuice => Proximity::ExclusiveExistence,
                        Self::DarkDestiny => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 7.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::EternalJoy => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 4.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::None => Proximity::InAnotherDimensionEntirely,
                    }
                },
                Self::DarkDestiny => {
                    match other {
                        Self::AppleCream => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 1.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::BananaBoost => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 5.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::CaramelJuice => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 7.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::DarkDestiny => Proximity::ExclusiveExistence,
                        Self::EternalJoy => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 3.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::None => Proximity::InAnotherDimensionEntirely,
                    }
                },
                Self::EternalJoy => {
                    match other {
                        Self::AppleCream => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 4.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::BananaBoost => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 8.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::CaramelJuice => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 4.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::DarkDestiny => Proximity::SomeDistanceAway { distances: vec![
                            Distance {
                                center: 3.0,
                                width: 0.0,
                                range_type: RangeType::Inclusive,
                            },
                        ]},
                        Self::EternalJoy => Proximity::ExclusiveExistence,
                        Self::None => Proximity::InAnotherDimensionEntirely,
                    }
                },
                Self::None => {
                    match other {
                        Self::AppleCream => Proximity::InAnotherDimensionEntirely,
                        Self::BananaBoost => Proximity::InAnotherDimensionEntirely,
                        Self::CaramelJuice => Proximity::InAnotherDimensionEntirely,
                        Self::DarkDestiny => Proximity::InAnotherDimensionEntirely,
                        Self::EternalJoy => Proximity::InAnotherDimensionEntirely,
                        Self::None => Proximity::InAnotherDimensionEntirely,
                    }
                },
            }
        }
    }

    #[test]
    fn test_w7b0_get_x_by_y_grid_proximity_graph() {
        let proximity_graph = get_x_by_y_grid_proximity_graph(2, 2);
        assert_eq!(4, proximity_graph.nodes.len());
        for index in 0..4 {
            assert_eq!(3, proximity_graph.nodes[index].distance_per_proximity_graph_node_id.keys().len());
        }
        println!("{:?}", proximity_graph.nodes);
    }

    #[test_case::test_case(5, 5, 0.0, 0.0)]
    #[test_case::test_case(4, 4, 1.0, 0.1)]
    #[test_case::test_case(3, 3, 2.0, 0.1)]
    fn test_h2s7_icecream_shops_in_grid(x: usize, y: usize, maximum_acceptable_distance_variance_factor: f32, acceptable_distance_variance_factor_difference: f32) {
        let proximity_graph = get_x_by_y_grid_proximity_graph(x, y);
        let values = get_values(x * y);
        let value_per_proximity_graph_node_id = proximity_graph.get_value_per_proximity_graph_node_id(values, maximum_acceptable_distance_variance_factor, acceptable_distance_variance_factor_difference, false).expect("Failed to get value per proximity graph node ID.");
        println_value_per_proximity_graph_node_id(x, y, &value_per_proximity_graph_node_id);
        println!("{:?}", value_per_proximity_graph_node_id);
        assert_eq!(IceCreamShop::AppleCream, *value_per_proximity_graph_node_id.get("node_0_0").unwrap());
        assert_eq!(IceCreamShop::BananaBoost, *value_per_proximity_graph_node_id.get(format!("node_{}_0", x - 1).as_str()).unwrap());
        assert_eq!(IceCreamShop::CaramelJuice, *value_per_proximity_graph_node_id.get(format!("node_{}_{}", x - 1, y - 1).as_str()).unwrap());
        assert_eq!(IceCreamShop::DarkDestiny, *value_per_proximity_graph_node_id.get("node_0_1").unwrap());
        assert_eq!(IceCreamShop::EternalJoy, *value_per_proximity_graph_node_id.get(format!("node_0_{}", y - 1).as_str()).unwrap());
    }

    #[test_case::test_case(6, 6, 0.0, 0.0)]
    fn test_y7c4_icecream_shops_in_grid(x: usize, y: usize, maximum_acceptable_distance_variance_factor: f32, acceptable_distance_variance_factor_difference: f32) {
        let proximity_graph = get_x_by_y_grid_proximity_graph(x, y);
        let values = get_values(x * y);
        let value_per_proximity_graph_node_id = proximity_graph.get_value_per_proximity_graph_node_id(values, maximum_acceptable_distance_variance_factor, acceptable_distance_variance_factor_difference, false).expect("Failed to get value per proximity graph node ID.");
        println_value_per_proximity_graph_node_id(x, y, &value_per_proximity_graph_node_id);
        println!("{:?}", value_per_proximity_graph_node_id);
        assert_eq!(IceCreamShop::AppleCream, *value_per_proximity_graph_node_id.get("node_0_0").unwrap());
        assert_eq!(IceCreamShop::BananaBoost, *value_per_proximity_graph_node_id.get("node_4_0").unwrap());
        assert_eq!(IceCreamShop::CaramelJuice, *value_per_proximity_graph_node_id.get("node_4_4").unwrap());
        assert_eq!(IceCreamShop::DarkDestiny, *value_per_proximity_graph_node_id.get("node_0_1").unwrap());
        assert_eq!(IceCreamShop::EternalJoy, *value_per_proximity_graph_node_id.get("node_0_4").unwrap());
    }

    #[test_case::test_case(6, 6, 1.0, 0.1)]
    fn test_t7c9_icecream_shops_in_grid(x: usize, y: usize, maximum_acceptable_distance_variance_factor: f32, acceptable_distance_variance_factor_difference: f32) {
        let proximity_graph = get_x_by_y_grid_proximity_graph(x, y);
        let values = get_values(x * y);
        let value_per_proximity_graph_node_id = proximity_graph.get_value_per_proximity_graph_node_id(values, maximum_acceptable_distance_variance_factor, acceptable_distance_variance_factor_difference, true).expect("Failed to get value per proximity graph node ID.");
        println_value_per_proximity_graph_node_id(x, y, &value_per_proximity_graph_node_id);
        println!("{:?}", value_per_proximity_graph_node_id);
        assert_eq!(IceCreamShop::AppleCream, *value_per_proximity_graph_node_id.get("node_0_0").unwrap());
        assert_eq!(IceCreamShop::BananaBoost, *value_per_proximity_graph_node_id.get("node_4_0").unwrap());
        assert_eq!(IceCreamShop::CaramelJuice, *value_per_proximity_graph_node_id.get("node_4_4").unwrap());
        assert_eq!(IceCreamShop::DarkDestiny, *value_per_proximity_graph_node_id.get("node_0_1").unwrap());
        assert_eq!(IceCreamShop::EternalJoy, *value_per_proximity_graph_node_id.get("node_0_4").unwrap());
    }

    #[test_case::test_case(4, 4, 0.5, 0.1)]
    #[test_case::test_case(3, 3, 1.0, 0.1)]
    fn test_o1n6_icecream_shops_in_grid(x: usize, y: usize, maximum_acceptable_distance_variance_factor: f32, acceptable_distance_variance_factor_difference: f32) {
        let proximity_graph = get_x_by_y_grid_proximity_graph(x, y);
        let values = get_values(x * y);
        let error = proximity_graph.get_value_per_proximity_graph_node_id(values, maximum_acceptable_distance_variance_factor, acceptable_distance_variance_factor_difference, false);
        assert!(error.is_err());
    }
}