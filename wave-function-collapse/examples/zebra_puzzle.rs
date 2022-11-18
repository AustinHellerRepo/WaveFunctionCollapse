use std::{default, collections::{HashMap, HashSet}, slice::Iter};
use log::debug;
extern crate pretty_env_logger;

use uuid::Uuid;
use wave_function_collapse::wave_function::{
    Node,
    NodeStateCollection,
    WaveFunction,
    CollapsedWaveFunction
};

#[derive(Eq, Hash, PartialEq, Debug)]
enum InformationType {
    NationalOrigin,
    HouseColor,
    CigaretteType,
    Pet,
    Drink
}

impl InformationType {
    fn iter() -> Iter<'static, InformationType> {
        [InformationType::NationalOrigin, InformationType::HouseColor, InformationType::CigaretteType, InformationType::Pet, InformationType::Drink].iter()
    }
    fn into_iter() -> std::array::IntoIter<InformationType, 5> {
        [InformationType::NationalOrigin, InformationType::HouseColor, InformationType::CigaretteType, InformationType::Pet, InformationType::Drink].into_iter()
    }
    fn get_node_state_ids(&self) -> Vec<String> {
        let mut node_state_ids: Vec<String> = Vec::new();
        match self {
            InformationType::NationalOrigin => {
                for national_origin in NationalOrigin::iter() {
                    node_state_ids.push(national_origin.to_string());
                }
            },
            InformationType::HouseColor => {
                for house_color in HouseColor::iter() {
                    node_state_ids.push(house_color.to_string());
                }
            },
            InformationType::CigaretteType => {
                for cigarette_type in CigaretteType::iter() {
                    node_state_ids.push(cigarette_type.to_string());
                }
            },
            InformationType::Pet => {
                for pet in Pet::iter() {
                    node_state_ids.push(pet.to_string());
                }
            },
            InformationType::Drink => {
                for drink in Drink::iter() {
                    node_state_ids.push(drink.to_string());
                }
            }
        }
        node_state_ids
    }
}

#[derive(Debug)]
enum NationalOrigin {
    England,
    Spain,
    Ukraine,
    Norway,
    Japan
}

impl NationalOrigin {
    fn iter() -> Iter<'static, NationalOrigin> {
        [NationalOrigin::England, NationalOrigin::Spain, NationalOrigin::Ukraine, NationalOrigin::Norway, NationalOrigin::Japan].iter()
    }
    fn into_iter() -> std::array::IntoIter<NationalOrigin, 5> {
        [NationalOrigin::England, NationalOrigin::Spain, NationalOrigin::Ukraine, NationalOrigin::Norway, NationalOrigin::Japan].into_iter()
    }
}

impl std::fmt::Display for NationalOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl NationalOrigin {
    fn get_node_state_id(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug)]
enum HouseColor {
    Red,
    Green,
    Ivory,
    Yellow,
    Blue
}

impl HouseColor {
    fn iter() -> Iter<'static, HouseColor> {
        [HouseColor::Red, HouseColor::Green, HouseColor::Ivory, HouseColor::Yellow, HouseColor::Blue].iter()
    }
    fn into_iter() -> std::array::IntoIter<HouseColor, 5> {
        [HouseColor::Red, HouseColor::Green, HouseColor::Ivory, HouseColor::Yellow, HouseColor::Blue].into_iter()
    }
}

impl std::fmt::Display for HouseColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HouseColor {
    fn get_node_state_id(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug)]
enum CigaretteType {
    OldGold,
    Kools,
    Chesterfields,
    LuckyStrike,
    Parliaments
}

impl CigaretteType {
    fn iter() -> Iter<'static, CigaretteType> {
        [CigaretteType::OldGold, CigaretteType::Kools, CigaretteType::Chesterfields, CigaretteType::LuckyStrike, CigaretteType::Parliaments].iter()
    }
    fn into_iter() -> std::array::IntoIter<CigaretteType, 5> {
        [CigaretteType::OldGold, CigaretteType::Kools, CigaretteType::Chesterfields, CigaretteType::LuckyStrike, CigaretteType::Parliaments].into_iter()
    }
}

impl std::fmt::Display for CigaretteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl CigaretteType {
    fn get_node_state_id(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug)]
