// this abstraction is a web of nodes that have a center and specific states are expected to be closer to each other than further away
// you can imagine a game needing points of interest that are nearby each other - you would not want quest-adjacent locations to be physically distant

use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};
use std::hash::Hash;
use serde::{Deserialize, Serialize};
use shiftnanigans::incrementer::fixed_binary_density_incrementer::FixedBinaryDensityIncrementer;
use shiftnanigans::incrementer::Incrementer;
use crate::wave_function::collapsable_wave_function::collapsable_wave_function::{CollapsableNode, CollapsableWaveFunction, CollapsedWaveFunction};
use crate::wave_function::collapsable_wave_function::sequential_collapsable_wave_function::SequentialCollapsableWaveFunction;
use crate::wave_function::{Node, NodeStateCollection, NodeStateProbability, WaveFunction};

pub enum Proximity {
    NeverMoreThanOne,
    // the values are different from each other in a quantifiable way
    SomeDistanceAway {
        distance: f32,
    },
    // the values are not related at all and are unquantifiably different
    InAnotherDimensionEntirely,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize)]
enum NodeState<TValue>
where
    TValue: PriorityWebNodeValue,
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
    TValue: PriorityWebNodeValue, // Ensure TValue implements Deserialize
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
            TValue: PriorityWebNodeValue,
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

trait PriorityWebNodeValue: Eq + Hash + Clone + std::fmt::Debug + Ord + Serialize + for<'de> Deserialize<'de> {
    fn get_proximity(&self, other: &Self) -> Proximity where Self: Sized;
}

pub struct PriorityWebNode {
    priority_web_node_id: String,
    distance_per_node_index: Vec<f32>,
}

pub enum PriorityWebError {
    FailedToMapValuesToNodesAtAnyDistance,
}

pub struct PriorityWeb<TValue: PriorityWebNodeValue> {
    values: Vec<TValue>,
    nodes: Vec<PriorityWebNode>,
}

impl<TValue: PriorityWebNodeValue> PriorityWeb<TValue> {
    pub fn new(values: Vec<TValue>, nodes: Vec<PriorityWebNode>) -> Self {
        Self {
            values,
            nodes,
        }
    }
    pub fn get_value_per_priority_web_node_id(&self, random_instance: Rc<RefCell<fastrand::Rng>>) -> Result<HashMap<String, TValue>, PriorityWebError> {
        let maximum_value_proximity = {
            let mut maximum_value_proximity = 0.0;
            for i in 0..(self.values.len() - 1) {
                for j in (i + 1)..self.values.len() {
                    maximum_value_proximity += match self.values[i].get_proximity(&self.values[j]) {
                        Proximity::NeverMoreThanOne => 0.0,
                        Proximity::SomeDistanceAway { distance } => distance,
                        Proximity::InAnotherDimensionEntirely => 0.0,
                    };
                }
            }
            maximum_value_proximity
        };

        let maximum_node_distance = {
            let mut maximum_node_distance = 0.0;
            for node in self.nodes.iter() {
                for distance in node.distance_per_node_index.iter() {
                    let distance = *distance;
                    if distance > maximum_node_distance {
                        maximum_node_distance = distance;
                    }
                }
            }
            maximum_node_distance
        };

        // iterate over the construction and collapsing of the wave function until the best solution is found
        // first start with the maximum distance being acceptable to ensure that the values can collapse at all
        // if they can collapse, then begin to binary-search for the optimal configuration by restricting what is an acceptable maximum proximity
        //      ex: divide in half first, too low? then make it 75% of original maximum, still too low? make it between 75%-100%, etc.

        let distance_variance_factor = 0.0;
        let mut best_collapsed_wave_function = None;
        while best_collapsed_wave_function.is_none() {
            let primary_node_state_ratio_per_node_state_id = {
                let node_state_ids = self.values.iter()
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
                for (priority_web_node_index, priority_web_node) in self.nodes.iter().enumerate() {
                    // setup the NodeStateCollections per neighbor
                    let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                    for (neighbor_distance_index, neighbor_distance) in priority_web_node.distance_per_node_index.iter().enumerate() {
                        let neighbor_distance = *neighbor_distance;

                        let mut node_state_collection_ids: Vec<String> = Vec::new();
                        if neighbor_distance_index != neighbor_distance_index {
                            // collect up each node state
                            for (current_value_index, current_value) in self.values.iter().enumerate() {
                                let current_node_state = NodeState::Primary {
                                    state: current_value.clone(),
                                };
                                let mut other_node_states = Vec::new();
                                for (other_value_index, other_value) in self.values.iter().enumerate() {
                                    match current_value.get_proximity(other_value) {
                                        Proximity::NeverMoreThanOne => {
                                            // do not add this other value as a possible value if they are the same index
                                            if current_value_index != other_value_index {
                                                let other_node_state = NodeState::Primary {
                                                    state: other_value.clone(),
                                                };
                                                other_node_states.push(other_node_state);
                                            }
                                        }
                                        Proximity::SomeDistanceAway { distance } => {
                                            let distance_variance = distance * distance_variance_factor;
                                            let from_distance = distance - distance_variance;
                                            let to_distance = distance + distance_variance;

                                            if from_distance <= neighbor_distance && neighbor_distance <= to_distance {
                                                // this neighbor is within range of being in this other state
                                                let other_node_state = NodeState::Primary {
                                                    state: other_value.clone(),
                                                };
                                                other_node_states.push(other_node_state);
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
                                let node_state_collection_id: String = format!("primary_{}_{}_{}", priority_web_node_index, neighbor_distance_index, current_value_index);

                                let node_state_collection = NodeStateCollection::new(
                                    node_state_collection_id.clone(),
                                    current_node_state,
                                    other_node_states,
                                );
                                node_state_collections.push(node_state_collection);

                                node_state_collection_ids.push(node_state_collection_id);
                            }
                        }

                        let neighbor_node_id = format!("primary_{}", neighbor_distance_index);
                        node_state_collection_ids_per_neighbor_node_id.insert(neighbor_node_id, node_state_collection_ids);
                    }

                    let node = Node::new(
                        format!("primary_{}", priority_web_node.priority_web_node_id),
                        primary_node_state_ratio_per_node_state_id.clone(),
                        node_state_collection_ids_per_neighbor_node_id,
                    );
                    nodes.push(node);
                }
                (nodes, node_state_collections)
            };
            {
                let wave_function = WaveFunction::new(nodes, node_state_collections);
                let mut collapsable_wave_function = wave_function.get_collapsable_wave_function::<SequentialCollapsableWaveFunction<NodeState<TValue>>>(None);
                match collapsable_wave_function.collapse() {
                    Ok(collapsed_wave_function) => {
                        // TODO store this as the best collapsed wave function
                        // TODO expand or retract the distance variance
                        todo!();
                    },
                    Err(_) => {
                        // TODO expand or retract the distance variance
                        // TODO if the distance variance is beyond some measure of the maximum value proximity versus the maximum node distance, return Err
                        todo!();
                    },
                }
            }
        }
        
        let best_collapsed_wave_function: CollapsedWaveFunction<NodeState<TValue>> = best_collapsed_wave_function.unwrap();
        let mut value_per_priority_web_node_id = HashMap::new();
        for (node_id, node_state) in best_collapsed_wave_function.node_state_per_node_id {
            match node_state {
                NodeState::Primary { state } => {
                    if let Some(priority_web_node_id) = node_id.strip_prefix("primary_") {
                        value_per_priority_web_node_id.insert(String::from(priority_web_node_id), state);
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
        Ok(value_per_priority_web_node_id)
    }
}