use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::any::{TypeId, Any};
use std::cell::{RefCell, RefMut, Ref};
use std::path::Path;
use std::time::Duration;

//sdl2
extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::rect::Point;

//sdl2_image
use sdl2::image::LoadTexture;

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
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

impl Drawable {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Drawable {
        return Drawable {x, y, w, h};
    }
}

struct Graphics {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

pub trait ComponentHashMap {
	fn as_any(&self) -> & dyn Any;
	
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
    entities: HashMap<u64, String>,
	entity_names: HashMap<String, u64>,
	components: HashMap<TypeId, Box<dyn ComponentHashMap>>,
}

impl EntitySystem {
	pub fn new() -> EntitySystem {
		return EntitySystem {next_id: 0, entities: HashMap::new(), entity_names: HashMap::new(), components: HashMap::new()};
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
		self.entity_names.insert(name.clone(), ent.id);
        self.entities.insert(ent.id, name);

		return Ok(ent);
	}

	pub fn remove_entity(&mut self, ent: Entity) -> Result<(), String> {
        let name = match self.entities.remove(&ent.id) {
            Some(n) => n,
            None => return Err(format!("No such entity with id:{}", ent.id)),
        };

		match self.entity_names.remove(&name) {
            Some(_) => (),
            None => return Err(format!("No such entity name {} but the entity id did exist", name)),
        }

		panic!("Removing components not implemented");
	}

//Currently not working and I'm too tired to fix it
    /*
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

		let component_hashmap = match self.components.get_mut(&TypeId::of::<ComponentType>()) {
			Some(chm) => chm,
			None => return Err(format!("Unknown component in store")),
		};

		if let Some(store) = component_hashmap.as_any_mut().downcast_mut::<RefCell<HashMap<u64,ComponentType>>>() {
			store.get_mut().insert(ent.id, component);
		}

		return Ok(());
	}

    pub fn get_entity_for_name(&self, name: String) -> Result<Entity, String> {
        let id = match self.entity_names.get(&name) {
            Some(i) => i,
            None => return Err(format!("No such entity with name:{}", name)),
        };

        return Ok(Entity {id: *id});
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

fn system_moveable(es: &mut EntitySystem, dt: f64) -> Result<(), String> {
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
//fn system_drawable(es: &EntitySystem, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, game_texture: &sdl2::render::Texture) -> Result<(), String> {
fn system_drawable(es: &mut EntitySystem, dt: f64) -> Result<(), String> {
    //Draw stuff here
    let mut graphics_map = match es.borrow_all_components_of_type_mut::<Graphics>() {
        Ok(g) => g,
        Err(e) => return Err(e),
    };

    if graphics_map.len() != 1 {
        return Err(format!("Expected exactly one graphics component got:{}", graphics_map.len()));
    }

    let graphics_ent = match es.get_entity_for_name(String::from("graphics")) {
        Ok(ge) => ge,
        Err(e) => return Err(e),
    };

    let mut graphics = match graphics_map.get_mut(&graphics_ent.id) {
        Some(g) => g,
        None => return Err(format!("Failed to find component for graphics entity")),
    };

    let mut canvas = &mut graphics.canvas;

    let texture_map = match es.borrow_all_components_of_type::<sdl2::render::Texture>() {
        Ok(tm) => tm,
        Err(e) => return Err(e),
    };

    if texture_map.len() != 1 {
        return Err(format!("Expected exactly one texture component got:{}", texture_map.len()));
    }

    let texture_ent = match es.get_entity_for_name(String::from("texture")) {
        Ok(te) => te,
        Err(e) => return Err(e),
    };

    let game_texture = match texture_map.get(&texture_ent.id) {
        Some(t) => t,
        None => return Err(format!("Failed to find component for texture entity")),
    };

	let drawables = match es.borrow_all_components_of_type::<Drawable>() {
		Ok(d) => d,
		Err(e) => return Err(e),
	};

	let mut positions = match es.borrow_all_components_of_type::<Position>() {
		Ok(p) => p,
		Err(e) => return Err(e),
	};

    canvas.clear();

	for (id, drawable) in drawables.iter() {
		match positions.get(&id) {
			Some(position) => {
                let center = Point::new(position.x as i32, position.y as i32);
                /*
                match canvas.copy(&game_texture, Some(Rect::new(drawable.x, drawable.y, drawable.w, drawable.h)), Some(Rect::from_center(center, drawable.w, drawable.h))) {
                    Ok(_) => (),
                    Err(e) => println!("Failed to copy texture:{}", e),
                }
                */
			},
			None => return Err(format!("No position for moveable for entity {}", id)),
		}

	}

    canvas.present();

    return Ok(());
}

