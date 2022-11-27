use std::collections::{VecDeque, HashSet};
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc, collections::HashMap};
use std::hash::Hash;
use bitvec::vec::BitVec;
use rand::seq::SliceRandom;
use super::collapsable_wave_function::{CollapsableWaveFunction, CollapsableNode, CollapsedNodeState, UncollapsedWaveFunction, CollapsedWaveFunction};

pub struct AccommodatingCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_nodes_length: usize,
    accommodate_node_ids: VecDeque<&'a str>,
    recently_accommodated_node_ids: VecDeque<&'a str>,
    recently_updated_neighbor_node_ids: VecDeque<&'a str>,
    accommodate_node_ids_index: usize,
    node_state_type: PhantomData<TNodeState>
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> AccommodatingCollapsableWaveFunction<'a, TNodeState> {
    fn initialize_nodes(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {

        // initialize each collapsable node to its first (random) state, storing them for the return
        // alter masks for every collapsable node to its neighbors
        // initialize the collapsable_nodes vector
        // put every node into the temp_recently_accommodated_nodes vector
        // initialize the temp_recently_updated_neighbors vector

        let mut initial_node_states: Vec<CollapsedNodeState<TNodeState>> = Vec::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let mut collapsable_node = wrapped_collapsable_node.borrow_mut();
            if !collapsable_node.node_state_indexed_view.try_move_next() {
                return Err(String::from("Cannot collapse wave function."));
            }
            
            self.recently_accommodated_node_ids.push_back(collapsable_node.id);
            let node_state = collapsable_node.node_state_indexed_view.get().unwrap();
            let collapsed_node_state: CollapsedNodeState<TNodeState> = CollapsedNodeState {
                node_id: String::from(collapsable_node.id),
                node_state_id: Some((*node_state).clone())
            };
            initial_node_states.push(collapsed_node_state);
        }

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

        self.recently_accommodated_node_ids.is_empty()
    }
    fn prepare_nodes_for_iteration(&mut self) {

        // put the recently_accommodated_nodes at the back of the collapsable_nodes vector
        // put the recently_updated_neighbors at the front of the collapsable_nodes vector
        // initialize pointer to first element of collapsable_nodes

        debug!("prior to being prepared: {:?}", self.accommodate_node_ids);

        while !self.recently_accommodated_node_ids.is_empty() {
            let node_id = self.recently_accommodated_node_ids.pop_front().unwrap();
            self.accommodate_node_ids.push_back(node_id);
        }

        while !self.recently_updated_neighbor_node_ids.is_empty() {
            let node_id = self.recently_updated_neighbor_node_ids.pop_back().unwrap();
            self.accommodate_node_ids.push_front(node_id);
        }

        self.accommodate_node_ids_index = 0;

        self.accommodate_node_ids.make_contiguous().shuffle(&mut rand::thread_rng());
     
        debug!("after being prepared: {:?}", self.accommodate_node_ids);
    }
    fn is_done_accommodating_nodes(&self) -> bool {

        // returns if pointer is outside the bounds of the collapsable_nodes

        self.accommodate_node_ids_index == self.accommodate_node_ids.len()
    }
    fn is_current_node_in_conflict(&mut self) -> bool {

        // returns if the current state of the current node is restricted
        // increment pointer if false

        let current_collapsable_node_id: &str = self.accommodate_node_ids[self.accommodate_node_ids_index];
        let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(current_collapsable_node_id).unwrap();
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        let is_current_collapsable_node_in_conflict = current_collapsable_node.node_state_indexed_view.is_current_state_restricted();

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
        // pop the neighbors out of the collapsable_nodes
        // push the neighbors into the back of recently_updated_neighbors
        // pop the current collapsable node out of collapsable_nodes
        // push the current collapsable node into the back of recently_accommodated_nodes

        let mut changed_parent_node_states: Vec<CollapsedNodeState<TNodeState>> = Vec::new();
        let mut to_node_state_and_from_node_state_tuple_per_parent_node_id: HashMap<&str, (&TNodeState, &TNodeState)> = HashMap::new();

        // try to get each parent neighbor node to accommodate the current node
        {
            let current_collapsable_node_id: &str = self.accommodate_node_ids[self.accommodate_node_ids_index];
            let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(current_collapsable_node_id).unwrap();
            let current_collapsable_node = wrapped_current_collapsable_node.borrow();

            let mut parent_neighbor_node_ids: HashSet<&str> = HashSet::new();

            // accommodate by making each parent try to move to a good next state
            for parent_neighbor_node_id in current_collapsable_node.parent_neighbor_node_ids.iter() {
                parent_neighbor_node_ids.insert(parent_neighbor_node_id);

                let wrapped_parent_neighbor_node = self.collapsable_node_per_id.get(parent_neighbor_node_id).unwrap();
                let mut parent_neighbor_node = wrapped_parent_neighbor_node.borrow_mut();
                let original_node_state = *parent_neighbor_node.node_state_indexed_view.get().unwrap();
                let mut current_node_state = original_node_state;
                let mut is_current_node_state_restrictive = true;
                while is_current_node_state_restrictive {
                    let is_current_mask_from_parent_restrictive: bool;
                    if parent_neighbor_node.mask_per_neighbor_per_state.contains_key(&current_node_state) {
                        let mask_per_neighbor = parent_neighbor_node.mask_per_neighbor_per_state.get(&current_node_state).unwrap();
                        let mask = mask_per_neighbor.get(current_collapsable_node_id).unwrap();
                        is_current_mask_from_parent_restrictive = current_collapsable_node.is_mask_restrictive_to_current_state(mask);
                    }
                    else {
                        is_current_mask_from_parent_restrictive = false;
                    }
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

            self.recently_updated_neighbor_node_ids.extend(&current_collapsable_node.parent_neighbor_node_ids);

            for index in (0..self.accommodate_node_ids.len()).rev() {
                let collapsable_node_id = self.accommodate_node_ids[index];
                if parent_neighbor_node_ids.contains(collapsable_node_id) {
                    debug!("found and removing parent node which just accommodated: {:?}", collapsable_node_id);
                    self.accommodate_node_ids.remove(index);
                    if index < self.accommodate_node_ids_index {
                        self.accommodate_node_ids_index -= 1;
                    }
                }
            }

            debug!("removing accommodated node: {:?}", current_collapsable_node_id);
            self.accommodate_node_ids.remove(self.accommodate_node_ids_index);
            self.recently_accommodated_node_ids.push_back(current_collapsable_node_id);

            debug!("current state of recently_updated_neighbor_node_ids: {:?}", self.recently_updated_neighbor_node_ids);
            debug!("current state of recently_accommodated_node_ids: {:?}", self.recently_accommodated_node_ids);
            debug!("current state of accommodate_node_ids: {:?}", self.accommodate_node_ids);
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

        changed_parent_node_states
    }
    #[time_graph::instrument]
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

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for AccommodatingCollapsableWaveFunction<'a, TNodeState> {
    #[time_graph::instrument]
    fn new(collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>, collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>) -> Self {
        let collapsable_nodes_length: usize = collapsable_nodes.len();
        AccommodatingCollapsableWaveFunction {
            collapsable_nodes: collapsable_nodes,
            collapsable_node_per_id: collapsable_node_per_id,
            collapsable_nodes_length: collapsable_nodes_length,
            accommodate_node_ids: VecDeque::new(),
            recently_accommodated_node_ids: VecDeque::new(),
            recently_updated_neighbor_node_ids: VecDeque::new(),
            accommodate_node_ids_index: 0,
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
                if iterations_total == 10 {
                    //panic!("stopping here for testing tt44");
                }
            }
        }
        debug!("fully collapsed after {:?} iterations", iterations_total);

        Ok(self.get_collapsed_wave_function())
    }
    fn collapse_into_steps(&'a mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {

        // initialize each collapsable node to its first (random) state
        // alter masks for every collapsable node to its neighbors
        // initialize the collapsable_nodes vector
        // put every node into the temp_recently_accommodated_nodes vector
        // initialize the temp_recently_updated_neighbors vector
        // while temp_recently_accommodated_nodes is not empty
        //      put the recently_accommodated_nodes at the back of the collapsable_nodes vector
        //      put the recently_updated_neighbors at the front of the collapsable_nodes vector
        //      initialize pointer to first element of collapsable_nodes
        //      while pointer is inside the bounds
        //          if current collapsable node is in conflict
        //              accommodate this collapsable node
        //              alter mask for neighbors
        //              pop the neighbors out of the collapsable_nodes
        //              push the neighbors into the back of recently_updated_neighbors
        //              pop the current collapsable node out of collapsable_nodes
        //              push the current collapsable node into the back of recently_accommodated_nodes
        //          // do not need to increment pointer since we just popped at the current index
        //          else
        //              increment pointer
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