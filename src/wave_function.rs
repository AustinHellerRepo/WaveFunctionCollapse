use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hash,
    io::BufReader,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use bitvec::prelude::*;
use log::debug;
extern crate pretty_env_logger;
mod indexed_view;
use crate::wave_function::collapsable_wave_function::collapsable_wave_function::CollapsableNode;

use self::{
    collapsable_wave_function::collapsable_wave_function::CollapsableWaveFunction,
    indexed_view::IndexedView,
};
mod probability_collection;
mod probability_tree;
mod probability_container;
pub mod collapsable_wave_function;
mod tests;

/// This struct makes for housing convenient utility functions.
pub struct NodeStateProbability;

impl NodeStateProbability {
    pub fn get_equal_probability<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord>(
        node_states: &Vec<TNodeState>,
    ) -> HashMap<TNodeState, f32> {
        let mut node_state_probability_per_node_state: HashMap<TNodeState, f32> = HashMap::new();

        for node_state in node_states {
            node_state_probability_per_node_state.insert(node_state.clone(), 1.0);
        }

        node_state_probability_per_node_state
    }
}

/// This is a node in the graph of the wave function. It can be in any of the provided node states, trying to achieve the cooresponding probability, connected to other nodes as described by the node state collections.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub id: String,
    pub node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>>,
    pub node_state_ids: Vec<TNodeState>,
    pub node_state_ratios: Vec<f32>,
}

impl<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> Node<TNodeState> {
    pub fn new(
        id: String,
        node_state_ratio_per_node_state_id: HashMap<TNodeState, f32>,
        node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>>,
    ) -> Self {
        let mut node_state_ids: Vec<TNodeState> = Vec::new();
        let mut node_state_ratios: Vec<f32> = Vec::new();
        for (node_state_id, node_state_ratio) in node_state_ratio_per_node_state_id.iter() {
            node_state_ids.push(node_state_id.clone());
            node_state_ratios.push(*node_state_ratio);
        }

        // sort the node_state_ids and node_state_probabilities
        let mut sort_permutation = permutation::sort(&node_state_ids);
        sort_permutation.apply_slice_in_place(&mut node_state_ids);
        sort_permutation.apply_slice_in_place(&mut node_state_ratios);

        Node {
            id,
            node_state_collection_ids_per_neighbor_node_id,
            node_state_ids,
            node_state_ratios,
        }
    }
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}

/// This struct represents a relationship between the state of one "original" node to another "neighbor" node, permitting only those node states for the connected neighbor if the original node is in the specific state. This defines the constraints between nodes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeStateCollection<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub id: String,
    pub node_state_id: TNodeState,
    pub node_state_ids: Vec<TNodeState>,
}

impl<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> NodeStateCollection<TNodeState> {
    pub fn new(id: String, node_state_id: TNodeState, node_state_ids: Vec<TNodeState>) -> Self {
        NodeStateCollection {
            id,
            node_state_id,
            node_state_ids,
        }
    }
}

/// This struct represents the uncollapsed definition of nodes and their relationships to other nodes.
#[derive(Serialize, Clone, Deserialize)]
pub struct WaveFunction<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    nodes: Vec<Node<TNodeState>>,
    node_state_collections: Vec<NodeStateCollection<TNodeState>>,
}