fn create_player_entity(es: &mut EntitySystem) -> Result<Entity, String> {
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

	match es.add_component_to_entity(player, Drawable::new(17, 272, 15, 15)) {
		Ok(_) => println!("Added Drawable component to player"),
		Err(e) => panic!("Failed to add drawable component to player:{}", e),
	}

    return Ok(player);
}

fn main() {
    //SDL2 setup
    let sdl_context = match sdl2::init() {
        Ok(sc) => sc,
        Err(e) => {
            println!("Failed to initialise SDL2:{}", e);
            return;
        },
    };

    let sdl_video = match sdl_context.video() {
        Ok(sv) => sv,
        Err(e) => {
            println!("Failed to initialise SDL2_video subsystem:{}", e);
            return;
        },
    };

    let window = match sdl_video.window("Bomberus", 1280, 720)
        .position_centered()
        .build() {
            Ok(w) => w,
            Err(e) => {
                println!("Failed to create SDL window:{}", e);
                return;
            },
    };

    let mut canvas = match window.into_canvas()
        .accelerated()
        .build() {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to create hw accelerated canvas:{}", e);
            return;
        },
    };

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    //SDL2_image setup
    let sdl_image_context = match sdl2::image::init(sdl2::image::InitFlag::PNG) {
        Ok(sic) => sic,
        Err(e) => {
            println!("Failed to initialise SDL2_image:{}", e);
            return;
        },
    };

    //Entity System setup
	let mut es = EntitySystem::new();

    //Load game texture
    let texture_ent = match es.new_entity_with_name(String::from("game_texture")) {
        Ok(te) => te,
        Err(e) => {
            println!("Failed to create texture entity:{}", e);
            return;
        },

    };

    let texture_creator_ent = match es.new_entity_with_name(String::from("texture_creator")) {
        Ok(tc) => tc,
        Err(e) => {
            println!("Failed to create texture entity:{}", e);
            return;
        },

    };

    let texture_creator = canvas.texture_creator();

    let game_texture = match texture_creator.load_texture(Path::new("assets/bomb_party_v4.png")) {
        Ok(gt) => gt,
        Err(e) => {
            println!("Unable to load game texture:{}", e);
            return;
        },
    };
    //es.add_component_to_entity::<sdl2::render::Texture>(texture_ent, game_texture);

    let player = match create_player_entity(&mut es) {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to create player:{}", e);
            return;
        },
    };
    let mut event_pump = match sdl_context.event_pump() {
        Ok(ep) => ep,
        Err(e) => {
            println!("Failed to get event pump:{}", e);
            return;
        },
    };

    //Systems
    let mut systems: Vec<&dyn Fn(&mut EntitySystem, f64) -> Result<(), String>> = vec![&system_moveable, &system_drawable];

    let mut frame: usize = 0;
    //Main game loop
    'running: loop {
        //Handle input
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::KeyDown {keycode: Escape, ..} => {
                    println!("Quiting");
                    break 'running;
                },
                Event::Quit {..} => {
                    println!("Quiting");
                    break 'running;
                },
                _ => (),
            }
        }

        //Game update
		/* Need to fix this when I'm not so tired
		match es.borrow_component_for_entity::<Position>(player) {
			Ok(position) => println!("Frame:{} Player position x:{} y:{}", i, position.x, position.y),
			Err(e) => println!("Failed to player position:{}", e),
		}
		match es.borrow_all_components_of_type::<Position>() {
			Ok(positions) => { 
				match positions.get(&player.id) {
					Some(position) => println!("Frame:{} Player position x:{} y:{}", frame, position.x, position.y),
					None => println!("Failed to player position"),
				}
			},
			Err(e) => println!("Unable to borrow positions:{}", e),
		};
		*/

        //drawing
        for system in systems.iter() {
            system(&mut es, 1.0);
        }

        //Sleep for the rest of the frame
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        frame += 1;
    }

}