enum Pet {
    Dog,
    Snails,
    Fox,
    Horse,
    Zebra
}

impl Pet {
    fn iter() -> Iter<'static, Pet> {
        [Pet::Dog, Pet::Snails, Pet::Fox, Pet::Horse, Pet::Zebra].iter()
    }
    fn into_iter() -> std::array::IntoIter<Pet, 5> {
        [Pet::Dog, Pet::Snails, Pet::Fox, Pet::Horse, Pet::Zebra].into_iter()
    }
}

impl std::fmt::Display for Pet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Pet {
    fn get_node_state_id(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug)]
enum Drink {
    Coffee,
    Tea,
    Milk,
    OrangeJuice,
    Water
}

impl Drink {
    fn iter() -> Iter<'static, Drink> {
        [Drink::Coffee, Drink::Tea, Drink::Milk, Drink::OrangeJuice, Drink::Water].iter()
    }
    fn into_iter() -> std::array::IntoIter<Drink, 5> {
        [Drink::Coffee, Drink::Tea, Drink::Milk, Drink::OrangeJuice, Drink::Water].into_iter()
    }
}

impl std::fmt::Display for Drink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drink {
    fn get_node_state_id(&self) -> String {
        self.to_string()
    }
}

// TODO create structure for containing the logic

enum Proximity {
    Same,
    NotSame,
    ImmediateLeft,
    RelativeLeft,
    ImmediateRight,
    RelativeRight,
    Index(usize),
    ImmediateAdjacent
}

struct Information {
    national_origin: Option<NationalOrigin>,
    house_color: Option<HouseColor>,
    cigarette_type: Option<CigaretteType>,
    pet: Option<Pet>,
    drink: Option<Drink>,
    information_type: Option<InformationType>
}

impl Default for Information {
    fn default() -> Self {
        Information {
            national_origin: None,
            house_color: None,
            cigarette_type: None,
            pet: None,
            drink: None,
            information_type: None
        }
    }
}

impl Information {
    fn new_national_origin(national_origin: NationalOrigin) -> Self {
        Information {
            national_origin: Some(national_origin),
            information_type: Some(InformationType::NationalOrigin),
            ..Default::default()
        }
    }
    fn new_house_color(house_color: HouseColor) -> Self {
        Information {
            house_color: Some(house_color),
            information_type: Some(InformationType::HouseColor),
            ..Default::default()
        }
    }
    fn new_cigarette_type(cigarette_type: CigaretteType) -> Self {
        Information {
            cigarette_type: Some(cigarette_type),
            information_type: Some(InformationType::CigaretteType),
            ..Default::default()
        }
    }
    fn new_pet(pet: Pet) -> Self {
        Information {
            pet: Some(pet),
            information_type: Some(InformationType::Pet),
            ..Default::default()
        }
    }
    fn new_drink(drink: Drink) -> Self {
        Information {
            drink: Some(drink),
            information_type: Some(InformationType::Drink),
            ..Default::default()
        }
    }
    fn get_node_state_id(&self) -> String {
        match self.information_type.as_ref().unwrap() {
            InformationType::NationalOrigin => {
                self.national_origin.as_ref().unwrap().get_node_state_id()
            },
            InformationType::HouseColor => {
                self.house_color.as_ref().unwrap().get_node_state_id()
            },
            InformationType::CigaretteType => {
                self.cigarette_type.as_ref().unwrap().get_node_state_id()
            },
            InformationType::Pet => {
                self.pet.as_ref().unwrap().get_node_state_id()
            },
            InformationType::Drink => {
                self.drink.as_ref().unwrap().get_node_state_id()
            }
        }
    }
}

struct Dependency {
    subject: Information,
    proximity: Proximity,
    target: Option<Information>
}