impl<
        TNodeState: Eq
            + Hash
            + Clone
            + std::fmt::Debug
            + Ord
            + Serialize
            + DeserializeOwned,
    > WaveFunction<TNodeState>
{
    pub fn new(
        nodes: Vec<Node<TNodeState>>,
        node_state_collections: Vec<NodeStateCollection<TNodeState>>,
    ) -> Self {
        WaveFunction {
            nodes,
            node_state_collections,
        }
    }

    pub fn get_nodes(&self) -> Vec<Node<TNodeState>> {
        self.nodes.clone()
    }

    pub fn get_node_state_collections(&self) -> Vec<NodeStateCollection<TNodeState>> {
        self.node_state_collections.clone()
    }

    pub fn validate(&self) -> Result<(), String> {
        let nodes_length: usize = self.nodes.len();

        let mut node_per_id: HashMap<&str, &Node<TNodeState>> = HashMap::new();
        let mut node_ids: HashSet<&str> = HashSet::new();
        self.nodes.iter().for_each(|node: &Node<TNodeState>| {
            node_per_id.insert(&node.id, node);
            node_ids.insert(&node.id);
        });

        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection<TNodeState>> =
            HashMap::new();
        self.node_state_collections
            .iter()
            .for_each(|node_state_collection| {
                node_state_collection_per_id.insert(&node_state_collection.id, node_state_collection);
            });

        // ensure that references neighbors are actually nodes
        for (_, node) in node_per_id.iter() {
            for (neighbor_node_id_string, _) in
                node.node_state_collection_ids_per_neighbor_node_id.iter()
            {
                let neighbor_node_id: &str = neighbor_node_id_string;
                if !node_ids.contains(neighbor_node_id) {
                    return Err(format!("Neighbor node {neighbor_node_id} does not exist in main list of nodes."));
                }
            }
        }

        let mut at_least_one_node_connects_to_all_other_nodes: bool = false;
        for node in self.nodes.iter() {
            // ensure that all nodes connect to all other nodes
            let mut all_traversed_node_ids: HashSet<&str> = HashSet::new();
            let mut potential_node_ids: Vec<&str> = Vec::new();

            potential_node_ids.push(&node.id);

            while let Some(node_id) = potential_node_ids.pop() {
                let node = node_per_id.get(node_id).unwrap();
                for neighbor_node_id_string in
                    node.node_state_collection_ids_per_neighbor_node_id.keys()
                {
                    let neighbor_node_id: &str = neighbor_node_id_string;
                    if !all_traversed_node_ids.contains(neighbor_node_id)
                        && !potential_node_ids.contains(&neighbor_node_id)
                    {
                        potential_node_ids.push(neighbor_node_id);
                    }
                }
                all_traversed_node_ids.insert(node_id);
            }

            let all_traversed_node_ids_length = all_traversed_node_ids.len();
            if all_traversed_node_ids_length == nodes_length {
                at_least_one_node_connects_to_all_other_nodes = true;
                break;
            }
        }

        if !at_least_one_node_connects_to_all_other_nodes {
            return Err(String::from(
                "Not all nodes connect together. At least one node must be able to traverse to all other nodes.",
            ));
        }

        Ok(())
    }

    pub fn get_collapsable_wave_function<
        'a,
        TCollapsableWaveFunction: CollapsableWaveFunction<'a, TNodeState>,
    >(
        &'a self,
        random_seed: Option<u64>,
    ) -> TCollapsableWaveFunction {
        // Build index map: node id -> node index
        let node_index_per_id: HashMap<&str, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id.as_str(), i))
            .collect();

        let mut node_per_id: HashMap<&str, &Node<TNodeState>> = HashMap::new();
        self.nodes
            .iter()
            .for_each(|node: &Node<TNodeState>| {
                node_per_id.insert(&node.id, node);
            });

        let mut node_state_collection_per_id: HashMap<&str, &NodeStateCollection<TNodeState>> =
            HashMap::new();
        self.node_state_collections
            .iter()
            .for_each(|node_state_collection| {
                node_state_collection_per_id.insert(&node_state_collection.id, node_state_collection);
            });

        // Convert neighbor id-based masks to index-based masks
        // neighbor_mask_mapped_view_per_node_index: node_index -> (node_state -> neighbor_index -> BitVec)
        let mut neighbor_mask_mapped_view_per_node_index: HashMap<
            usize,
            HashMap<&TNodeState, HashMap<usize, BitVec>>,
        > = HashMap::new();

        // First, build parent->child masks
        // mask_per_parent_state_per_parent_neighbor_per_node_index: child_node_index -> parent_node_index -> parent_state -> BitVec
        let mut mask_per_parent_state_per_parent_neighbor_per_node_index: HashMap<
            usize,
            HashMap<usize, HashMap<&TNodeState, BitVec>>,
        > = HashMap::new();

        for (child_node_index, child_node) in self.nodes.iter().enumerate() {
            let mut mask_per_parent_state_per_parent_neighbor: HashMap<
                usize,
                HashMap<&TNodeState, BitVec>,
            > = HashMap::new();

            for parent_neighbor_node in self.nodes.iter() {
                if parent_neighbor_node
                    .node_state_collection_ids_per_neighbor_node_id
                    .contains_key(&child_node.id)
                {
                    let parent_node_index = node_index_per_id[parent_neighbor_node.id.as_str()];

                    debug!(
                        "constructing mask for {:?}'s child node {:?}.",
                        parent_neighbor_node.id, child_node.id
                    );

                    let mut mask_per_parent_state: HashMap<&TNodeState, BitVec> = HashMap::new();

                    let node_state_collection_ids: &Vec<String> = parent_neighbor_node
                        .node_state_collection_ids_per_neighbor_node_id
                        .get(&child_node.id)
                        .unwrap();

                    for node_state_collection_id in node_state_collection_ids.iter() {
                        let node_state_collection = node_state_collection_per_id
                            .get(node_state_collection_id.as_str())
                            .unwrap();

                        let mut mask: BitVec = BitVec::new();
                        for node_state_id in child_node.node_state_ids.iter() {
                            mask.push(node_state_collection.node_state_ids.contains(node_state_id));
                        }
                        mask_per_parent_state.insert(&node_state_collection.node_state_id, mask);
                    }

                    mask_per_parent_state_per_parent_neighbor
                        .insert(parent_node_index, mask_per_parent_state);
                }
            }

            mask_per_parent_state_per_parent_neighbor_per_node_index
                .insert(child_node_index, mask_per_parent_state_per_parent_neighbor);
        }

        // Fill neighbor_mask_mapped_view_per_node_index
        for (node_index, node) in self.nodes.iter().enumerate() {
            let mut mask_per_neighbor_per_state: HashMap<&TNodeState, HashMap<usize, BitVec>> =
                HashMap::new();

            for (neighbor_node_id, _) in
                node.node_state_collection_ids_per_neighbor_node_id.iter()
            {
                let neighbor_node_index = node_index_per_id[neighbor_node_id.as_str()];

                let mask_per_parent_state_per_parent_neighbor = mask_per_parent_state_per_parent_neighbor_per_node_index
                    .get(&neighbor_node_index)
                    .unwrap();
                let mask_per_parent_state = mask_per_parent_state_per_parent_neighbor
                    .get(&node_index)
                    .unwrap();

                for (node_state_id, mask) in mask_per_parent_state.iter() {
                    mask_per_neighbor_per_state
                        .entry(node_state_id)
                        .or_insert(HashMap::new())
                        .insert(neighbor_node_index, mask.clone());
                }
            }

            neighbor_mask_mapped_view_per_node_index
                .insert(node_index, mask_per_neighbor_per_state);
        }

        let mut node_state_indexed_view_per_node_index: HashMap<usize, IndexedView<&TNodeState>> =
            HashMap::new();

        for (node_index, node) in self.nodes.iter().enumerate() {
            let referenced_node_state_ids: Vec<&TNodeState> = node.node_state_ids.iter().collect();
            let cloned_node_state_ratios: Vec<f32> = node.node_state_ratios.clone();

            let node_state_indexed_view =
                IndexedView::new(referenced_node_state_ids, cloned_node_state_ratios);
            node_state_indexed_view_per_node_index
                .insert(node_index, node_state_indexed_view);
        }

        let mut random_instance = if let Some(seed) = random_seed {
            fastrand::Rng::with_seed(seed)
        } else {
            fastrand::Rng::new()
        };

        let mut collapsable_nodes: Vec<CollapsableNode<TNodeState>> = Vec::with_capacity(
            self.nodes.len(),
        );

        for (node_index, node) in self.nodes.iter().enumerate() {
            let node_state_indexed_view: IndexedView<&TNodeState> = node_state_indexed_view_per_node_index
                .remove(&node_index)
                .unwrap();
            let mask_per_neighbor_per_state = neighbor_mask_mapped_view_per_node_index
                .remove(&node_index)
                .unwrap();

            let mut collapsable_node = CollapsableNode::new(
                node.id.as_str(),
                &node.node_state_collection_ids_per_neighbor_node_id,
                mask_per_neighbor_per_state,
                node_state_indexed_view,
            );

            // Fill in actual neighbor indices
            collapsable_node.neighbor_node_indices.clear();
            for neighbor_id in node.node_state_collection_ids_per_neighbor_node_id.keys() {
                collapsable_node
                    .neighbor_node_indices
                    .push(node_index_per_id[neighbor_id.as_str()]);
            }
            collapsable_node.neighbor_node_indices.sort();

            if random_seed.is_some() {
                collapsable_node.randomize(&mut random_instance);
            }

            collapsable_nodes.push(collapsable_node);
        }

        // Fill in parent neighbor indices
        for (node_index, collapsable_node) in collapsable_nodes.iter_mut().enumerate() {
            if mask_per_parent_state_per_parent_neighbor_per_node_index
                .contains_key(&node_index)
            {
                let mask_per_parent_state_per_parent_neighbor =
                    mask_per_parent_state_per_parent_neighbor_per_node_index
                        .get(&node_index)
                        .unwrap();
                for parent_neighbor_node_index in mask_per_parent_state_per_parent_neighbor.keys() {
                    collapsable_node
                        .parent_neighbor_node_indices
                        .push(*parent_neighbor_node_index);
                }
                if random_seed.is_some() {
                    random_instance.shuffle(&mut collapsable_node.parent_neighbor_node_indices);
                } else {
                    collapsable_node.parent_neighbor_node_indices.sort();
                }
            }
        }

        TCollapsableWaveFunction::new(
            collapsable_nodes,
            random_instance,
        )
    }

    pub fn save_to_file(&self, file_path: &str) {
        let serialized_self = serde_json::to_string(self).unwrap();
        std::fs::write(file_path, serialized_self).unwrap();
    }

    pub fn load_from_file(file_path: &str) -> Self {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let deserialized_self: WaveFunction<TNodeState> = serde_json::from_reader(reader).unwrap();
        deserialized_self
    }
}
