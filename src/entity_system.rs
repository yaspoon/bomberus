use std::fmt::{self};
use std::collections::HashMap;
use std::any::{TypeId, Any};
use std::cell::{RefCell, RefMut, Ref};

use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use std::error::Error;

#[derive(Debug)]
pub enum EntitySystemError {
    NoSuchComponent(String),
    DowncastFailed(String),
}

impl fmt::Display for EntitySystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntitySystemError::NoSuchComponent(msg) => {
                write!(f, "no such component stored in EntitySystem: {}", msg)
            },
            EntitySystemError::DowncastFailed(msg) => {
                write!(f, "Downcast failed for component: {}", msg)
            },
        }
    }
}

impl Error for EntitySystemError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return Some(self);
    }
}

pub trait ComponentHashMap {
	fn as_any(&self) -> & dyn Any;
	
	fn as_any_mut(&mut self) -> &mut dyn Any;

        fn remove_entity(&mut self, id: Entity);
}

impl<T: 'static> ComponentHashMap for RefCell<HashMap<u64, T>> {
	fn as_any(&self) -> &dyn Any {
		return self as &dyn Any;
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		return self as &mut dyn Any;
	}

        fn remove_entity(&mut self, id: Entity) {
            if let Some(store) = self.as_any_mut().downcast_ref::<RefCell<HashMap<u64, T>>>() {
                store.borrow_mut().remove(&id);
            }
        }
}

pub type Entity = u64;

pub struct EntitySystem<'a> {
	next_id: u64,
    entities: HashMap<u64, String>,
	entity_names: HashMap<String, u64>,
	components: HashMap<TypeId, Box<dyn ComponentHashMap>>,
    canvas: RefCell<Canvas<Window>>,
    texture: Texture<'a>
}

impl<'a> EntitySystem<'a> {
	pub fn new(canvas: Canvas<Window>, texture: Texture<'a>) -> EntitySystem<'a> {
		return EntitySystem {next_id: 0, entities: HashMap::new(), entity_names: HashMap::new(), components: HashMap::new(), canvas: RefCell::new(canvas), texture};
	}

	pub fn new_entity(&mut self) -> Result<Entity, String> {
		return self.new_entity_with_name("Unknown".to_string());
	}

	pub fn new_entity_with_name(&mut self, name: String) -> Result<Entity, String> {
		if self.next_id == u64::MAX {
			panic!("Overflowing next_id, this should be fixed...");	
		}

		let ent = self.next_id;

		self.next_id += 1;
		self.entity_names.insert(name.clone(), ent);
        self.entities.insert(ent, name);

		return Ok(ent);
	}

    pub fn remove_entity(&mut self, ent: Entity) -> Result<(), String> {
        let name = match self.entities.remove(&ent) {
            Some(n) => n,
            None => return Err(format!("No such entity with id:{}", ent)),
        };

	match self.entity_names.remove(&name) {
            Some(_) => (),
            None => return Err(format!("No such entity name {} but the entity id did exist", name)),
        }

        for (_, component_hashmap) in self.components.iter_mut() {
	    component_hashmap.remove_entity(ent);
        }

        Ok(())
    }

	pub fn borrow_all_components_of_type<ComponentType: 'static>(&self) -> Result<Ref<HashMap<u64, ComponentType>>, EntitySystemError> {
		let component_hashmap = match self.components.get(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(EntitySystemError::NoSuchComponent(format!("Unknown component type <{}> in store", std::any::type_name::<ComponentType>()))),
		};

		if let Some(store) = component_hashmap.as_any().downcast_ref::<RefCell<HashMap<u64,ComponentType>>>() {
			//This will panic if something has already borrowed it. Should probably not panic....
			return Ok(store.borrow());
		}

		return Err(EntitySystemError::DowncastFailed(format!("Unable to downcast ref to expected component type <{}>", std::any::type_name::<ComponentType>())));
	}

