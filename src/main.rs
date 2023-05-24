use std::path::Path;
use std::time::Duration;

//sdl2
extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::rect::Point;

//sdl2_image
use sdl2::image::LoadTexture;

mod components;
mod entity_system;
use components::{Position, Moveable, Drawable, Collidable};
use entity_system::{Entity, EntitySystem};

fn system_moveable(es: &mut EntitySystem, dt: f64) -> Result<(), String> {
	let moveables = match es.borrow_all_components_of_type::<Moveable>() {
		Ok(m) => m,
		Err(e) => return Err(e),
	};

	let mut positions = match es.borrow_all_components_of_type_mut::<Position>() {
		Ok(p) => p,
		Err(e) => return Err(e),
	};

    const MAX_SPEED: f64 = 10.0;

	for (id, moveable) in moveables.iter() {
		match positions.get_mut(&id) {
			Some(position) => {
                //If we're moving in a diagonal we need to scale the movement so the actual length
                //is 1. This will probably break when I add in smoothing....
                if moveable.dx != 0.0 && moveable.dy != 0.0 {
                    let length = ((moveable.dx * moveable.dx) + (moveable.dy * moveable.dy)).sqrt();
                    position.x += (MAX_SPEED * (moveable.dx / length)) * dt;
                    position.y += (MAX_SPEED * (moveable.dy / length)) * dt;
                } else {
                    position.x += (MAX_SPEED * moveable.dx) * dt;
                    position.y += (MAX_SPEED * moveable.dy) * dt;
                }
			},
			None => return Err(format!("No position for moveable for entity {}", id)),
		}

	}

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

	match es.add_component_to_entity(player, Moveable::new(0.0, 0.0)) {
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
    match sdl2::image::init(sdl2::image::InitFlag::PNG) {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to initialise SDL2_image:{}", e);
            return;
        },
    };

    //Entity System setup
	let mut es = EntitySystem::new();

    let texture_creator = canvas.texture_creator();

    let game_texture = match texture_creator.load_texture(Path::new("assets/bomb_party_v4.png")) {
        Ok(gt) => gt,
        Err(e) => {
            println!("Unable to load game texture:{}", e);
            return;
        },
    };

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

    let mut system_drawable = move |es: &mut EntitySystem, _dt: f64| -> Result<(), String> {
        let drawables = match es.borrow_all_components_of_type::<Drawable>() {
            Ok(d) => d,
            Err(e) => return Err(e),
        };

        let positions = match es.borrow_all_components_of_type::<Position>() {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        canvas.clear();

        for (id, drawable) in drawables.iter() {
            match positions.get(&id) {
                Some(position) => {
                    let center = Point::new(position.x as i32, position.y as i32);
                    match canvas.copy(&game_texture, Some(Rect::new(drawable.x, drawable.y, drawable.w, drawable.h)), Some(Rect::from_center(center, drawable.w*2, drawable.h*2))) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to copy texture:{}", e),
                    }
                },
                None => return Err(format!("No position for moveable for entity {}", id)),
            }

        }

        canvas.present();

        return Ok(());
    };

    //Systems
    let systems: Vec<&dyn Fn(&mut EntitySystem, f64) -> Result<(), String>> = vec![&system_moveable];
    let mut systems_mut: Vec<&mut dyn FnMut(&mut EntitySystem, f64) -> Result<(), String>> = vec![&mut system_drawable];

    let mut frame: usize = 0;
    //Main game loop
    'running: loop {
        //Handle input
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::KeyDown {keycode: Some(Keycode::Left), repeat: false, ..} => { 
                    match es.component_for_entity_mut::<Moveable, _>(player, |mv: &mut Moveable| -> Result<(), String> {
                        if mv.dx > 0.0 {
                            mv.dx = 0.0;
                        } else {
                            mv.dx = -1.0;
                        }
                        return Ok(());
                    }) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to up date moveable for player"),
                    }
                },
                Event::KeyDown {keycode: Some(Keycode::Right), repeat: false, ..} => {
                    match es.component_for_entity_mut::<Moveable, _>(player, |mv: &mut Moveable| -> Result<(), String> {
                        if mv.dx < 0.0 {
                            mv.dx = 0.0;
                        } else {
                            mv.dx = 1.0;
                        }
                        return Ok(());
                    }) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to up date moveable for player"),
                    }
                },
                Event::KeyDown {keycode: Some(Keycode::Up), repeat: false, ..} => {
                    match es.component_for_entity_mut::<Moveable, _>(player, |mv: &mut Moveable| -> Result<(), String> {
                        if mv.dy > 0.0 {
                            mv.dy = 0.0;
                        } else {
                            mv.dy = -1.0;
                        }
                        return Ok(());
                    }) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to up date moveable for player"),
                    }
                },
                Event::KeyDown {keycode: Some(Keycode::Down), repeat: false, ..} => {
                    match es.component_for_entity_mut::<Moveable, _>(player, |mv: &mut Moveable| -> Result<(), String> {
                        if mv.dy < 0.0 {
                            mv.dy = 0.0;
                        } else {
                            mv.dy = 1.0;
                        }
                        return Ok(());
                    }) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to up date moveable for player"),
                    }
                },
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    println!("Quiting");
                    break 'running;
                },
                Event::KeyUp {keycode: Some(Keycode::Left), repeat: false, ..} => {
                    match es.component_for_entity_mut::<Moveable, _>(player, |mv: &mut Moveable| -> Result<(), String> {
                        mv.dx += 1.0;
                        return Ok(());
                    }) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to up date moveable for player"),
                    }
                },
                Event::KeyUp {keycode: Some(Keycode::Right), repeat: false, ..} => {
                    match es.component_for_entity_mut::<Moveable, _>(player, |mv: &mut Moveable| -> Result<(), String> {
                        mv.dx -= 1.0;
                        return Ok(());
                    }) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to up date moveable for player"),
                    }
                },
                Event::KeyUp {keycode: Some(Keycode::Up), repeat: false, ..} => {
                    match es.component_for_entity_mut::<Moveable, _>(player, |mv: &mut Moveable| -> Result<(), String> {
                        mv.dy += 1.0;
                        return Ok(());
                    }) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to up date moveable for player"),
                    }
                },
                Event::KeyUp {keycode: Some(Keycode::Down), repeat: false, ..} => {
                    match es.component_for_entity_mut::<Moveable, _>(player, |mv: &mut Moveable| -> Result<(), String> {
                        mv.dy -= 1.0;
                        return Ok(());
                    }) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to up date moveable for player"),
                    }
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

        for system_mut in systems_mut.iter_mut() {
            system_mut(&mut es, 1.0);
        }

        //Sleep for the rest of the frame
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        println!("Frame:{}", frame);

        frame += 1;
    }

}
