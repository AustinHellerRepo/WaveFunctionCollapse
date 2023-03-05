use std::{rc::Rc, cell::RefCell, collections::{HashMap, HashSet}, marker::PhantomData};
use std::hash::Hash;
use bitvec::vec::BitVec;
use rand::seq::SliceRandom;
use crate::wave_function::indexed_view::IndexedViewMaskState;
use super::collapsable_wave_function::{CollapsableNode, CollapsedNodeState, CollapsedWaveFunction, CollapsableWaveFunction};

pub struct AccommodatingSequentialCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    spread_node_ids: Vec<&'a str>,
    spread_node_ids_length: usize,
    spread_node_ids_index: usize,
    impacted_node_ids: HashSet<&'a str>,
    stash_per_neighbor_node_id: HashMap<&'a str, IndexedViewMaskState>,
    original_node_state_per_node_id: HashMap<&'a str, &'a TNodeState>,
    current_neighbor_node_ids: Vec<&'a str>,
    great_neighbor_node_ids_per_neighbor_node_id: HashMap<&'a str, Vec<&'a str>>,
    nongreat_neighbor_node_ids_per_neighbor_node_id: HashMap<&'a str, Vec<&'a str>>,
    current_neighbor_node_ids_index: usize,
    current_neighbor_node_ids_length: usize,
    is_current_neighbor_node_cycle_required: bool,
    is_current_node_neighbors_collapse_possible: bool,
    node_state_type: PhantomData<TNodeState>
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> AccommodatingSequentialCollapsableWaveFunction<'a, TNodeState> {
    fn initialize_nodes(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {

        // initialize each collapsable node to its first (random) state, storing them for the return
        // alter masks for every collapsable node to its neighbors
        // initialize the collapsable_nodes vector

        let mut initial_node_states: Vec<CollapsedNodeState<TNodeState>> = Vec::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let mut collapsable_node = wrapped_collapsable_node.borrow_mut();
            if !collapsable_node.node_state_indexed_view.try_move_next() {
                return Err(String::from("Cannot collapse wave function."));
            }
            
            self.spread_node_ids.push(collapsable_node.id);
            let node_state = collapsable_node.node_state_indexed_view.get().unwrap();
            let collapsed_node_state: CollapsedNodeState<TNodeState> = CollapsedNodeState {
                node_id: String::from(collapsable_node.id),
                node_state_id: Some((*node_state).clone())
            };
            debug!("node {:?} is currently in state {:?}", collapsable_node.id, node_state);
            initial_node_states.push(collapsed_node_state);
        }
        self.spread_node_ids_length = self.spread_node_ids.len();
        debug!("set spread_node_ids_length to {}", self.spread_node_ids_length);

        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            let node_state = collapsable_node.node_state_indexed_view.get().unwrap();
            let neighbor_node_ids: &Vec<&str> = &collapsable_node.neighbor_node_ids;
            let mask_per_neighbor_per_state: &HashMap<&TNodeState, HashMap<&str, BitVec>> = &collapsable_node.mask_per_neighbor_per_state;
            if let Some(mask_per_neighbor) = mask_per_neighbor_per_state.get(node_state) {
                for neighbor_node_id in neighbor_node_ids.iter() {
                    if mask_per_neighbor.contains_key(neighbor_node_id) {
                        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                        //debug!("looking for mask from parent {:?} to child {:?}.", collapsable_node.id, neighbor_node_id);
                        //debug!("mask_per_neighbor: {:?}", mask_per_neighbor);
                        let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                        neighbor_collapsable_node.add_mask(mask);
                        debug!("adding mask to {:?} from {:?} when in initialize_nodes", neighbor_node_id, collapsable_node.id);
                    }
                }
            }
        }

        Ok(initial_node_states)
    }
    fn is_fully_collapsed(&self) -> bool {

        // returns if the temp_recently_accommodated_nodes is empty

        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let mut collapsable_node = wrapped_collapsable_node.borrow_mut();
            if collapsable_node.is_fully_restricted() {
                debug!("collapsable node is fully restricted: {:?}", collapsable_node.id);
                debug!("index view: {:?}", collapsable_node.node_state_indexed_view);
                return false;
            }
        }
        true
    }
    fn prepare_nodes_for_iteration(&mut self) {

        // shuffle collapsable nodes
        // initialize pointer to first element of collapsable_nodes
        // reset spread node total

        debug!("prior to being prepared: {:?}", self.spread_node_ids);

        self.spread_node_ids_index = 0;
        self.spread_node_ids.shuffle(&mut rand::thread_rng());  // TODO use a provided random instance for deterministic results
        self.impacted_node_ids.clear();
     
        debug!("after being prepared: {:?}", self.spread_node_ids);
    }
    fn is_done_spreading_nodes(&self) -> bool {

        // returns if pointer is outside the bounds of the collapsable_nodes

        self.spread_node_ids_index == self.spread_node_ids_length
    }
    fn is_current_node_in_conflict(&mut self) -> bool {

        // returns if current collapsable node is in conflict and not already impacted

        if false {
            let current_collapsable_node_id: &str = self.spread_node_ids[self.spread_node_ids_index];
            let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(current_collapsable_node_id).unwrap();
            let current_collapsable_node = wrapped_current_collapsable_node.borrow();
            debug!("node {:?} is assumed to always be in conflict regardless of it being in state {:?}", current_collapsable_node_id, current_collapsable_node.node_state_indexed_view.get().unwrap());
            return true;
        }

        let current_collapsable_node_id: &str = self.spread_node_ids[self.spread_node_ids_index];
        let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(current_collapsable_node_id).unwrap();
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        let mut is_current_collapsable_node_in_conflict = current_collapsable_node.node_state_indexed_view.is_current_state_restricted();

        if !is_current_collapsable_node_in_conflict {
            for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                let neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow();
                if neighbor_collapsable_node.node_state_indexed_view.is_current_state_restricted() {
                    is_current_collapsable_node_in_conflict = true;
                    break;
                }
            }

            if !is_current_collapsable_node_in_conflict {
                for neighbor_node_id in current_collapsable_node.parent_neighbor_node_ids.iter() {
                    let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                    let neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow();
                    if neighbor_collapsable_node.node_state_indexed_view.is_current_state_restricted() {
                        is_current_collapsable_node_in_conflict = true;
                        break;
                    }
                }
            }
        }

        if self.impacted_node_ids.contains(current_collapsable_node_id) {
            debug!("current node was already impacted");
            is_current_collapsable_node_in_conflict = false;
        }
        else {
            for parent_neighbor_node_id in current_collapsable_node.parent_neighbor_node_ids.iter() {
                if self.impacted_node_ids.contains(parent_neighbor_node_id) {
                    debug!("current node's parent neighbor node {:?} was already impacted", parent_neighbor_node_id);
                    is_current_collapsable_node_in_conflict = false;
                    break;
                }
            }
            if !is_current_collapsable_node_in_conflict {
                for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                    if self.impacted_node_ids.contains(neighbor_node_id) {
                        debug!("current node's child neighbor node {:?} was already impacted", neighbor_node_id);
                        is_current_collapsable_node_in_conflict = false;
                        break;
                    }
                }
            }
        }

        if !is_current_collapsable_node_in_conflict {
            debug!("node {:?} is not in conflict while in state {:?}", current_collapsable_node_id, current_collapsable_node.node_state_indexed_view.get().unwrap());
        }
        else {
            debug!("node {:?} is in conflict while in state {:?}", current_collapsable_node_id, current_collapsable_node.node_state_indexed_view.get().unwrap());
        }

        is_current_collapsable_node_in_conflict
    }
    fn prepare_current_node_neighbors(&mut self) {

        // cache all relevant neighbor nodes (parents and children together)
        // remove current collapsable node mask from neighbors
        // remove each neighbor's masks from all other nodes
        // cache the state from each neighbor
        // cache the stash from each neighbor
        // add current collapsable node masks to neighbors
        // randomize order of neighbor nodes
        // cache great neighbor node ids per neighbor (excluding other nodes)
        // cache non-great neighbor node ids per neighbor (only other nodes)
        // initialize neighbor pointer to first neighbor
        // set current neighbor node cycle not required
        // set neighbors collapse possible true

        let current_collapsable_node_id: &str = self.spread_node_ids[self.spread_node_ids_index];

        // cache all relevant neighbor nodes (parents and children together)
        // remove current collapsable node mask from neighbors
        {
            let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(current_collapsable_node_id).unwrap();
            let current_collapsable_node = wrapped_current_collapsable_node.borrow();

            self.current_neighbor_node_ids.extend(current_collapsable_node.neighbor_node_ids.clone());
            self.current_neighbor_node_ids.extend(current_collapsable_node.parent_neighbor_node_ids.clone());
            self.current_neighbor_node_ids.sort();  // NOTE: without sorting, dedup does not necessarily find all duplicates
            self.current_neighbor_node_ids.dedup();
            debug!("caching current neighbor nodes: {:?}", self.current_neighbor_node_ids);

            let current_collapsable_node_state = current_collapsable_node.node_state_indexed_view.get().unwrap();
            if current_collapsable_node.mask_per_neighbor_per_state.contains_key(current_collapsable_node_state) {
                let mask_per_neighbor = current_collapsable_node.mask_per_neighbor_per_state.get(current_collapsable_node_state).unwrap();
                for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                    if mask_per_neighbor.contains_key(neighbor_node_id) {
                        let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                        neighbor_collapsable_node.subtract_mask(mask);
                    }
                }
            }

            debug!("removed current node mask from neighbors");
        }

        // remove each neighbor's masks from all other nodes
        // cache the state from each neighbor
        {
            for neighbor_node_id in self.current_neighbor_node_ids.iter() {
                let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                let neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow();
                let neighbor_collapsable_node_state = neighbor_collapsable_node.node_state_indexed_view.get().unwrap();
                
                self.original_node_state_per_node_id.insert(neighbor_node_id, neighbor_collapsable_node_state);

                if neighbor_collapsable_node.mask_per_neighbor_per_state.contains_key(neighbor_collapsable_node_state) {
                    let mask_per_neighbor = neighbor_collapsable_node.mask_per_neighbor_per_state.get(neighbor_collapsable_node_state).unwrap();
                    for great_neighbor_node_id in neighbor_collapsable_node.neighbor_node_ids.iter() {
                        if mask_per_neighbor.contains_key(great_neighbor_node_id) {
                            let mask = mask_per_neighbor.get(great_neighbor_node_id).unwrap();
                            let wrapped_great_neighbor_collapsable_node = self.collapsable_node_per_id.get(great_neighbor_node_id).unwrap();
                            let mut great_neighbor_collapsable_node = wrapped_great_neighbor_collapsable_node.borrow_mut();
                            great_neighbor_collapsable_node.subtract_mask(mask);
                        }
                    }
                }
            }

            debug!("removed parent and child neighbors' masks from their child neighbors");
        }

        // cache the stash from each neighbor
        {
            for neighbor_node_id in self.current_neighbor_node_ids.iter() {
                let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                let indexed_view_mask_state = neighbor_collapsable_node.node_state_indexed_view.stash_mask_state();
                
                self.stash_per_neighbor_node_id.insert(neighbor_node_id, indexed_view_mask_state);
            }

            debug!("stashing masks from parent and child neighbors, making them fully unmasked");
        }

        // add current collapsable node masks to neighbors
        {
            let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(current_collapsable_node_id).unwrap();
            let current_collapsable_node = wrapped_current_collapsable_node.borrow();
            let current_collapsable_node_state = current_collapsable_node.node_state_indexed_view.get().unwrap();
            if current_collapsable_node.mask_per_neighbor_per_state.contains_key(current_collapsable_node_state) {
                let mask_per_neighbor = current_collapsable_node.mask_per_neighbor_per_state.get(current_collapsable_node_state).unwrap();
                for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                    if mask_per_neighbor.contains_key(neighbor_node_id) {
                        let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                        neighbor_collapsable_node.add_mask(mask);
                    }
                }
            }

            debug!("added current node masks to neighbors");
        }

        // randomize order of neighbor nodes
        self.current_neighbor_node_ids.shuffle(&mut rand::thread_rng());
        debug!("shuffled neighbors: {:?}", self.current_neighbor_node_ids);

        // cache great neighbor node ids per neighbor (excluding other nodes)
        {
            for neighbor_node_id in self.current_neighbor_node_ids.iter() {
                let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                let neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow();

                let mut great_neighbor_node_ids: Vec<&str> = Vec::new();
                let mut nongreat_neighbor_node_ids: Vec<&str> = Vec::new();

                let mut possible_great_neighbor_node_ids: Vec<&str> = Vec::new();
                possible_great_neighbor_node_ids.extend(neighbor_collapsable_node.neighbor_node_ids.clone());
                possible_great_neighbor_node_ids.extend(neighbor_collapsable_node.parent_neighbor_node_ids.clone());
                possible_great_neighbor_node_ids.sort();
                possible_great_neighbor_node_ids.dedup();

                for possible_great_neighbor_node_id in possible_great_neighbor_node_ids.iter() {
                    if *possible_great_neighbor_node_id == current_collapsable_node_id || self.current_neighbor_node_ids.contains(possible_great_neighbor_node_id) {
                        great_neighbor_node_ids.push(possible_great_neighbor_node_id);
                    }
                    else {
                        nongreat_neighbor_node_ids.push(possible_great_neighbor_node_id);
                    }
                }
                self.great_neighbor_node_ids_per_neighbor_node_id.insert(neighbor_node_id, great_neighbor_node_ids);
                self.nongreat_neighbor_node_ids_per_neighbor_node_id.insert(neighbor_node_id, nongreat_neighbor_node_ids);
            }
        }

        // initialize neighbor pointer to first neighbor
        self.current_neighbor_node_ids_index = 0;
        self.current_neighbor_node_ids_length = self.current_neighbor_node_ids.len();

        // set current neighbor node cycle not required
        self.is_current_neighbor_node_cycle_required = false;

        // set neighbor collapse possible true
        self.is_current_node_neighbors_collapse_possible = true;
    }
    fn is_current_node_neighbors_collapsed(&self) -> bool {

        // while pointer is inside the bounds and neighbors are possible

        let is_neighbor_index_within_bounds = self.current_neighbor_node_ids_index < self.current_neighbor_node_ids_length;
        debug!("is_neighbor_index_within_bounds: {:?}", is_neighbor_index_within_bounds);
        debug!("is_current_node_neighbors_collapse_possible: {:?}", self.is_current_node_neighbors_collapse_possible);
        !(is_neighbor_index_within_bounds && self.is_current_node_neighbors_collapse_possible)
    }
    fn is_current_node_neighbor_state_change_required(&self) -> bool {

        // if the neighbor is in a restricted state or neighbor node cycle is required

        if self.is_current_neighbor_node_cycle_required {
            true
        }
        else {
            let neighbor_node_id = self.current_neighbor_node_ids[self.current_neighbor_node_ids_index];
            let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
            let neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow();
            neighbor_collapsable_node.node_state_indexed_view.is_current_state_restricted()
        }
    }
    fn change_state_of_current_node_neighbor(&mut self) -> Vec<CollapsedNodeState<TNodeState>> {

        // set current neighbor cycle not required
        // try to cycle the neighbor node state
        // if it was successful
        //     try to inform the current node and neighbor nodes of their new restrictions
        //     if all affected nodes have at least one valid state and are not currently restricted
        //         move the neighbor pointer to the next neighbor
        //     else
        //         set current neighbor node cycle required
        // else the cached node state was rediscovered after cycling
        //     if the pointer is at the first neighbor
        //         set neighbors collapse possible false
        //     else
        //         revert to the previous neighbor node so that it can try a different state
        //         set current neighbor node cycle required

        let mut changed_neighbor_node_states: Vec<CollapsedNodeState<TNodeState>> = Vec::new();

        self.is_current_neighbor_node_cycle_required = false;

        let neighbor_node_id = self.current_neighbor_node_ids[self.current_neighbor_node_ids_index];
        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();

        let original_neighbor_node_state = self.original_node_state_per_node_id.get(neighbor_node_id).unwrap();

        debug!("neighbor node trying to cycle: {:?}", neighbor_collapsable_node.id);
        let is_successful_neighbor_nove_next_cycle = neighbor_collapsable_node.node_state_indexed_view.try_move_next_cycle(original_neighbor_node_state);
        let neighbor_collapsable_node_state = neighbor_collapsable_node.node_state_indexed_view.get().unwrap();

        changed_neighbor_node_states.push(CollapsedNodeState {
            node_id: String::from(neighbor_node_id),
            node_state_id: Some((*neighbor_collapsable_node_state).clone())
        });
        
        if is_successful_neighbor_nove_next_cycle {
            debug!("successfully move next cycled");
            let neighbor_node_state = neighbor_collapsable_node.node_state_indexed_view.get().unwrap();
            if neighbor_collapsable_node.mask_per_neighbor_per_state.contains_key(neighbor_collapsable_node_state) {
                let mask_per_neighbor = neighbor_collapsable_node.mask_per_neighbor_per_state.get(neighbor_node_state).unwrap();
                let great_neighbor_node_ids = self.great_neighbor_node_ids_per_neighbor_node_id.get(neighbor_node_id).unwrap();
                let mut masked_great_neighbor_node_ids: Vec<&str> = Vec::new();
                let mut is_rollback_required: bool = false;

                for great_neighbor_node_id in great_neighbor_node_ids.iter() {
                    if mask_per_neighbor.contains_key(great_neighbor_node_id) {
                        let mask = mask_per_neighbor.get(great_neighbor_node_id).unwrap();
                        let wrapped_great_neighbor_collapsable_node = self.collapsable_node_per_id.get(great_neighbor_node_id).unwrap();
                        let mut great_neighbor_collapsable_node = wrapped_great_neighbor_collapsable_node.borrow_mut();

                        if !great_neighbor_collapsable_node.node_state_indexed_view.is_mask_restrictive_to_current_state(mask) {

                            great_neighbor_collapsable_node.add_mask(mask);
                            masked_great_neighbor_node_ids.push(great_neighbor_node_id);
                        }
                        else {
                            is_rollback_required = true;
                            break;
                        }
                    }
                }

                if is_rollback_required {
                    debug!("rollback required after over-restricting neighbors");
                    for great_neighbor_node_id in masked_great_neighbor_node_ids.iter() {
                        let mask = mask_per_neighbor.get(great_neighbor_node_id).unwrap();
                        let wrapped_great_neighbor_collapsable_node = self.collapsable_node_per_id.get(great_neighbor_node_id).unwrap();
                        let mut great_neighbor_collapsable_node = wrapped_great_neighbor_collapsable_node.borrow_mut();
                        great_neighbor_collapsable_node.subtract_mask(mask);
                    }
                    self.is_current_neighbor_node_cycle_required = true;
                }
                else {
                    debug!("maintaining state was successful, move to next neighbor (1)");
                    self.current_neighbor_node_ids_index += 1;
                }
            }
            else {
                debug!("maintaining state was successful, move to next neighbor (2)");
                self.current_neighbor_node_ids_index += 1;
            }
        }
        else {
            debug!("failed to move next cycle");
            if self.current_neighbor_node_ids_index == 0 {
                self.is_current_node_neighbors_collapse_possible = false;
            }
            else {
                debug!("move back a neighbor");
                self.current_neighbor_node_ids_index -= 1;
                self.is_current_neighbor_node_cycle_required = true;

                // remove masks from great neighbor nodes
                let previous_neighbor_node_id = self.current_neighbor_node_ids[self.current_neighbor_node_ids_index];
                let wrapped_previous_neighbor_collapsable_node = self.collapsable_node_per_id.get(previous_neighbor_node_id).unwrap();
                let previous_neighbor_collapsable_node = wrapped_previous_neighbor_collapsable_node.borrow();
                let previous_neighbor_node_state = previous_neighbor_collapsable_node.node_state_indexed_view.get().unwrap();
                if previous_neighbor_collapsable_node.mask_per_neighbor_per_state.contains_key(previous_neighbor_node_state) {
                    let mask_per_neighbor = previous_neighbor_collapsable_node.mask_per_neighbor_per_state.get(previous_neighbor_node_state).unwrap();
                    let great_neighbor_node_ids = self.great_neighbor_node_ids_per_neighbor_node_id.get(previous_neighbor_node_id).unwrap();
                    for great_neighbor_node_id in great_neighbor_node_ids.iter() {
                        if mask_per_neighbor.contains_key(great_neighbor_node_id) {
                            let mask = mask_per_neighbor.get(great_neighbor_node_id).unwrap();
                            if *great_neighbor_node_id == neighbor_node_id {
                                neighbor_collapsable_node.subtract_mask(mask);
                            }
                            else {
                                let wrapped_great_neighbor_collapsable_node = self.collapsable_node_per_id.get(great_neighbor_node_id).unwrap();
                                let mut great_neighbor_collapsable_node = wrapped_great_neighbor_collapsable_node.borrow_mut();
                                great_neighbor_collapsable_node.subtract_mask(mask);
                            }
                        }
                    }
                }
            }
        }

        changed_neighbor_node_states
    }
    fn allow_current_node_neighbor_to_maintain_state(&mut self) {

        // try to inform the current node and neighbor nodes of their new restrictions
        // if all affected nodes have at least one valid state and are not currently restricted
        //     move the neighbor pointer to the next neighbor
        // else
        //     set current neighbor node cycle required

        let neighbor_node_id = self.current_neighbor_node_ids[self.current_neighbor_node_ids_index];
        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
        let neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow();
        let neighbor_node_state = neighbor_collapsable_node.node_state_indexed_view.get().unwrap();

        if neighbor_collapsable_node.mask_per_neighbor_per_state.contains_key(neighbor_node_state) {
            let mask_per_neighbor = neighbor_collapsable_node.mask_per_neighbor_per_state.get(neighbor_node_state).unwrap();
            let great_neighbor_node_ids = self.great_neighbor_node_ids_per_neighbor_node_id.get(neighbor_node_id).unwrap();
            let mut masked_great_neighbor_node_ids: Vec<&str> = Vec::new();
            let mut is_rollback_required: bool = false;
            
            for great_neighbor_node_id in great_neighbor_node_ids.iter() {
                if mask_per_neighbor.contains_key(great_neighbor_node_id) {
                    let mask = mask_per_neighbor.get(great_neighbor_node_id).unwrap();
                    let wrapped_great_neighbor_collapsable_node = self.collapsable_node_per_id.get(great_neighbor_node_id).unwrap();
                    let mut great_neighbor_collapsable_node = wrapped_great_neighbor_collapsable_node.borrow_mut();
                    if !great_neighbor_collapsable_node.node_state_indexed_view.is_mask_restrictive_to_current_state(mask) {

                        great_neighbor_collapsable_node.add_mask(mask);
                        masked_great_neighbor_node_ids.push(great_neighbor_node_id);
                    }
                    else {
                        is_rollback_required = true;
                        break;
                    }
                }
            }

            if is_rollback_required {
                debug!("rollback required after over-restricting neighbors");
                for great_neighbor_node_id in masked_great_neighbor_node_ids.iter() {
                    let mask = mask_per_neighbor.get(great_neighbor_node_id).unwrap();
                    let wrapped_great_neighbor_collapsable_node = self.collapsable_node_per_id.get(great_neighbor_node_id).unwrap();
                    let mut great_neighbor_collapsable_node = wrapped_great_neighbor_collapsable_node.borrow_mut();
                    great_neighbor_collapsable_node.subtract_mask(mask);
                }
                self.is_current_neighbor_node_cycle_required = true;
            }
            else {
                debug!("maintaining state was successful, move to next neighbor (1)");
                self.current_neighbor_node_ids_index += 1;
            }
        }
        else {
            debug!("maintaining state was successful, move to next neighbor (2)");
            self.current_neighbor_node_ids_index += 1;
        }
    }
    fn cleanup_current_node_neighbors(&mut self) {

        // if pointer is outside the bounds
        //     cache impacted nodes
        //     add neighbor masks only to other nodes
        // else
        //     add neighbor masks to all of their neighbors and other nodes
        // unstash the neighbors
        // clear cached state of each neighbor
        // clear cached great neighbor node ids
        // clear cached non-great neighbor node ids
        // clear cache of all relevant neighbor node ids

        if self.current_neighbor_node_ids_index == self.current_neighbor_node_ids_length {
            let current_collapsable_node_id: &str = self.spread_node_ids[self.spread_node_ids_index];
            self.impacted_node_ids.insert(current_collapsable_node_id);
            self.impacted_node_ids.extend(self.current_neighbor_node_ids.clone());

            for neighbor_node_id in self.current_neighbor_node_ids.iter() {
                let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                let neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow();
                let neighbor_node_state = neighbor_collapsable_node.node_state_indexed_view.get().unwrap();
                if neighbor_collapsable_node.mask_per_neighbor_per_state.contains_key(neighbor_node_state) {
                    let mask_per_neighbor = neighbor_collapsable_node.mask_per_neighbor_per_state.get(neighbor_node_state).unwrap();

                    let nongreat_neighbor_node_ids = self.nongreat_neighbor_node_ids_per_neighbor_node_id.get(neighbor_node_id).unwrap();
                    for nongreat_neighbor_node_id in nongreat_neighbor_node_ids.iter() {
                        if mask_per_neighbor.contains_key(nongreat_neighbor_node_id) {
                            let mask = mask_per_neighbor.get(nongreat_neighbor_node_id).unwrap();
                            let wrapped_nongreat_neighbor_collapsable_node = self.collapsable_node_per_id.get(nongreat_neighbor_node_id).unwrap();
                            let mut nongreat_collapsable_node = wrapped_nongreat_neighbor_collapsable_node.borrow_mut();
                            nongreat_collapsable_node.add_mask(mask);
                        }
                    }
                }
            }
        }
        else {
            for neighbor_node_id in self.current_neighbor_node_ids.iter() {
                let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                let neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow();
                let neighbor_node_state = neighbor_collapsable_node.node_state_indexed_view.get().unwrap();
                if neighbor_collapsable_node.mask_per_neighbor_per_state.contains_key(neighbor_node_state) {
                    let mask_per_neighbor = neighbor_collapsable_node.mask_per_neighbor_per_state.get(neighbor_node_state).unwrap();

                    for all_great_neighbor_node_id in neighbor_collapsable_node.neighbor_node_ids.iter() {
                        if mask_per_neighbor.contains_key(all_great_neighbor_node_id) {
                            let mask = mask_per_neighbor.get(all_great_neighbor_node_id).unwrap();
                            let wrapped_nongreat_neighbor_collapsable_node = self.collapsable_node_per_id.get(all_great_neighbor_node_id).unwrap();
                            let mut nongreat_collapsable_node = wrapped_nongreat_neighbor_collapsable_node.borrow_mut();
                            nongreat_collapsable_node.add_mask(mask);
                        }
                    }
                }
            }
        }

        for (neighbor_node_id, mask_state) in self.stash_per_neighbor_node_id.iter_mut() {
            let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
            let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
            neighbor_collapsable_node.node_state_indexed_view.unstash_mask_state(mask_state);
        }

        self.original_node_state_per_node_id.clear();
        self.great_neighbor_node_ids_per_neighbor_node_id.clear();
        self.nongreat_neighbor_node_ids_per_neighbor_node_id.clear();
        self.current_neighbor_node_ids.clear();

    }
    fn move_to_next_node(&mut self) {

        // increment pointer

        self.spread_node_ids_index += 1;
    }
    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut node_state_per_node: HashMap<String, TNodeState> = HashMap::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            let node_state: TNodeState = (*collapsable_node.node_state_indexed_view.get().unwrap()).clone();
            let node: String = String::from(collapsable_node.id);
            debug!("established node {node} in state {:?}.", node_state);
            node_state_per_node.insert(node, node_state);
        }
        CollapsedWaveFunction {
            node_state_per_node: node_state_per_node
        }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for AccommodatingSequentialCollapsableWaveFunction<'a, TNodeState> {
    fn new(collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>, collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>) -> Self {
        AccommodatingSequentialCollapsableWaveFunction {
            collapsable_nodes: collapsable_nodes,
            collapsable_node_per_id: collapsable_node_per_id,
            spread_node_ids: Vec::new(),
            spread_node_ids_length: 0,
            spread_node_ids_index: 0,
            impacted_node_ids: HashSet::new(),
            stash_per_neighbor_node_id: HashMap::new(),
            original_node_state_per_node_id: HashMap::new(),
            current_neighbor_node_ids: Vec::new(),
            great_neighbor_node_ids_per_neighbor_node_id: HashMap::new(),
            nongreat_neighbor_node_ids_per_neighbor_node_id: HashMap::new(),
            current_neighbor_node_ids_index: 0,
            current_neighbor_node_ids_length: 0,
            is_current_neighbor_node_cycle_required: false,
            is_current_node_neighbors_collapse_possible: true,
            node_state_type: PhantomData
        }
    }
    fn collapse(&'a mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {
      
        let mut iterations_total: u32 = 0;

        debug!("initializing node states");
        let initialized_node_states_result = self.initialize_nodes();
        if initialized_node_states_result.is_err() {
            return Err(initialized_node_states_result.err().unwrap());
        }

        debug!("beginning main while loop");
        while !self.is_fully_collapsed() {
            debug!("preparing nodes for iteration");
            self.prepare_nodes_for_iteration();
            debug!("while not done spreading nodes");
            while !self.is_done_spreading_nodes() {
                debug!("checking if current node is in conflict");
                if self.is_current_node_in_conflict() {
                    debug!("preparing current node neighbors");
                    self.prepare_current_node_neighbors();
                    debug!("while current node neighbors not yet collapsed");
                    while !self.is_current_node_neighbors_collapsed() {
                        debug!("checking if current node neighbor state change is required");
                        if self.is_current_node_neighbor_state_change_required() {
                            debug!("changing state of current node neighbor");
                            self.change_state_of_current_node_neighbor();
                        }
                        else {
                            debug!("allowing current node neighbor to maintain its state");
                            self.allow_current_node_neighbor_to_maintain_state();
                        }
                    }
                    debug!("cleaning up current node neighbors");
                    self.cleanup_current_node_neighbors();
                }
                debug!("moving to next collapsable node at index {:?}", self.spread_node_ids_index);
                self.move_to_next_node();
            }
            iterations_total += 1;
        }

        debug!("fully collapsed after {:?} iterations", iterations_total);

        Ok(self.get_collapsed_wave_function())
    }
    fn collapse_into_steps(&'a mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {

        // initialize each collapsable node to its first (random) state
        // alter masks for every collapsable node to its neighbors
        // initialize the collapsable_nodes vector
        // while not yet fully collapsed
        //      shuffle collapsable nodes
        //      initialize pointer to first element of collapsable_nodes
        //      reset spread node total
        //      while pointer is inside the bounds
        //          if current collapsable node is in conflict and not already impacted
        //              cache all relevant neighbor nodes (parents and children together)
        //              remove current collapsable node mask from neighbors
        //              remove each neighbor's masks from all other nodes
        //              cache the state from each neighbor
        //              cache the stash from each neighbor
        //              add current collapsable node masks to neighbors
        //              randomize order of neighbor nodes
        //              cache great neighbor node ids per neighbor (excluding other nodes)
        //              cache non-great neighbor node ids per neighbor (only other nodes)
        //              initialize neighbor pointer to first neighbor
        //              set current neighbor node cycle not required
        //              set neighbors collapse possible true
        //              while pointer is inside the bounds and neighbors are possible
        //                  if the neighbor is in a restricted state or neighbor node cycle is required
        //                      set current neighbor cycle not required
        //                      try to cycle the neighbor node state
        //                      if it was successful
        //                          try to inform the current node and neighbor nodes of their new restrictions
        //                          if all affected nodes have at least one valid state and are not currently restricted
        //                              move the neighbor pointer to the next neighbor
        //                          else
        //                              set current neighbor node cycle required
        //                      else the cached node state was rediscovered after cycling
        //                          if the pointer is at the first neighbor
        //                              set neighbors collapse possible false
        //                          else
        //                              revert to the previous neighbor node so that it can try a different state
        //                              set current neighbor node cycle required
        //                  else
        //                      try to inform the current node and neighbor nodes of their new restrictions
        //                      if all affected nodes have at least one valid state and are not currently restricted
        //                          move the neighbor pointer to the next neighbor
        //                      else
        //                          set current neighbor node cycle required
        //              if pointer is outside the bounds
        //                  cache impacted nodes
        //                  add neighbor masks only to other nodes
        //              else
        //                  add neighbor masks to all of their neighbors and other nodes
        //              unstash the neighbors
        //              clear cached state of each neighbor
        //              clear cached great neighbor node ids
        //              clear cached non-great neighbor node ids
        //              clear cache of all relevant neighbor node ids
        //          increment pointer
        //
        // NOTE: this could cause an infinite loop for the AB<-->CD unit test

        let mut collapsed_node_states: Vec<CollapsedNodeState<TNodeState>> = Vec::new();

        let initialized_node_states_result = self.initialize_nodes();
        if initialized_node_states_result.is_err() {
            return Err(initialized_node_states_result.err().unwrap());
        }
        let initialized_node_states = initialized_node_states_result.unwrap();
        collapsed_node_states.extend(initialized_node_states);

        while !self.is_fully_collapsed() {
            self.prepare_nodes_for_iteration();
            while !self.is_done_spreading_nodes() {
                if self.is_current_node_in_conflict() {
                    self.prepare_current_node_neighbors();
                    while !self.is_current_node_neighbors_collapsed() {
                        if self.is_current_node_neighbor_state_change_required() {
                            let spreading_neighbor_node_state = self.change_state_of_current_node_neighbor();
                            collapsed_node_states.extend(spreading_neighbor_node_state);
                        }
                        else {
                            self.allow_current_node_neighbor_to_maintain_state();
                        }
                    }
                    self.cleanup_current_node_neighbors();
                }
                self.move_to_next_node();
            }
        }

        Ok(collapsed_node_states)
    }
}