	pub fn borrow_all_components_of_type_mut<ComponentType: 'static>(&self) -> Result<RefMut<HashMap<u64, ComponentType>>, EntitySystemError> {
		let component_hashmap = match self.components.get(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(EntitySystemError::NoSuchComponent(format!("Unknown component type <{}> in store", std::any::type_name::<ComponentType>()))),
		};

		if let Some(store) = component_hashmap.as_any().downcast_ref::<RefCell<HashMap<u64,ComponentType>>>() {
			//This will panic if something has already borrowed it. Should probably not panic....
			return Ok(store.borrow_mut());
		}

		return Err(EntitySystemError::DowncastFailed(format!("Unable to downcast ref to expected component type <{}>", std::any::type_name::<ComponentType>())));
	}

    pub fn _component_for_entity<ComponentType: 'static, F>(&self, ent: Entity, cb: F) -> Result<(), String> where
        F: Fn(&ComponentType) -> Result<(), String> {
		let component_hashmap = match self.components.get(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component type in store")),
		};

		if let Some(store) = component_hashmap.as_any().downcast_ref::<RefCell<HashMap<u64,ComponentType>>>() {
			//This will panic if something has already borrowed it. Should probably not panic....
			let store = store.borrow();
            let comp = match store.get(&ent) {
                Some(c) => c,
                None => return Err(format!("No such component for entity:{}", ent)),
            };

            return cb(comp);
		}

		return Err(format!("Unable to downcast ref to expected component type"));
    }

    pub fn component_for_entity_mut<ComponentType: 'static, F>(&self, ent: Entity, cb: F) -> Result<(), String> where
        F: Fn(&mut ComponentType) -> Result<(), String> {
		let component_hashmap = match self.components.get(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component type in store")),
		};

		if let Some(store) = component_hashmap.as_any().downcast_ref::<RefCell<HashMap<u64,ComponentType>>>() {
			//This will panic if something has already borrowed it. Should probably not panic....
			let mut store = store.borrow_mut();
            let comp = match store.get_mut(&ent) {
                Some(c) => c,
                None => return Err(format!("No such component for entity:{}", ent)),
            };

            return cb(comp);
		}

		return Err(format!("Unable to downcast ref to expected component type"));
    }

	pub fn add_component_to_entity<ComponentType: 'static>(&mut self, ent: Entity, component: ComponentType) -> Result<(), String> {
		if !self.components.contains_key(&TypeId::of::<ComponentType>()) {
			println!("Store doesn't contain component, creating");
			self.components.insert(TypeId::of::<ComponentType>(), Box::new(RefCell::new(HashMap::<u64, ComponentType>::new())));
		}

		let component_hashmap = match self.components.get_mut(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component in store")),
		};

		if let Some(store) = component_hashmap.as_any_mut().downcast_mut::<RefCell<HashMap<u64,ComponentType>>>() {
			store.get_mut().insert(ent, component);
		}

		return Ok(());
	}

    pub fn _get_entity_for_name(&self, name: String) -> Result<Entity, String> {
        let id = match self.entity_names.get(&name) {
            Some(i) => i,
            None => return Err(format!("No such entity with name:{}", name)),
        };

        return Ok(*id);
    }

	pub fn _remove_component_for_entity<ComponentType: 'static>(&mut self, ent: Entity) -> Result<(), String> {
		let component_hashmap = match self.components.get(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component type in store")),
		};

		if let Some(store) = component_hashmap.as_any().downcast_ref::<RefCell<HashMap<u64,ComponentType>>>() {
			//This will panic if something has already borrowed it. Should probably not panic....
            match store.borrow_mut().remove(&ent) {
                Some(_) => return Ok(()),
                None => return Err(format!("No such component for entity!")),
            }
		}

		return Err(format!("Unable to downcast ref to expected component type"));
	}

    pub fn get_mut_canvas(&self) -> RefMut<Canvas<Window>> {
        return self.canvas.borrow_mut();
    }

    pub fn get_texture(&self) -> &Texture<'a> {
        return &self.texture;
    }
}

