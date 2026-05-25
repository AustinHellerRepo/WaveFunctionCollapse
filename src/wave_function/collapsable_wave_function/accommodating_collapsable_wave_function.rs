use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use bitvec::vec::BitVec;
use super::collapsable_wave_function::{CollapsableWaveFunction, CollapsableNode, CollapsedNodeState, CollapsedWaveFunction};

pub struct AccommodatingCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
    accommodate_node_indices: Vec<usize>,
    accommodate_node_indices_length: usize,
    accommodate_node_indices_index: usize,
    accommodated_total: usize,
    impacted_node_indices: HashSet<usize>,
    random_instance: fastrand::Rng,
}

fn collect_masks_from_node<T: Eq + Hash + Clone + std::fmt::Debug + Ord>(
    nodes: &[CollapsableNode<'_, T>],
    node_index: usize,
) -> Vec<(usize, BitVec)> {
    let node = &nodes[node_index];
    let mut m = Vec::new();
    if let Some(state) = node.node_state_indexed_view.get() {
        if let Some(mask_map) = node.mask_per_neighbor_per_state.get(state) {
            for &neighbor_index in &node.neighbor_node_indices {
                if let Some(mask) = mask_map.get(&neighbor_index) {
                    m.push((neighbor_index, mask.clone()));
                }
            }
        }
    }
    m
}

fn collect_masks_for_state<T: Eq + Hash + Clone + std::fmt::Debug + Ord>(
    nodes: &[CollapsableNode<'_, T>],
    node_index: usize,
    state: &T,
) -> Vec<(usize, BitVec)> {
    let node = &nodes[node_index];
    let mut m = Vec::new();
    if let Some(mask_map) = node.mask_per_neighbor_per_state.get(state) {
        for &neighbor_index in &node.neighbor_node_indices {
            if let Some(mask) = mask_map.get(&neighbor_index) {
                m.push((neighbor_index, mask.clone()));
            }
        }
    }
    m
}

/// Check if the parent at parent_index, when in the given state, would restrict the current node
/// Returns the mask if one exists, None otherwise
fn get_restricting_mask_from_parent_to_current<T: Eq + Hash + Clone + std::fmt::Debug + Ord>(
    nodes: &[CollapsableNode<'_, T>],
    parent_index: usize,
    current_index: usize,
    parent_state: &T,
) -> Option<BitVec> {
    let parent = &nodes[parent_index];
    if let Some(mask_map) = parent.mask_per_neighbor_per_state.get(parent_state) {
        mask_map.get(&current_index).cloned()
    } else {
        None
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> AccommodatingCollapsableWaveFunction<'a, TNodeState> {
    fn initialize_nodes(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {
        let mut initial = Vec::new();
        for (idx, node) in self.collapsable_nodes.iter_mut().enumerate() {
            if !node.node_state_indexed_view.try_move_next() {
                return Err(String::from("Cannot collapse wave function."));
            }
            self.accommodate_node_indices.push(idx);
            let state = node.node_state_indexed_view.get().unwrap();
            initial.push(CollapsedNodeState {
                node_id: String::from(node.id),
                node_state_id: Some((*state).clone()),
            });
        }
        self.accommodate_node_indices_length = self.accommodate_node_indices.len();
        self.accommodated_total = self.accommodate_node_indices_length;

        let mut all_masks = Vec::new();
        for idx in 0..self.collapsable_nodes.len() {
            all_masks.extend(collect_masks_from_node(&self.collapsable_nodes, idx));
        }
        for (target, mask) in all_masks {
            self.collapsable_nodes[target].add_mask(&mask);
        }
        Ok(initial)
    }

    fn is_fully_collapsed(&self) -> bool {
        self.accommodated_total == 0
    }

    fn prepare_nodes_for_iteration(&mut self) {
        self.accommodate_node_indices_index = 0;
        self.random_instance.shuffle(&mut self.accommodate_node_indices);
        self.accommodated_total = 0;
        self.impacted_node_indices.clear();
    }

    fn is_done_accommodating_nodes(&self) -> bool {
        self.accommodate_node_indices_index == self.accommodate_node_indices_length
    }

    fn is_current_node_in_conflict(&mut self) -> bool {
        let ci = self.accommodate_node_indices[self.accommodate_node_indices_index];
        let node = &self.collapsable_nodes[ci];
        let mut conflict = node.node_state_indexed_view.is_current_state_restricted();

        if self.impacted_node_indices.contains(&ci) {
            conflict = false;
        } else {
            for &pi in &node.parent_neighbor_node_indices {
                if self.impacted_node_indices.contains(&pi) {
                    conflict = false;
                    break;
                }
            }
        }

        if !conflict {
            self.accommodate_node_indices_index += 1;
        }
        conflict
    }

    fn accommodate_current_node(&mut self) -> Vec<CollapsedNodeState<TNodeState>> {
        let mut changed_states = Vec::new();
        let mut state_changes: Vec<(usize, TNodeState, TNodeState, String)> = Vec::new();

        let current_index = self.accommodate_node_indices[self.accommodate_node_indices_index];
        self.impacted_node_indices.insert(current_index);

        // Collect parent indices first
        let parent_indices: Vec<usize> = self.collapsable_nodes[current_index].parent_neighbor_node_indices.clone();

        for &parent_index in &parent_indices {
            self.impacted_node_indices.insert(parent_index);

            // Read original state and id (owned copies)
            let original_state: TNodeState = (*self.collapsable_nodes[parent_index].node_state_indexed_view.get().unwrap()).clone();
            let parent_id = String::from(self.collapsable_nodes[parent_index].id);

            let mut current_state = original_state.clone();
            let mut is_restrictive = true;

            while is_restrictive {
                // Get the mask from parent@current_state targeting current_index (owned clone)
                let restricting_mask = get_restricting_mask_from_parent_to_current(
                    &self.collapsable_nodes,
                    parent_index,
                    current_index,
                    &current_state,
                );

                let is_this_restrictive = match restricting_mask {
                    Some(ref mask) => {
                        // Check if mask restricts current node — need to read current node state
                        let current_mask_data = {
                            let cn = &self.collapsable_nodes[current_index];
                            cn.node_state_indexed_view.is_mask_restrictive_to_current_state(mask)
                        };
                        current_mask_data
                    }
                    None => false,
                };

                if !is_this_restrictive {
                    is_restrictive = false;
                    // Move parent to current_state
                    {
                        let parent = &mut self.collapsable_nodes[parent_index];
                        loop {
                            let cs = parent.node_state_indexed_view.get().unwrap();
                            if **cs == current_state {
                                break;
                            }
                            parent.node_state_indexed_view.move_next();
                        }
                    }

                    if current_state != original_state {
                        changed_states.push(CollapsedNodeState {
                            node_id: parent_id.clone(),
                            node_state_id: Some(current_state.clone()),
                        });
                        state_changes.push((parent_index, original_state.clone(), current_state.clone(), parent_id.clone()));
                    }
                } else {
                    {
                        let parent = &mut self.collapsable_nodes[parent_index];
                        parent.node_state_indexed_view.move_next();
                    }
                    let next_state: TNodeState = (*self.collapsable_nodes[parent_index].node_state_indexed_view.get().unwrap()).clone();
                    if next_state == original_state {
                        break;
                    }
                    current_state = next_state;
                }
            }
        }

        // Apply mask changes
        for (parent_index, original_state, current_state, _id) in state_changes {
            let subtract = collect_masks_for_state(&self.collapsable_nodes, parent_index, &original_state);
            for (target, mask) in subtract {
                self.collapsable_nodes[target].subtract_mask(&mask);
            }
            let add = collect_masks_for_state(&self.collapsable_nodes, parent_index, &current_state);
            for (target, mask) in add {
                self.collapsable_nodes[target].add_mask(&mask);
            }
        }

        self.accommodated_total += 1;
        changed_states
    }

    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut m = HashMap::new();
        for node in &self.collapsable_nodes {
            let state: TNodeState = (*node.node_state_indexed_view.get().unwrap()).clone();
            m.insert(String::from(node.id), state);
        }
        CollapsedWaveFunction { node_state_per_node_id: m }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for AccommodatingCollapsableWaveFunction<'a, TNodeState> {
    fn new(
        collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
        random_instance: fastrand::Rng,
    ) -> Self {
        AccommodatingCollapsableWaveFunction {
            collapsable_nodes,
            accommodate_node_indices: Vec::new(),
            accommodate_node_indices_length: 0,
            accommodate_node_indices_index: 0,
            accommodated_total: 0,
            impacted_node_indices: HashSet::new(),
            random_instance,
        }
    }

    fn collapse(&mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {
        let init = self.initialize_nodes();
        if init.is_err() {
            return Err(init.err().unwrap());
        }
        let mut iterations_total: u32 = 0;
        while !self.is_fully_collapsed() {
            self.prepare_nodes_for_iteration();
            while !self.is_done_accommodating_nodes() {
                if self.is_current_node_in_conflict() {
                    self.accommodate_current_node();
                }
                iterations_total += 1;
            }
        }
        debug!("fully collapsed after {:?} iterations", iterations_total);
        Ok(self.get_collapsed_wave_function())
    }

    fn collapse_into_steps(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {
        let mut collapsed_node_states = Vec::new();
        let init = self.initialize_nodes();
        if init.is_err() {
            return Err(init.err().unwrap());
        }
        collapsed_node_states.extend(init.unwrap());
        while !self.is_fully_collapsed() {
            self.prepare_nodes_for_iteration();
            while !self.is_done_accommodating_nodes() {
                if self.is_current_node_in_conflict() {
                    collapsed_node_states.extend(self.accommodate_current_node());
                }
            }
        }
        Ok(collapsed_node_states)
    }
}
