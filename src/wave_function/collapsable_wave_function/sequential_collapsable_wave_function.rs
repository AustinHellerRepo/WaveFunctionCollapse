use std::collections::HashMap;
use std::hash::Hash;
use bitvec::vec::BitVec;
use super::collapsable_wave_function::{CollapsableWaveFunction, CollapsableNode, CollapsedNodeState, CollapsedWaveFunction};

pub struct SequentialCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
    collapsable_nodes_length: usize,
    current_collapsable_node_index: usize,
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> SequentialCollapsableWaveFunction<'a, TNodeState> {
    fn try_increment_current_collapsable_node_state(&mut self) -> CollapsedNodeState<TNodeState> {
        let current = &mut self.collapsable_nodes[self.current_collapsable_node_index];
        let is_successful = current.node_state_indexed_view.try_move_next();
        if is_successful {
            current.current_chosen_from_sort_index = Some(self.current_collapsable_node_index);
            CollapsedNodeState {
                node_id: String::from(current.id),
                node_state_id: Some((*current.node_state_indexed_view.get().unwrap()).clone()),
            }
        } else {
            current.current_chosen_from_sort_index = None;
            CollapsedNodeState {
                node_id: String::from(current.id),
                node_state_id: None,
            }
        }
    }

    fn try_alter_reference_to_current_collapsable_node_mask(&mut self) -> bool {
        // Clone all needed masks before doing any mutation
        let masks_to_apply: Vec<(usize, BitVec)> = {
            let current = &self.collapsable_nodes[self.current_collapsable_node_index];
            let mut m = Vec::new();
            if let Some(state_index) = current.node_state_indexed_view.get_index() {
                if state_index < current.masks_by_state_index.len() {
                    let mask_per_neighbor = &current.masks_by_state_index[state_index];
                    for &neighbor_index in &current.neighbor_node_indices {
                        if let Some(mask) = mask_per_neighbor.get(&neighbor_index) {
                            m.push((neighbor_index, mask.clone()));
                        }
                    }
                }
            }
            m
        };

        let mut is_successful = true;
        let mut traversed = Vec::new();
        for (neighbor_index, mask) in &masks_to_apply {
            self.collapsable_nodes[*neighbor_index].forward_mask(mask);
            debug!("adding mask to {:?} when in try_alter_reference_to_current_collapsable_node_mask", neighbor_index);
            traversed.push(*neighbor_index);
            if self.collapsable_nodes[*neighbor_index].is_fully_restricted() {
                is_successful = false;
                break;
            }
        }

        if !is_successful {
            for &neighbor_index in &traversed {
                debug!("reversing mask for {:?} when in try_alter_reference_to_current_collapsable_node_mask", neighbor_index);
                self.collapsable_nodes[neighbor_index].reverse_mask();
            }
        }

        is_successful
    }

    fn move_to_next_collapsable_node(&mut self) {
        let id = self.collapsable_nodes[self.current_collapsable_node_index].id;
        let idx = self.current_collapsable_node_index;
        debug!("moving from {id} at index {idx}");
        self.current_collapsable_node_index += 1;
        if cfg!(debug_assertions) {
            if self.current_collapsable_node_index == self.collapsable_nodes_length {
                debug!("moved outside of bounds at index {}", self.current_collapsable_node_index);
            } else {
                debug!("moved to {} at index {}", self.collapsable_nodes[self.current_collapsable_node_index].id, self.current_collapsable_node_index);
            }
        }
    }

    fn is_fully_collapsed(&self) -> bool {
        self.current_collapsable_node_index == self.collapsable_nodes_length
    }

    fn try_move_to_previous_collapsable_node_neighbor(&mut self) {
        {
            let current = &mut self.collapsable_nodes[self.current_collapsable_node_index];
            current.node_state_indexed_view.reset();
            current.current_chosen_from_sort_index = None;
        }

        if self.current_collapsable_node_index != 0 {
            self.current_collapsable_node_index -= 1;
            // Collect neighbor indices to reverse (no data needed, just indices since reverse_mask uses internal state)
            let indices_to_reverse: Vec<usize> = {
                let current = &self.collapsable_nodes[self.current_collapsable_node_index];
                let mut idxs = Vec::new();
                if let Some(state_index) = current.node_state_indexed_view.get_index() {
                    if state_index < current.masks_by_state_index.len() {
                        let mask_per_neighbor = &current.masks_by_state_index[state_index];
                        for &neighbor_index in &current.neighbor_node_indices {
                            if mask_per_neighbor.contains_key(&neighbor_index) {
                                idxs.push(neighbor_index);
                            }
                        }
                    }
                }
                idxs
            };
            for neighbor_index in indices_to_reverse {
                debug!("reversing mask for {:?} when in try_move_to_previous_collapsable_node_neighbor", neighbor_index);
                self.collapsable_nodes[neighbor_index].reverse_mask();
            }
        }
    }

    fn is_fully_reset(&self) -> bool {
        if self.current_collapsable_node_index != 0 {
            return false;
        }
        self.collapsable_nodes[self.current_collapsable_node_index].current_chosen_from_sort_index.is_none()
    }

    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut node_state_per_node_id: HashMap<String, TNodeState> = HashMap::new();
        for node in &self.collapsable_nodes {
            let state: TNodeState = (*node.node_state_indexed_view.get().unwrap()).clone();
            node_state_per_node_id.insert(String::from(node.id), state);
        }
        CollapsedWaveFunction { node_state_per_node_id }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for SequentialCollapsableWaveFunction<'a, TNodeState> {
    fn new(
        collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
        _random_instance: fastrand::Rng,
    ) -> Self {
        SequentialCollapsableWaveFunction {
            collapsable_nodes_length: collapsable_nodes.len(),
            current_collapsable_node_index: 0,
            collapsable_nodes,
        }
    }

    fn collapse_into_steps(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {
        let mut collapsed_node_states = Vec::new();
        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !self.is_fully_collapsed() {
            let collapsed_node_state = self.try_increment_current_collapsable_node_state();
            let is_successful = collapsed_node_state.node_state_id.is_some();
            collapsed_node_states.push(collapsed_node_state);
            if is_successful {
                if self.try_alter_reference_to_current_collapsable_node_mask() {
                    self.move_to_next_collapsable_node();
                }
            } else {
                self.try_move_to_previous_collapsable_node_neighbor();
                if self.is_fully_reset() {
                    is_unable_to_collapse = true;
                }
            }
        }
        debug!("finished while loop");
        Ok(collapsed_node_states)
    }

    fn collapse(&mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {
        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !self.is_fully_collapsed() {
            let is_successful = self.try_increment_current_collapsable_node_state().node_state_id.is_some();
            if is_successful {
                if self.try_alter_reference_to_current_collapsable_node_mask() {
                    self.move_to_next_collapsable_node();
                }
            } else {
                self.try_move_to_previous_collapsable_node_neighbor();
                if self.is_fully_reset() {
                    is_unable_to_collapse = true;
                }
            }
        }
        debug!("finished while loop");
        if is_unable_to_collapse {
            Err(String::from("Cannot collapse wave function."))
        } else {
            Ok(self.get_collapsed_wave_function())
        }
    }
}
