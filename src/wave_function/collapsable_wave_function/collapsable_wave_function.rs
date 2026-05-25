use std::fmt::Display;
use std::collections::HashMap;
use bitvec::vec::BitVec;
use fastrand::Rng;
use serde::{Serialize, Deserialize};
use std::hash::Hash;
use crate::wave_function::indexed_view::IndexedView;

/// This trait defines the relationship between collapsable nodes and a collapsed state.
pub trait CollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    fn new(
        collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
        random_instance: fastrand::Rng,
    ) -> Self
    where
        Self: Sized;
    fn collapse_into_steps(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String>;
    fn collapse(&mut self) -> Result<CollapsedWaveFunction<TNodeState>, String>;
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct CollapsedNodeState<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub node_id: String,
    pub node_state_id: Option<TNodeState>,
}

#[derive(Serialize)]
pub struct CollapsedWaveFunction<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub node_state_per_node_id: HashMap<String, TNodeState>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UncollapsedWaveFunction<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub node_state_per_node: HashMap<String, Option<TNodeState>>,
}

impl<TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> Hash for UncollapsedWaveFunction<TNodeState> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for property in self.node_state_per_node.iter() {
            property.hash(state);
        }
    }
}

/// This struct represents a stateful node in a collapsable wave function which references a base node from the wave function.
#[derive(Debug)]
pub struct CollapsableNode<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    // the node id that this collapsable node refers to
    pub id: &'a str,
    // this nodes list of neighbor node indices
    pub neighbor_node_indices: Vec<usize>,
    // the full list of possible node states, masked by internal references to neighbor masks
    pub node_state_indexed_view: IndexedView<&'a TNodeState>,
    // the mapped view that this node's neighbors will have a reference to and pull their masks from
    // keyed by neighbor node index
    pub mask_per_neighbor_per_state: HashMap<&'a TNodeState, HashMap<usize, BitVec>>,
    // the index of traversed nodes based on the sorted vector of nodes as they are chosen for state determination
    pub current_chosen_from_sort_index: Option<usize>,
    // the neighbors that are pointing to this collapsable node (stored as indices)
    pub parent_neighbor_node_indices: Vec<usize>,
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableNode<'a, TNodeState> {
    pub fn new(
        id: &'a str,
        node_state_collection_ids_per_neighbor_node_id: &'a HashMap<String, Vec<String>>,
        mask_per_neighbor_per_state: HashMap<&'a TNodeState, HashMap<usize, BitVec>>,
        node_state_indexed_view: IndexedView<&'a TNodeState>,
    ) -> Self {
        // get the neighbors for this node
        let mut neighbor_node_indices: Vec<usize> = Vec::new();

        for _neighbor_node_id_string in node_state_collection_ids_per_neighbor_node_id.keys() {
            // indices are filled in after construction via a separate pass
            neighbor_node_indices.push(0);
        }
        neighbor_node_indices.sort();

        CollapsableNode {
            id,
            neighbor_node_indices,
            node_state_indexed_view,
            mask_per_neighbor_per_state,
            current_chosen_from_sort_index: None,
            parent_neighbor_node_indices: Vec::new(),
        }
    }
    pub fn randomize(&mut self, random_instance: &mut Rng) {
        self.node_state_indexed_view.shuffle(random_instance);
    }
    pub fn is_fully_restricted(&mut self) -> bool {
        self.node_state_indexed_view
            .is_fully_restricted()
            || self.node_state_indexed_view.is_current_state_restricted()
    }
    pub fn add_mask(&mut self, mask: &BitVec) {
        self.node_state_indexed_view.add_mask(mask);
    }
    pub fn subtract_mask(&mut self, mask: &BitVec) {
        self.node_state_indexed_view.subtract_mask(mask);
    }
    pub fn forward_mask(&mut self, mask: &BitVec) {
        self.node_state_indexed_view.forward_mask(mask);
    }
    pub fn reverse_mask(&mut self) {
        self.node_state_indexed_view.reverse_mask();
    }
    pub fn is_mask_restrictive_to_current_state(&self, mask: &BitVec) -> bool {
        let is_restrictive = self
            .node_state_indexed_view
            .is_mask_restrictive_to_current_state(mask);
        if is_restrictive {
            debug!("mask is restrictive");
        } else {
            debug!("mask is not restrictive");
        }
        is_restrictive
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> Display for CollapsableNode<'a, TNodeState> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