impl Dependency {
    fn new(subject: Information, proximity: Proximity, target: Option<Information>) -> Self {
        Dependency {
            subject: subject,
            proximity: proximity,
            target: target
        }
    }
    fn is_static(&self) -> bool {
        self.target.is_none()
    }
    fn is_staticly_applicable(&self, from_house_index: usize, from_information_type: &InformationType) -> bool {
        if self.subject.information_type.as_ref().unwrap() == from_information_type {
            if self.target.is_none() {
                match self.proximity {
                    Proximity::Index(index) => {
                        if from_house_index == index {
                            return true;
                        }
                    },
                    Proximity::ImmediateLeft => {
                        panic!("Cannot use ImmediateLeft when a target is not specified.");
                    },
                    Proximity::RelativeLeft => {
                        panic!("Cannot use RelativeLeft when a target is not specified.");
                    },
                    Proximity::Same => {
                        panic!("Cannot use Same when a target is not specified.");
                    },
                    Proximity::RelativeRight => {
                        panic!("Cannot use RelativeRight when a target is not specified.");
                    },
                    Proximity::ImmediateRight => {
                        panic!("Cannot use ImmediateRight when a target is not specified.");
                    }
                    Proximity::NotSame => {
                        panic!("Cannot use NotSame when a target is not specified.");
                    }
                    Proximity::ImmediateAdjacent => {
                        panic!("Cannot use ImmediateAdjacent when a target is not specified.");
                    }
                }
            }
        }
        false
    }
    fn is_cross_domain(&self) -> bool {
        if let Some(target) = &self.target {
            return self.subject.information_type.as_ref().unwrap() != target.information_type.as_ref().unwrap()
        }
        false
    }
    fn is_relatively_applicable(&self, from_house_index: usize, from_information_type: &InformationType, to_house_index: usize, to_information_type: &InformationType) -> bool {
        if self.subject.information_type.as_ref().unwrap() == from_information_type {
            if let Some(target) = &self.target {
                if target.information_type.as_ref().unwrap() == to_information_type {
                    // the subject and target match the provided types

                    match self.proximity {
                        Proximity::Index(index) => {
                            panic!("Cannot use Index when a target is specified.");
                        },
                        Proximity::ImmediateLeft => {
                            // when the from_house_index is to the immediate left of to_house_index
                            if from_house_index + 1 == to_house_index {
                                return true;
                            }
                        },
                        Proximity::RelativeLeft => {
                            // when the from_house_index is somewhere to the left of to_house_index
                            // TODO do not force the target but instead restrict all other house indexes
                            todo!("need to add another 'is' method for 'is target restricted' and maybe only use that");
                            if from_house_index < to_house_index {
                                return true;
                            }
                        },
                        Proximity::Same => {
                            if from_house_index == to_house_index {
                                return true;
                            }
                        },
                        Proximity::RelativeRight => {
                            todo!("need to add another 'is' method for 'is target restricted' and maybe only use that");
                            if from_house_index > to_house_index {
                                return true;
                            }
                        },
                        Proximity::ImmediateRight => {
                            if from_house_index == to_house_index + 1 {
                                return true;
                            }
                        }
                        Proximity::NotSame => {
                            todo!("need to add another 'is' method for 'is target restricted' and maybe only use that");
                            if from_house_index != to_house_index {
                                return true;
                            }
                        }
                        Proximity::ImmediateAdjacent => {
                            if from_house_index.abs_diff(to_house_index) == 1 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct NodeStateCollectionKey {
    from_node_state_id: String,
    to_node_state_ids: Vec<String>
}

struct ZebraPuzzle {
    dependencies: Vec<Dependency>
}

impl ZebraPuzzle {
    fn new() -> Self {
        ZebraPuzzle {
            dependencies: Vec::new()
        }
    }
    fn insert_dependency(&mut self, dependency: Dependency) {
        self.dependencies.push(dependency);
    }
    fn get_wave_function(&self) -> WaveFunction {
        
        let mut nodes: Vec<Node> = Vec::new();
        let mut node_state_collections: Vec<NodeStateCollection> = Vec::new();

        let mut all_node_ids: Vec<String> = Vec::new();
        let mut node_id_per_information_type_per_house_index: Vec<HashMap<InformationType, &str>> = Vec::new();
        {
            // create all node ids
            for house_index in 0..5 {
                for information_type in InformationType::iter() {
                    let node_id: String = format!("{}_{:?}_{}", house_index, information_type, Uuid::new_v4().to_string());
                    all_node_ids.push(node_id);
                }
            }

            // make the nodes more accessible, based on its column index and information type
            {
                let mut node_index: usize = 0;
                while node_index < all_node_ids.len() {
                    let mut node_id_per_information_type: HashMap<InformationType, &str> = HashMap::new();
                    for information_type in InformationType::into_iter() {
                        node_id_per_information_type.insert(information_type, all_node_ids.get(node_index).unwrap());
                        node_index += 1;
                    }
                    node_id_per_information_type_per_house_index.push(node_id_per_information_type);
                }
            }
        }

        // cache NodeStateCollection instances for each information type
        let mut not_same_node_state_collection_id_per_node_state_id_per_information_type: HashMap<&InformationType, HashMap<String, String>> = HashMap::new();
        {
            for information_type in InformationType::iter() {
                let mut information_type_node_state_collection_ids: HashMap<String, String> = HashMap::new();
                let information_type_node_state_ids = information_type.get_node_state_ids();
                for chosen_node_state_id in information_type_node_state_ids.iter() {
                    let mut permitted_node_state_ids: Vec<String> = Vec::new();
                    for permitted_node_state_id in information_type.get_node_state_ids().iter() {
                        if chosen_node_state_id != permitted_node_state_id {
                            permitted_node_state_ids.push(permitted_node_state_id.clone());
                        }
                    }
                    let node_state_collection_id: String = Uuid::new_v4().to_string();
                    let node_state_collection = NodeStateCollection {
                        id: node_state_collection_id.clone(),
                        node_state_id: chosen_node_state_id.clone(),
                        node_state_ids: permitted_node_state_ids
                    };
                    node_state_collections.push(node_state_collection);
                    information_type_node_state_collection_ids.insert(chosen_node_state_id.clone(), node_state_collection_id);
                }
                not_same_node_state_collection_id_per_node_state_id_per_information_type.insert(information_type, information_type_node_state_collection_ids);
            }
        }

        // iterate over each node with each other node to create node state collections per combination
        let mut permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id: HashMap<&str, HashMap<&str, HashMap<String, Vec<String>>>> = HashMap::new();
        let mut restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id: HashMap<&str, HashMap<&str, HashMap<String, Vec<String>>>> = HashMap::new();

        let mut existing_node_state_id_per_information_type_per_house_index: HashMap<usize, HashMap<&InformationType, String>> = HashMap::new();
        {
            for from_house_index in 0..node_id_per_information_type_per_house_index.len() {
                for (from_information_type, from_node_id) in node_id_per_information_type_per_house_index[from_house_index].iter() {
                    let from_node_id: &str = from_node_id;
                    for dependency in self.dependencies.iter() {
                        if dependency.is_static() {
                            if dependency.is_staticly_applicable(from_house_index, from_information_type) {
                                // the nth house for this specific information type is this subject value
                                let node_state_id: String = dependency.subject.get_node_state_id();

                                if !existing_node_state_id_per_information_type_per_house_index.contains_key(&from_house_index) {
                                    let node_state_id_per_information_type: HashMap<&InformationType, String> = HashMap::new();
                                    existing_node_state_id_per_information_type_per_house_index.insert(from_house_index, node_state_id_per_information_type);
                                }
                                existing_node_state_id_per_information_type_per_house_index.get_mut(&from_house_index).unwrap().insert(from_information_type, node_state_id);
                            }
                        }
                    }

                    for to_house_index in 0..node_id_per_information_type_per_house_index.len() {
                        for (to_information_type, to_node_id) in node_id_per_information_type_per_house_index[to_house_index].iter() {
                            let to_node_id: &str = to_node_id;
                            if from_node_id != to_node_id {
                                for dependency in self.dependencies.iter() {
                                    if !dependency.is_static() {
                                        if !dependency.is_cross_domain() && !dependency.is_relatively_applicable(from_house_index, from_information_type, to_house_index, to_information_type) {
                                            // if the information types are the same, then any applicable dependencies should be restrictive

                                            let from_node_state_id: String = dependency.subject.get_node_state_id();
                                            let to_node_state_id: String = dependency.target.as_ref().unwrap().get_node_state_id();

                                            // this dependency does not exist between these two nodes and should therefore be restricted
                                            if !restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.contains_key(from_node_id) {
                                                restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.insert(from_node_id, HashMap::new());
                                            }
                                            if !restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(from_node_id).unwrap().contains_key(to_node_id) {
                                                restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get_mut(from_node_id).unwrap().insert(to_node_id, HashMap::new());
                                            }
                                            if !restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(from_node_id).unwrap().get(to_node_id).unwrap().contains_key(&from_node_state_id) {
                                                restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get_mut(from_node_id).unwrap().get_mut(to_node_id).unwrap().insert(from_node_state_id.clone(), Vec::new());
                                            }
                                            restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get_mut(from_node_id).unwrap().get_mut(to_node_id).unwrap().get_mut(&from_node_state_id).unwrap().push(to_node_state_id);
                                        }
                                        else if dependency.is_cross_domain() && dependency.is_relatively_applicable(from_house_index, from_information_type, to_house_index, to_information_type) {
                                            // else if the information types are different, then node state collection is purely additive
                                            
                                            let from_node_state_id: String = dependency.subject.get_node_state_id();
                                            let to_node_state_id: String = dependency.target.as_ref().unwrap().get_node_state_id();

                                            if !permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.contains_key(from_node_id) {
                                                permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.insert(from_node_id, HashMap::new());
                                            }
                                            if !permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(from_node_id).unwrap().contains_key(to_node_id) {
                                                permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get_mut(from_node_id).unwrap().insert(to_node_id, HashMap::new());
                                            }
                                            if !permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(from_node_id).unwrap().get(to_node_id).unwrap().contains_key(&from_node_state_id) {
                                                permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get_mut(from_node_id).unwrap().get_mut(to_node_id).unwrap().insert(from_node_state_id.clone(), Vec::new());
                                            }
                                            permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get_mut(from_node_id).unwrap().get_mut(to_node_id).unwrap().get_mut(&from_node_state_id).unwrap().push(to_node_state_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // create nodes, using either all possible node states or the specific previously determined state
        // set the possible node states per node given its information type
        let mut node_state_collection_id_per_node_state_collection_key: HashMap<NodeStateCollectionKey, String> = HashMap::new();
        let mut node_index: usize = 0;
        for house_index in 0..5 as usize {
            for information_type in InformationType::iter() {
                let node_id: &str = all_node_ids.get(node_index).unwrap();
                let mut node_state_ids: Vec<String> = Vec::new();
                if existing_node_state_id_per_information_type_per_house_index.contains_key(&house_index) && existing_node_state_id_per_information_type_per_house_index.get(&house_index).unwrap().contains_key(information_type) {
                    let node_state_id = existing_node_state_id_per_information_type_per_house_index.get(&house_index).unwrap().get(information_type).unwrap();
                    node_state_ids.push(node_state_id.clone());
                }
                else {
                    node_state_ids.extend(information_type.get_node_state_ids());
                }

                // tie this node to all other neighbors of the same information type
                let mut node_state_collection_ids_per_neighbor_node_id: HashMap<String, Vec<String>> = HashMap::new();
                let mut neighbor_node_index: usize = 0;
                for neighbor_house_index in 0..5 as usize {
                    for neighbor_information_type in InformationType::iter() {
                        let neighbor_node_id: &str = all_node_ids.get(neighbor_node_index).unwrap();
                        if node_index != neighbor_node_index {
                            if information_type == neighbor_information_type {
                                for from_node_state_id in information_type.get_node_state_ids().into_iter() {
                                    let mut permitted_node_state_ids = information_type.get_node_state_ids();
                                    if restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.contains_key(node_id) &&
                                        restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(node_id).unwrap().contains_key(neighbor_node_id) &&
                                        restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(node_id).unwrap().get(neighbor_node_id).unwrap().contains_key(&from_node_state_id) {

                                        let restricted_node_state_ids = restricted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(node_id).unwrap().get(neighbor_node_id).unwrap().get(&from_node_state_id).unwrap();
                                        permitted_node_state_ids.retain(|node_state_id| !restricted_node_state_ids.contains(node_state_id));
                                    }
                                    permitted_node_state_ids.retain(|node_state_id| node_state_id != &from_node_state_id);
                                    let node_state_collection_key = NodeStateCollectionKey {
                                        from_node_state_id: from_node_state_id,
                                        to_node_state_ids: permitted_node_state_ids
                                    };
                                    debug!("connection {house_index} {:?} to {neighbor_house_index} {:?} via {:?}.", information_type, neighbor_information_type, node_state_collection_key);
                                    if !node_state_collection_id_per_node_state_collection_key.contains_key(&node_state_collection_key) {
                                        let cloned_node_state_collection_key = node_state_collection_key.clone();
                                        let node_state_collection_id: String = Uuid::new_v4().to_string();
                                        let node_state_collection = NodeStateCollection {
                                            id: node_state_collection_id.clone(),
                                            node_state_id: cloned_node_state_collection_key.from_node_state_id.clone(),
                                            node_state_ids: cloned_node_state_collection_key.to_node_state_ids.clone()
                                        };
                                        node_state_collections.push(node_state_collection);
                                        node_state_collection_id_per_node_state_collection_key.insert(cloned_node_state_collection_key, node_state_collection_id);
                                    }
                                    if !node_state_collection_ids_per_neighbor_node_id.contains_key(neighbor_node_id) {
                                        node_state_collection_ids_per_neighbor_node_id.insert(String::from(neighbor_node_id), Vec::new());
                                    }
                                    let node_state_collection_id: String = node_state_collection_id_per_node_state_collection_key.get(&node_state_collection_key).unwrap().clone();
                                    node_state_collection_ids_per_neighbor_node_id.get_mut(neighbor_node_id).unwrap().push(node_state_collection_id);
                                }
                            }
                            else {
                                if permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.contains_key(node_id) &&
                                    permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(node_id).unwrap().contains_key(neighbor_node_id) {

                                    for (from_node_state_id, to_node_state_ids) in permitted_to_node_state_ids_per_from_node_state_id_per_to_node_id_per_from_node_id.get(node_id).unwrap().get(neighbor_node_id).unwrap().iter() {
                                        let node_state_collection_key = NodeStateCollectionKey {
                                            from_node_state_id: from_node_state_id.clone(),
                                            to_node_state_ids: to_node_state_ids.clone()
                                        };
                                        if !node_state_collection_id_per_node_state_collection_key.contains_key(&node_state_collection_key) {
                                            let cloned_node_state_collection_key = node_state_collection_key.clone();
                                            let node_state_collection_id: String = Uuid::new_v4().to_string();
                                            let node_state_collection = NodeStateCollection {
                                                id: node_state_collection_id.clone(),
                                                node_state_id: cloned_node_state_collection_key.from_node_state_id.clone(),
                                                node_state_ids: cloned_node_state_collection_key.to_node_state_ids.clone()
                                            };
                                            node_state_collections.push(node_state_collection);
                                            node_state_collection_id_per_node_state_collection_key.insert(cloned_node_state_collection_key, node_state_collection_id);
                                        }
                                        if !node_state_collection_ids_per_neighbor_node_id.contains_key(neighbor_node_id) {
                                            node_state_collection_ids_per_neighbor_node_id.insert(String::from(neighbor_node_id), Vec::new());
                                        }
                                        let node_state_collection_id: String = node_state_collection_id_per_node_state_collection_key.get(&node_state_collection_key).unwrap().clone();
                                        node_state_collection_ids_per_neighbor_node_id.get_mut(neighbor_node_id).unwrap().push(node_state_collection_id);
                                    }
                                }
                            }
                        }
                        neighbor_node_index += 1;
                    }
                }

                let node = Node {
                    id: String::from(node_id),
                    node_state_ids: node_state_ids,
                    node_state_collection_ids_per_neighbor_node_id: node_state_collection_ids_per_neighbor_node_id
                };
                nodes.push(node);
                node_index += 1;
            }
        }

        WaveFunction::new(nodes, node_state_collections)
    }
}

struct ZebraSolution {
    collapsed_wave_function: CollapsedWaveFunction
}

impl ZebraSolution {
    fn print(&self) {
        let mut node_state_per_column_per_row: Vec<Vec<Option<String>>> = Vec::new();
        for information_type in InformationType::iter() {
            let mut node_state_per_column: Vec<Option<String>> = Vec::new();
            for house_index in 0..5 {
                node_state_per_column.push(None);
            }
            node_state_per_column_per_row.push(node_state_per_column);
        }

        for (node, node_state) in self.collapsed_wave_function.node_state_per_node.iter() {
            let node_split = node.split("_").collect::<Vec<&str>>();
            let collapsed_node_house_index: usize = node_split[0].parse::<u8>().unwrap() as usize;
            let collapsed_node_information_type: &str = node_split[1];

            for (information_type_index, information_type) in InformationType::iter().enumerate() {
                if format!("{:?}", information_type) == collapsed_node_information_type {
                    node_state_per_column_per_row[information_type_index][collapsed_node_house_index] = Some(node_state.clone());
                }
            }
        }

        println!("----------------------");
        for row in node_state_per_column_per_row.into_iter() {
            print!("|");
            for column in row.into_iter() {
                print!("{}|", column.unwrap());
            }
            println!("");
        }
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    // There are five houses.
    // The Englishman lives in the red house.
    // The Spaniard owns the dog.
    // Coffee is drunk in the green house.
    // The Ukrainian drinks tea.
    // The green house is immediately to the right of the ivory house.
    // The Old Gold smoker owns snails.
    // Kools are smoked in the yellow house.
    // Milk is drunk in the middle house.
    // The Norwegian lives in the first house.
    // The man who smokes Chesterfields lives in the house next to the man with the fox.
    // Kools are smoked in the house next to the house where the horse is kept.
    // The Lucky Strike smoker drinks orange juice.
    // The Japanese smokes Parliaments.
    // The Norwegian lives next to the blue house.

    let mut puzzle = ZebraPuzzle::new();
    puzzle.insert_dependency(Dependency::new(
        Information::new_national_origin(NationalOrigin::England),
        Proximity::Same,
        Some(Information::new_house_color(HouseColor::Red))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_national_origin(NationalOrigin::Spain),
        Proximity::Same,
        Some(Information::new_pet(Pet::Dog))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_drink(Drink::Coffee),
        Proximity::Same,
        Some(Information::new_house_color(HouseColor::Green))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_national_origin(NationalOrigin::Ukraine),
        Proximity::Same,
        Some(Information::new_drink(Drink::Tea))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_house_color(HouseColor::Green),
        Proximity::ImmediateRight,
        Some(Information::new_house_color(HouseColor::Ivory))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_cigarette_type(CigaretteType::OldGold),
        Proximity::Same,
        Some(Information::new_pet(Pet::Snails))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_cigarette_type(CigaretteType::Kools),
        Proximity::Same,
        Some(Information::new_house_color(HouseColor::Yellow))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_drink(Drink::Milk),
        Proximity::Index(2),
        None
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_national_origin(NationalOrigin::Norway),
        Proximity::Index(0),
        None
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_cigarette_type(CigaretteType::Chesterfields),
        Proximity::ImmediateAdjacent,
        Some(Information::new_pet(Pet::Fox))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_cigarette_type(CigaretteType::Kools),
        Proximity::ImmediateAdjacent,
        Some(Information::new_pet(Pet::Horse))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_cigarette_type(CigaretteType::LuckyStrike),
        Proximity::Same,
        Some(Information::new_drink(Drink::OrangeJuice))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_national_origin(NationalOrigin::Japan),
        Proximity::Same,
        Some(Information::new_cigarette_type(CigaretteType::Parliaments))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_national_origin(NationalOrigin::Norway),
        Proximity::ImmediateAdjacent,
        Some(Information::new_house_color(HouseColor::Blue))
    ));

    let wave_function: WaveFunction = puzzle.get_wave_function();
    wave_function.validate().unwrap();
    return;

    let solution_result = wave_function.collapse(None);

    if let Ok(solution) = solution_result {
        // TODO print the result
        for (node, node_state) in solution.node_state_per_node.iter() {
            println!("{} is {}", node, node_state);
        }

        let zebra_solution = ZebraSolution {
            collapsed_wave_function: solution
        };
        zebra_solution.print();
    }
    else {
        println!("Error: {}", solution_result.err().unwrap());
    }
}