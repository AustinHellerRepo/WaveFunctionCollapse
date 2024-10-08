use std::collections::HashSet;
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc, collections::HashMap};
use std::hash::Hash;
use bitvec::vec::BitVec;
use super::collapsable_wave_function::{CollapsableWaveFunction, CollapsableNode, CollapsedNodeState, CollapsedWaveFunction};

/// This struct represents a CollapsableWaveFunction that picks a random node, tries to get each parent to accommodate to the current state of the random node, repeating until all nodes are unrestricted. This is best for finding solutions when the condition problem has many possible solutions and you want a more random solution. If there are very few solutions, the wave function is uncollapsable by design, or there are certain types of cycles in the graph, this algorithm with perform poorly or never complete.
pub struct AccommodatingCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    accommodate_node_ids: Vec<&'a str>,
    accommodate_node_ids_length: usize,
    accommodate_node_ids_index: usize,
    accommodated_total: usize,
    impacted_node_ids: HashSet<&'a str>,
    random_instance: Rc<RefCell<fastrand::Rng>>,
    node_state_type: PhantomData<TNodeState>
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> AccommodatingCollapsableWaveFunction<'a, TNodeState> {
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
            
            self.accommodate_node_ids.push(collapsable_node.id);
            let node_state = collapsable_node.node_state_indexed_view.get().unwrap();
            let collapsed_node_state: CollapsedNodeState<TNodeState> = CollapsedNodeState {
                node_id: String::from(collapsable_node.id),
                node_state_id: Some((*node_state).clone())
            };
            initial_node_states.push(collapsed_node_state);
        }
        self.accommodate_node_ids_length = self.accommodate_node_ids.len();
        self.accommodated_total = self.accommodate_node_ids_length;

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
                        debug!("adding mask to {:?} when in initialize_nodes", neighbor_node_id);
                    }
                }
            }
        }

        Ok(initial_node_states)
    }
    fn is_fully_collapsed(&self) -> bool {

        // returns if the temp_recently_accommodated_nodes is empty

        self.accommodated_total == 0
    }
    fn prepare_nodes_for_iteration(&mut self) {

        // shuffle collapsable nodes
        // initialize pointer to first element of collapsable_nodes

        debug!("prior to being prepared: {:?}", self.accommodate_node_ids);

        self.accommodate_node_ids_index = 0;
        self.random_instance.borrow_mut().shuffle(self.accommodate_node_ids.as_mut_slice());
        self.accommodated_total = 0;
        self.impacted_node_ids.clear();
     
        debug!("after being prepared: {:?}", self.accommodate_node_ids);
    }
    fn is_done_accommodating_nodes(&self) -> bool {

        // returns if pointer is outside the bounds of the collapsable_nodes

        self.accommodate_node_ids_index == self.accommodate_node_ids_length
    }
    fn is_current_node_in_conflict(&mut self) -> bool {

        // returns if the current state of the current node is restricted and not yet impacted
        // increment pointer if false

        let current_collapsable_node_id: &str = self.accommodate_node_ids[self.accommodate_node_ids_index];
        let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(current_collapsable_node_id).unwrap();
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        let mut is_current_collapsable_node_in_conflict = current_collapsable_node.node_state_indexed_view.is_current_state_restricted();

        if self.impacted_node_ids.contains(current_collapsable_node_id) {
            is_current_collapsable_node_in_conflict = false;
        }
        else {
            for parent_neighbor_node_id in current_collapsable_node.parent_neighbor_node_ids.iter() {
                if self.impacted_node_ids.contains(parent_neighbor_node_id) {
                    is_current_collapsable_node_in_conflict = false;
                    break;
                }
            }
        }

        if !is_current_collapsable_node_in_conflict {
            self.accommodate_node_ids_index += 1;
            debug!("node is not in conflict: {:?}", current_collapsable_node_id);
        }
        else {
            debug!("node is in conflict: {:?}", current_collapsable_node_id);
        }

        is_current_collapsable_node_in_conflict
    }
    fn accommodate_current_node(&mut self) -> Vec<CollapsedNodeState<TNodeState>> {

        // accommodate this collapsable node, storing the node states for the return
        // alter mask for neighbors
        // cache impacted nodes

        // NOTE: resetting the indexed_view for each accommodating parent significantly reduces the performance of this algorithm

        let mut changed_parent_node_states: Vec<CollapsedNodeState<TNodeState>> = Vec::new();
        let mut to_node_state_and_from_node_state_tuple_per_parent_node_id: HashMap<&str, (&TNodeState, &TNodeState)> = HashMap::new();

        // try to get each parent neighbor node to accommodate the current node
        {
            let current_collapsable_node_id: &str = self.accommodate_node_ids[self.accommodate_node_ids_index];
            let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(current_collapsable_node_id).unwrap();
            let current_collapsable_node = wrapped_current_collapsable_node.borrow();

            self.impacted_node_ids.insert(current_collapsable_node_id);

            // accommodate by making each parent try to move to a good next state
            for parent_neighbor_node_id in current_collapsable_node.parent_neighbor_node_ids.iter() {
                self.impacted_node_ids.insert(parent_neighbor_node_id);

                let wrapped_parent_neighbor_node = self.collapsable_node_per_id.get(parent_neighbor_node_id).unwrap();
                let mut parent_neighbor_node = wrapped_parent_neighbor_node.borrow_mut();
                let original_node_state = *parent_neighbor_node.node_state_indexed_view.get().unwrap();
                let mut current_node_state = original_node_state;
                let mut is_current_node_state_restrictive = true;
                while is_current_node_state_restrictive {
                    let is_current_mask_from_parent_restrictive: bool = if parent_neighbor_node.mask_per_neighbor_per_state.contains_key(&current_node_state) {
                        let mask_per_neighbor = parent_neighbor_node.mask_per_neighbor_per_state.get(&current_node_state).unwrap();
                        if let Some(mask) = mask_per_neighbor.get(current_collapsable_node_id) {
                            current_collapsable_node.is_mask_restrictive_to_current_state(mask)
                        }
                        else {
                            false
                        }
                    }
                    else {
                        false
                    };
                    if !is_current_mask_from_parent_restrictive {
                        debug!("found unrestricted mask (or no mask) for neighbor {:?}", parent_neighbor_node_id);
                        is_current_node_state_restrictive = false;  // leave the while loop for this parent neighbor node

                        if current_node_state != original_node_state {
                            debug!("the node state had to change to {:?}", current_node_state);

                            // store the changed node state
                            changed_parent_node_states.push(CollapsedNodeState {
                                node_id: String::from(*parent_neighbor_node_id),
                                node_state_id: Some(current_node_state.clone())
                            });
                            
                            to_node_state_and_from_node_state_tuple_per_parent_node_id.insert(parent_neighbor_node_id, (original_node_state, current_node_state));
                        }
                        else {
                            debug!("the node state was already good at {:?}", current_node_state);
                        }
                    }
                    else {
                        parent_neighbor_node.node_state_indexed_view.move_next();
                        let next_node_state = *parent_neighbor_node.node_state_indexed_view.get().unwrap();
                        if next_node_state == original_node_state {
                            // unable to accommodate the current collapsable node
                            debug!("Unable to accommodate the current collapsable node {:?} at state {:?}", current_collapsable_node_id, current_collapsable_node.node_state_indexed_view.get().unwrap());
                            break;
                        }
                        current_node_state = next_node_state;
                    }
                }
            }
        }

        // subtract original masks for altered neighbors and add new masks
        {
            for (parent_neighbor_node_id, (original_node_state, current_node_state)) in to_node_state_and_from_node_state_tuple_per_parent_node_id.iter() {
                let wrapped_parent_neighbor_node = self.collapsable_node_per_id.get(parent_neighbor_node_id).unwrap();
                let parent_neighbor_node = wrapped_parent_neighbor_node.borrow();
                
                // inform the impacted neighbors
                let neighbor_node_ids: &Vec<&str> = &parent_neighbor_node.neighbor_node_ids;
                let mask_per_neighbor_per_state: &HashMap<&TNodeState, HashMap<&str, BitVec>> = &parent_neighbor_node.mask_per_neighbor_per_state;
                if let Some(mask_per_neighbor) = mask_per_neighbor_per_state.get(original_node_state) {
                    for neighbor_node_id in neighbor_node_ids.iter() {
                        if mask_per_neighbor.contains_key(neighbor_node_id) {
                            let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                            let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                            //debug!("looking for mask from parent {:?} to child {:?}.", collapsable_node.id, neighbor_node_id);
                            //debug!("mask_per_neighbor: {:?}", mask_per_neighbor);
                            let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                            neighbor_collapsable_node.subtract_mask(mask);
                            debug!("subtracting mask to {:?} when in accommodate_current_node", neighbor_node_id);
                        }
                    }
                }
                if let Some(mask_per_neighbor) = mask_per_neighbor_per_state.get(current_node_state) {
                    for neighbor_node_id in neighbor_node_ids.iter() {
                        if mask_per_neighbor.contains_key(neighbor_node_id) {
                            let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                            let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                            //debug!("looking for mask from parent {:?} to child {:?}.", collapsable_node.id, neighbor_node_id);
                            //debug!("mask_per_neighbor: {:?}", mask_per_neighbor);
                            let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                            neighbor_collapsable_node.add_mask(mask);
                            debug!("adding mask to {:?} when in accommodate_current_node", neighbor_node_id);
                        }
                    }
                }
            }
        }

        self.accommodated_total += 1;

        changed_parent_node_states
    }
    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut node_state_per_node_id: HashMap<String, TNodeState> = HashMap::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            let node_state: TNodeState = (*collapsable_node.node_state_indexed_view.get().unwrap()).clone();
            let node_id: String = String::from(collapsable_node.id);
            debug!("established node {node_id} in state {:?}.", node_state);
            node_state_per_node_id.insert(node_id, node_state);
        }
        CollapsedWaveFunction {
            node_state_per_node_id
        }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for AccommodatingCollapsableWaveFunction<'a, TNodeState> {
    fn new(
        collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
        collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
        random_instance: Rc<RefCell<fastrand::Rng>>
    ) -> Self {
        AccommodatingCollapsableWaveFunction {
            collapsable_nodes,
            collapsable_node_per_id,
            accommodate_node_ids: Vec::new(),
            accommodate_node_ids_length: 0,
            accommodate_node_ids_index: 0,
            accommodated_total: 0,
            impacted_node_ids: HashSet::new(),
            random_instance,
            node_state_type: PhantomData
        }
    }
    fn collapse(&'a mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {
        let initialize_result = self.initialize_nodes();
        if initialize_result.is_err() {
            return Err(initialize_result.err().unwrap());
        }

        let mut iterations_total: u32 = 0;

        debug!("about to enter while loop");
        while !self.is_fully_collapsed() {
            debug!("preparing nodes for iteration");
            self.prepare_nodes_for_iteration();
            debug!("checking if done accommodating nodes");
            while !self.is_done_accommodating_nodes() {
                debug!("checking if current node is in conflict");
                if self.is_current_node_in_conflict() {
                    debug!("accommodating current node");
                    self.accommodate_current_node();
                }
                iterations_total += 1;
            }
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
        //      while pointer is inside the bounds
        //          if current collapsable node is in conflict and not already impacted
        //              accommodate this collapsable node
        //              alter mask for neighbors
        //              cache impacted nodes
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
            while !self.is_done_accommodating_nodes() {
                if self.is_current_node_in_conflict() {
                    let accommodated_neighbor_node_states = self.accommodate_current_node();
                    collapsed_node_states.extend(accommodated_neighbor_node_states);
                }
            }
        }

        Ok(collapsed_node_states)
    }
}