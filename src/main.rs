use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::any::{TypeId, Any};
use std::cell::{RefCell, RefMut, Ref};

struct Position {
	x: f64,
	y: f64,
}

impl Position {
	pub fn new(x: f64, y: f64) -> Position {
		return Position {x, y};
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

pub trait ComponentHashMap {
	fn as_any(&self) -> &dyn Any;
	
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> ComponentHashMap for RefCell<HashMap<u64, T>> {
	fn as_any(&self) -> &dyn Any {
		return self as &dyn Any;
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		return self as &mut dyn Any;
	}
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
	components: HashMap<TypeId, Box<dyn ComponentHashMap>>,
}

impl EntitySystem {
	pub fn new() -> EntitySystem {
		return EntitySystem {next_id: 0, entities: Vec::new(), entity_names: HashMap::new(), components: HashMap::new()};
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

		panic!("Removing components not implemented");
	}

/* Currently not working and I'm too tired to fix it
	pub fn borrow_component_for_entity<ComponentType: 'static>(&self, entity: Entity) -> Result<&ComponentType, String> {
		let component_hashmap = match self.components.get(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component type in store")),
		};

		if let Some(store) = component_hashmap.as_any().downcast_ref::<RefCell<HashMap<u64,ComponentType>>>() {
			//This will panic if something has already borrowed it. Should probably not panic....
			match store.borrow().get(&entity.id) {
				Some(c) => return Ok(c),
				None => return Err(format!("Couldn't find component for entity:{}", entity)),
			}
		}

		return Err(format!("Unable to downcast ref to expected component type"));
	}
*/

	pub fn borrow_all_components_of_type<ComponentType: 'static>(&self) -> Result<Ref<HashMap<u64, ComponentType>>, String> {
		let component_hashmap = match self.components.get(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component type in store")),
		};

		if let Some(store) = component_hashmap.as_any().downcast_ref::<RefCell<HashMap<u64,ComponentType>>>() {
			//This will panic if something has already borrowed it. Should probably not panic....
			return Ok(store.borrow());
		}

		return Err(format!("Unable to downcast ref to expected component type"));
	}

	pub fn borrow_all_components_of_type_mut<ComponentType: 'static>(&self) -> Result<RefMut<HashMap<u64, ComponentType>>, String> {
		let component_hashmap = match self.components.get(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component type in store")),
		};

		if let Some(store) = component_hashmap.as_any().downcast_ref::<RefCell<HashMap<u64,ComponentType>>>() {
			//This will panic if something has already borrowed it. Should probably not panic....
			return Ok(store.borrow_mut());
		}

		return Err(format!("Unable to downcast ref to expected component type"));
	}

	pub fn add_component_to_entity<ComponentType: 'static>(&mut self, ent: Entity, component: ComponentType) -> Result<(), String> {
		if !self.components.contains_key(&TypeId::of::<ComponentType>()) {
			println!("Store doesn't contain component, creating");
			self.components.insert(TypeId::of::<ComponentType>(), Box::new(RefCell::new(HashMap::<u64, ComponentType>::new())));
		}

		let mut component_hashmap = match self.components.get_mut(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component in store")),
		};

		if let Some(store) = component_hashmap.as_any_mut().downcast_mut::<RefCell<HashMap<u64,ComponentType>>>() {
			store.get_mut().insert(ent.id, component);
		}

		return Ok(());
	}

/*
	pub fn remove_component_from_entity<ComponentType: 'static>(&mut self, ent: Entity) -> Result<(), String> {
		let mut store = match self.components.get_mut(&comp) {
			Some(s) => s,
			None => return Err(format!("Unknown component type for store")),
		};

		match store.remove(&ent.id) {
			Some(v) => return Ok(()),
			None => return Err(format!("No such component for entity {}", ent)),
		}
	}
*/
}

fn System_Moveable(es: &mut EntitySystem, dt: f64) -> Result<(), String> {
	let moveables = match es.borrow_all_components_of_type::<Moveable>() {
		Ok(m) => m,
		Err(e) => return Err(e),
	};

	let mut positions = match es.borrow_all_components_of_type_mut::<Position>() {
		Ok(p) => p,
		Err(e) => return Err(e),
	};

	for (id, moveable) in moveables.iter() {
		match positions.get_mut(&id) {
			Some(position) => {
				position.x += moveable.dx * dt;
				position.y += moveable.dy * dt;
			},
			None => return Err(format!("No position for moveable for entity {}", id)),
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

	match es.add_component_to_entity(player, Position::new(0.0, 0.0)) {
		Ok(_) => println!("Added position component to player"),
		Err(e) => panic!("Failed to add position component to player:{}", e),
	}

	match es.add_component_to_entity(player, Moveable::new(1.0, 1.0)) {
		Ok(_) => println!("Added moveable component to player"),
		Err(e) => panic!("Failed to add moveable component to player:{}", e),
	}

	for i in 0..10 {
		/* Need to fix this when I'm not so tired
		match es.borrow_component_for_entity::<Position>(player) {
			Ok(position) => println!("Frame:{} Player position x:{} y:{}", i, position.x, position.y),
			Err(e) => println!("Failed to player position:{}", e),
		}
		*/
		match es.borrow_all_components_of_type::<Position>() {
			Ok(positions) => { 
				match positions.get(&player.id) {
					Some(position) => println!("Frame:{} Player position x:{} y:{}", i, position.x, position.y),
					None => println!("Failed to player position"),
				}
			},
			Err(e) => println!("Unable to borrow positions:{}", e),
		};


		match System_Moveable(&mut es, 1.0) {
			Ok(_) => println!("Ran System_Moveable"),
			Err(e) => println!("Failed to run System_Moveable:{}", e),
		}
	}

}
