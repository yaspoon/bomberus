use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Display;
use std::any::{TypeId, Any};

struct Position {
	x: f64,
	y: f64,
}

impl Position {
	pub fn new(x: f64, y: f64) -> Position {
		return Position {x, y};
	}
}

impl Component for Position {
	fn get_type(&self) -> TypeId {
		//return ComponentType::Position;
		return TypeId::of::<Position>();
	}

	fn as_any(&self) -> &dyn Any {
		return self;
	}
}

impl Display for Position {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		return write!(f, "x:{}, y:{}", self.x, self.y);
	}
}

struct Moveable {
	dx: f64,
	dy: f64,
}

impl Moveable {
	pub fn new(dx: f64, dy: f64) -> Moveable {
		return Moveable {dx, dy};
	}
}

impl Component for Moveable {
	fn get_type(&self) -> TypeId {
		return TypeId::of::<Moveable>();
	}

	fn as_any(&self) -> &dyn Any {
		return self;
	}
}

impl fmt::Display for Moveable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		return write!(f, "dx:{}, dy:{}", self.dx, self.dy);
	}
}

struct Collidable {
	width: f64,
	height: f64,
}

struct Drawable {
	//Image stuff here
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum ComponentType {
	Position = 0x0,
	Moveable,
	Collidable,
	Drawable,	
	Count,
}

impl Display for ComponentType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match *self {
			ComponentType::Position => "Position",
			_ => panic!("Unknown component type"),
		};
		return write!(f, "type:{}", name);
	}
}

pub trait Component: fmt::Display {
	//fn get_type(&self) -> ComponentType;
	fn get_type(&self) -> TypeId;

	fn as_any(&self) -> &dyn Any;
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Entity {
	id: u64,
}

impl Display for Entity {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		return write!(f, "id:{}", self.id);
	}
}

struct EntitySystem {
	next_id: u64,
	entities: Vec<Entity>,
	entity_names: HashMap<u64, String>,
	components: HashMap<TypeId, HashMap<u64, Box<dyn Component>>>,
}

impl EntitySystem {
	pub fn new() -> EntitySystem {
		let mut components = HashMap::new();
		/*
		components.insert(ComponentType::Position, HashMap::new());
		components.insert(ComponentType::Moveable, HashMap::new());
		components.insert(ComponentType::Collidable, HashMap::new());
		components.insert(ComponentType::Drawable, HashMap::new());
		*/
		components.insert(TypeId::of::<Position>(), HashMap::new());

		/*
		if components.len() != ComponentType::Count as usize {
			panic!("Missing component types from store count in store {} expected {}", components.len(), ComponentType::Count as u64);
		}
		*/
		return EntitySystem {next_id: 0, entities: Vec::new(), entity_names: HashMap::new(), components};
	}

	pub fn new_entity(&mut self) -> Result<Entity, String> {
		return self.new_entity_with_name("Unknown".to_string());
	}

	pub fn new_entity_with_name(&mut self, name: String) -> Result<Entity, String> {
		if self.next_id == u64::MAX {
			panic!("Overflowing next_id, this should be fixed...");	
		}

		let ent = Entity {id: self.next_id};

		self.next_id += 1;
		self.entities.push(ent);
		self.entity_names.insert(ent.id, name);

		return Ok(ent);
	}

	pub fn remove_entity(&mut self, ent: Entity) {
		let mut found = false;
		let mut index = 0;

		for (i, e) in self.entities.iter().enumerate() {
			if *e == ent {
				index = i;
				found = true;
			}
		}

		if found {
			self.entities.remove(index);
		} else {
			panic!("Failed to find entity for removal:{:?}", ent);
		}
	}

	pub fn get_component_for_entity(&mut self, ent: Entity, component_type: TypeId) -> Result<&Box<dyn Component>, String> {
		let store = match self.components.get(&component_type) {
			Some(s) => s,
			None => return Err("No such component type in store".to_string()),
		};	

		match store.get(&ent.id) {
			Some(c) => return Ok(c),
			None => return Err(format!("No component for Entity {}", ent)),
		};
	}

