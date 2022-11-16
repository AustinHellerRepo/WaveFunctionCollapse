use std::{default, collections::HashMap, slice::Iter};

use uuid::Uuid;
use wave_function_collapse::wave_function::{
    Node,
    NodeStateCollection,
    WaveFunction,
    CollapsedWaveFunction
};

#[derive(Eq, Hash, PartialEq)]
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
}

#[derive(Debug)]
enum NationalOrigin {
    English,
    Spain,
    Ukraine,
    Norway,
    Japan
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
    Left,
    Right,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Adjacent
}

struct Information {
    national_origin: Option<NationalOrigin>,
    house_color: Option<HouseColor>,
    cigarette_type: Option<CigaretteType>,
    pet: Option<Pet>,
    drink: Option<Drink>
}

impl Default for Information {
    fn default() -> Self {
        Information {
            national_origin: None,
            house_color: None,
            cigarette_type: None,
            pet: None,
            drink: None
        }
    }
}

impl Information {
    fn new_national_origin(national_origin: NationalOrigin) -> Self {
        Information {
            national_origin: Some(national_origin),
            ..Default::default()
        }
    }
    fn new_house_color(house_color: HouseColor) -> Self {
        Information {
            house_color: Some(house_color),
            ..Default::default()
        }
    }
    fn new_cigarette_type(cigarette_type: CigaretteType) -> Self {
        Information {
            cigarette_type: Some(cigarette_type),
            ..Default::default()
        }
    }
    fn new_pet(pet: Pet) -> Self {
        Information {
            pet: Some(pet),
            ..Default::default()
        }
    }
    fn new_drink(drink: Drink) -> Self {
        Information {
            drink: Some(drink),
            ..Default::default()
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
        let mut node_per_information_type_per_house_index: Vec<HashMap<InformationType, &Node>> = Vec::new();
        {
            /*
            for _ in 0..6 {
                for _ in InformationType::iter() {
                    let node_id: String = Uuid::new_v4().to_string();
                    let node = Node {
                        id: node_id.clone(),
                        node_state_collection_ids_per_neighbor_node_id: HashMap::new()
                    };
                    all_node_ids.push(node_id.clone());
                    nodes.push(node);
                }
            }

            {
                let mut node_index: usize = 0;
                while node_index < nodes.len() {
                    let mut node_per_information_type: HashMap<InformationType, &Node> = HashMap::new();
                    for information_type in InformationType::into_iter() {
                        node_per_information_type.insert(information_type, nodes.get(node_index).unwrap());
                        node_index += 1;
                    }
                    node_per_information_type_per_house_index.push(node_per_information_type);
                }
            }

            {
                // connect all nodes to each other
                for node in nodes.iter_mut() {
                    for node_id in all_node_ids.iter() {
                        if node.id != *node_id {
                            node.node_state_collection_ids_per_neighbor_node_id.insert(node_id.clone(), Vec::new());
                        }
                    }
                }
            }
            */
        }

        // iterate over each node with each other node to create node state collections per combination
        for from_house_index in 0..node_per_information_type_per_house_index.len() {
            for (from_information_type, from_node) in node_per_information_type_per_house_index[from_house_index].iter() {
                for to_house_index in 0..node_per_information_type_per_house_index.len() {
                    for (to_information_type, to_node) in node_per_information_type_per_house_index[to_house_index].iter() {

                    }
                }
            }
        }

        // TODO construct NodeStateCollection instances per scenario
        for dependency in self.dependencies.iter() {
            let from_information_type: InformationType;
            let to_information_type: Option<InformationType>;
            let proximity: &Proximity = &dependency.proximity;
            let from_node_state_id: String;
            let to_node_state_id: String;
            if let Some(national_origin) = &dependency.subject.national_origin {
                from_information_type = InformationType::NationalOrigin;
                from_node_state_id = national_origin.get_node_state_id();
            }
            else if let Some(house_color) = &dependency.subject.house_color {
                from_information_type = InformationType::HouseColor;
                from_node_state_id = house_color.get_node_state_id();
            }
            else if let Some(cigarette_type) = &dependency.subject.cigarette_type {
                from_information_type = InformationType::CigaretteType;
                from_node_state_id = cigarette_type.get_node_state_id();
            }
            else if let Some(pet) = &dependency.subject.pet {
                from_information_type = InformationType::Pet;
                from_node_state_id = pet.get_node_state_id();
            }
            else if let Some(drink) = &dependency.subject.drink {
                from_information_type = InformationType::Drink;
                from_node_state_id = drink.get_node_state_id();
            }
            else {
                panic!("Unexpected subject information type.");
            }

            if let Some(national_origin) = &dependency.subject.national_origin {
                to_information_type = Some(InformationType::NationalOrigin);
                to_node_state_id = national_origin.get_node_state_id();
            }
            else if let Some(house_color) = &dependency.subject.house_color {
                to_information_type = Some(InformationType::HouseColor);
                to_node_state_id = house_color.get_node_state_id();
            }
            else if let Some(cigarette_type) = &dependency.subject.cigarette_type {
                to_information_type = Some(InformationType::CigaretteType);
                to_node_state_id = cigarette_type.get_node_state_id();
            }
            else if let Some(pet) = &dependency.subject.pet {
                to_information_type = Some(InformationType::Pet);
                to_node_state_id = pet.get_node_state_id();
            }
            else if let Some(drink) = &dependency.subject.drink {
                to_information_type = Some(InformationType::Drink);
                to_node_state_id = drink.get_node_state_id();
            }
            else {
                panic!("Unexpected subject information type.");
            }

            match proximity {
                Proximity::First => {
                    if let Some(information_type) = &to_information_type {
                        panic!("Unexpected comparison. Must use relative proximity.");
                    }
                    else {
                        // only the from_node_state_id can exist in the first position

                    }
                },
                Proximity::Second => {
                    if let Some(information_type) = &to_information_type {
                        panic!("Unexpected comparison. Must use relative proximity.");
                    }
                    else {
                        // only the from_node_state_id can exist in the second position
                    }
                },
                Proximity::Third => {
                    if let Some(information_type) = &to_information_type {
                        panic!("Unexpected comparison. Must use relative proximity.");
                    }
                    else {
                        // only the from_node_state_id can exist in the third position
                    }
                },
                Proximity::Fourth => {
                    if let Some(information_type) = &to_information_type {
                        panic!("Unexpected comparison. Must use relative proximity.");
                    }
                    else {
                        // only the from_node_state_id can exist in the fourth position
                    }
                },
                Proximity::Fifth => {
                    if let Some(information_type) = &to_information_type {
                        panic!("Unexpected comparison. Must use relative proximity.");
                    }
                    else {
                        // only the from_node_state_id can exist in the fifth position
                    }
                },
                Proximity::Adjacent => {
                    if let Some(information_type) = &to_information_type {
                        // the from_node_state_id is next to the to_node_state_id

                        let mut from_to_node_state_collection: NodeStateCollection = NodeStateCollection {
                            id: Uuid::new_v4().to_string(),
                            node_state_id: from_node_state_id.clone(),
                            node_state_ids: vec![to_node_state_id.clone()]
                        };
                        let mut to_from_node_state_collection: NodeStateCollection = NodeStateCollection {
                            id: Uuid::new_v4().to_string(),
                            node_state_id: to_node_state_id.clone(),
                            node_state_ids: vec![from_node_state_id.clone()]
                        };

                        node_state_collections.push(from_to_node_state_collection);
                        node_state_collections.push(to_from_node_state_collection);
                    }
                    else {
                        panic!("Unexpected lack of comparison. Must specify a target.");
                    }
                },
                Proximity::Left => {
                    if let Some(information_type) = &to_information_type {
                        // the from_node_state_id is to the left of the to_node_state_id
                    }
                    else {
                        panic!("Unexpected lack of comparison. Must specify a target.");
                    }
                },
                Proximity::Right => {
                    if let Some(information_type) = &to_information_type {
                        // the from_node_state_id is to the right of the to_node_state_id
                    }
                    else {
                        panic!("Unexpected lack of comparison. Must specify a target.");
                    }
                },
                Proximity::Same => {
                    if let Some(information_type) = &to_information_type {
                        // the from_node_state_id is in the same house as the to_node_state_id
                    }
                    else {
                        panic!("Unexpected lack of comparison. Must specify a target.");
                    }
                }
            }
        }

        todo!()
    }
}

struct ZebraSolution {
    collapsed_wave_function: CollapsedWaveFunction
}

impl ZebraSolution {
    fn print(&self) {
        todo!()
    }
}

fn main() {

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
        Information::new_national_origin(NationalOrigin::English),
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
        Proximity::Right,
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
        Proximity::Third,
        None
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_national_origin(NationalOrigin::Norway),
        Proximity::First,
        None
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_cigarette_type(CigaretteType::Chesterfields),
        Proximity::Adjacent,
        Some(Information::new_pet(Pet::Fox))
    ));
    puzzle.insert_dependency(Dependency::new(
        Information::new_cigarette_type(CigaretteType::Kools),
        Proximity::Adjacent,
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
        Proximity::Adjacent,
        Some(Information::new_house_color(HouseColor::Blue))
    ));

    let wave_function: WaveFunction = puzzle.get_wave_function();
    wave_function.validate().unwrap();

    let solution_result = wave_function.collapse(None);

    if let Ok(solution) = solution_result {
        // TODO print the result
    }
    else {
        println!("Error: {}", solution_result.err().unwrap());
    }
}