use std::fmt::Display;
use std::{collections::HashMap, marker::PhantomData};
use std::rc::Rc;
use std::cell::RefCell;
use bitvec::vec::BitVec;
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::hash::Hash;
use crate::wave_function::indexed_view::IndexedView;

/// This trait defines the relationship between collapsable nodes and a collapsed state.
pub trait CollapsableWaveFunction<'a, TIdentifier: Eq + Hash + Clone + std::fmt::Debug + Ord, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    fn new(collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TIdentifier, TNodeState>>>>, collapsable_node_per_id: HashMap<&'a TIdentifier, Rc<RefCell<CollapsableNode<'a, TIdentifier, TNodeState>>>>) -> Self where Self: Sized;
    fn collapse_into_steps(&'a mut self) -> Result<Vec<CollapsedNodeState<TIdentifier, TNodeState>>, String>;
    fn collapse(&'a mut self) -> Result<CollapsedWaveFunction<TIdentifier, TNodeState>, String>;
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct CollapsedNodeState<TIdentifier: Eq + Hash + Clone + std::fmt::Debug + Ord, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub node_id: TIdentifier,
    pub node_state_id: Option<TNodeState>
}

#[derive(Serialize)]
pub struct CollapsedWaveFunction<TIdentifier: Eq + Hash + Clone + std::fmt::Debug + Ord, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub node_state_per_node: HashMap<TIdentifier, TNodeState>
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UncollapsedWaveFunction<TIdentifier: Eq + Hash + Clone + std::fmt::Debug + Ord, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    pub node_state_per_node: HashMap<TIdentifier, Option<TNodeState>>
}

impl<TIdentifier: Eq + Hash + Clone + std::fmt::Debug + Ord, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> Hash for UncollapsedWaveFunction<TIdentifier, TNodeState> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for property in self.node_state_per_node.iter() {
            property.hash(state);
        }
    }
}

/// This struct represents a stateful node in a collapsable wave function which references a base node from the wave function.
#[derive(Debug)]
pub struct CollapsableNode<'a, TIdentifier: Eq + Hash + Clone + std::fmt::Debug + Ord, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    // the node id that this collapsable node refers to
    pub id: &'a TIdentifier,
    // this nodes list of neighbor node ids
    pub neighbor_node_ids: Vec<&'a TIdentifier>,
    // the full list of possible node states, masked by internal references to neighbor masks
    pub node_state_indexed_view: IndexedView<&'a TNodeState>,
    // the mapped view that this node's neighbors will have a reference to and pull their masks from
    pub mask_per_neighbor_per_state: HashMap<&'a TNodeState, HashMap<&'a TIdentifier, BitVec>>,
    // the index of traversed nodes based on the sorted vector of nodes as they are chosen for state determination
    pub current_chosen_from_sort_index: Option<usize>,
    // the neighbors that are pointing to this collapsable node
    pub parent_neighbor_node_ids: Vec<&'a TIdentifier>,
    // allowing for Node<TNodeState> to be an argument of CollapsableNode functions
    node_state_type: PhantomData<TNodeState>
}

impl<'a, TIdentifier: Eq + Hash + Clone + std::fmt::Debug + Ord, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableNode<'a, TIdentifier, TNodeState> {
    pub fn new(id: &'a TIdentifier, node_state_collection_ids_per_neighbor_node_id: &'a HashMap<TIdentifier, Vec<TIdentifier>>, mask_per_neighbor_per_state: HashMap<&'a TNodeState, HashMap<&'a TIdentifier, BitVec>>, node_state_indexed_view: IndexedView<&'a TNodeState>) -> Self {
        // get the neighbors for this node
        let mut neighbor_node_ids: Vec<&TIdentifier> = Vec::new();

        for neighbor_node_id_string in node_state_collection_ids_per_neighbor_node_id.keys() {
            let neighbor_node_id: &TIdentifier = neighbor_node_id_string;
            neighbor_node_ids.push(neighbor_node_id);
        }
        neighbor_node_ids.sort();

        CollapsableNode {
            id: id,
            neighbor_node_ids: neighbor_node_ids,
            node_state_indexed_view: node_state_indexed_view,
            mask_per_neighbor_per_state: mask_per_neighbor_per_state,
            current_chosen_from_sort_index: None,
            parent_neighbor_node_ids: Vec::new(),
            node_state_type: PhantomData
        }
    }
    pub fn randomize<R: Rng + ?Sized>(&mut self, random_instance: &mut R) {
        self.node_state_indexed_view.shuffle(random_instance);
    }
    pub fn is_fully_restricted(&mut self) -> bool {
        self.node_state_indexed_view.is_fully_restricted() || self.node_state_indexed_view.is_current_state_restricted()
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
        let is_restrictive = self.node_state_indexed_view.is_mask_restrictive_to_current_state(mask);
        if is_restrictive {
            debug!("mask is restrictive");
        }
        else {
            debug!("mask is not restrictive");
        }
        is_restrictive
    }
}

impl<'a, TIdentifier: Eq + Hash + Clone + std::fmt::Debug + Ord, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> Display for CollapsableNode<'a, TIdentifier, TNodeState> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}