	pub fn get_all_components_of_type(&self, component_type: TypeId) -> Result<Vec<(&u64, &Box<dyn Component>)>, String> {
		let store = match self.components.get(&component_type) {
			Some(s) => s,
			None => return Err(format!("Unknown component type in store")),
		};

		return Ok(store.iter().collect());
	}

	pub fn get_all_components_of_type_for_ids_mut(&self, ids: HashSet<u64>, component_type: TypeId) -> Result<Vec<(&u64, &Box<dyn Component>)>, String> {
		let store = match self.components.get(&component_type) {
			Some(s) => s,
			None => return Err(format!("Unknown component type in store")),
		};

		return Ok(store.iter().filter(|(id, comp)| ids.contains(id)).collect());
	}

	pub fn add_component(&mut self, ent: Entity, comp: Box<dyn Component>) -> Result<(), String> {
		if !self.components.contains_key(&comp.get_type()) {
			println!("Store doesn't contain component, creating");
			self.components.insert(comp.get_type(), HashMap::new());
		}

		let mut store = match self.components.get_mut(&comp.get_type()) {
			Some(s) => s,
			None => return Err(format!("Unknown component in store")),
		};

		store.insert(ent.id, comp);

		return Ok(());
	}

	pub fn remove_component(&mut self, ent: Entity, comp: TypeId) -> Result<(), String> {
		let mut store = match self.components.get_mut(&comp) {
			Some(s) => s,
			None => return Err(format!("Unknown component type for store")),
		};

		match store.remove(&ent.id) {
			Some(v) => return Ok(()),
			None => return Err(format!("No such component for entity {}", ent)),
		}
	}
}

fn System_Moveable(es: &mut EntitySystem, dt: f64) -> Result<(), String> {
	let moveables = match es.get_all_components_of_type(TypeId::of::<Moveable>()) {
		Ok(m) => m,
		Err(e) => return Err(e),
	};

	let entity_ids: HashSet<u64> = moveables.iter().map(|(id, comp)| **id).collect();

	let positions = match es.get_all_components_of_type_for_ids_mut(entity_ids, TypeId::of::<Position>()) {
		Ok(p) => p,
		Err(e) => return Err(e),
	};

	if positions.len() != moveables.len() {
		return Err(format!("Positions.len() == {} Moveables.len() == {}", positions.len(), moveables.len()));
	}

	//TODO:Trying to get this working
	for mp in moveables.into_iter().zip(positions.into_iter()) {
		match mp {
			((mid, mcomponent), (pid, pcomponent)) => {
				println!("Moveable {} data {} Position {} data {}", mid, mcomponent, pid, pcomponent);
				if mid != pid {
					return Err(format!("mid and pid don't match mid:{} pid:{}", mid, pid));
				}

				let moveable: &mut Moveable = match mcomponent.as_any().downcast_mut::<Moveable>() {
					Some(m) => m,
					None => return Err(format!("Failed to downcast mcomponent to moveable")),
				};

				let position: &mut Position = match pcomponent.as_any().downcast_mut::<Position>() {
					Some(p) => p,
					None => return Err(format!("Failed to downcast pcomponent to position")),
				};

				position.x += moveable.dx * dt;
				position.y += moveable.dy * dt;

				
			},
			_ => println!("Failed to match, expected 2 component tuple"),
		}
	}

	return Ok(());
}

fn main() {
	let mut es = EntitySystem::new();

	let player = match es.new_entity_with_name("Player".to_string()) {
		Ok(p) => {
			println!("player:{}", p);
			p
		},
		Err(e) => panic!("Failed to create player:{}", e),
	};

	match es.add_component(player, Box::new(Position::new(0.0, 0.0))) {
		Ok(_) => println!("Added position component to player"),
		Err(e) => panic!("Failed to add position component to player:{}", e),
	}

	match es.add_component(player, Box::new(Moveable::new(0.0, 0.0))) {
		Ok(_) => println!("Added moveable component to player"),
		Err(e) => panic!("Failed to add moveable component to player:{}", e),
	}

	match System_Moveable(&mut es, 1.0) {
		Ok(_) => println!("Ran System_Moveable"),
		Err(e) => println!("Failed to run System_Moveable:{}", e),
	}

}
