use std::error::Error;
use std::fmt;

use sdl2::rect::Rect;
use sdl2::rect::Point;

use crate::GameError;
use crate::components::{Position, Moveable, Drawable, Animations, AnimationType, Direction};
use crate::entity_system::{Entity, EntitySystem, EntitySystemError};

#[derive(Debug)]
pub enum SystemsError {
    Moveable(String),
    Drawable(String),
    Position(String),
    Direction(String),
}

impl Error for SystemsError {}

impl fmt::Display for SystemsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemsError::Moveable(e) => {
                write!(f, "SystemsError::Moveable::{}", e)
            },
            SystemsError::Drawable(e) => {
                write!(f, "SystemsError::Drawable::{}", e)
            },
            SystemsError::Position(e) => {
                write!(f, "SystemsError::Position::{}", e)
            },
            SystemsError::Direction(e) => {
                write!(f, "SystemsError::Direction::{}", e)
            },
        }
    }
}

pub fn system_moveable(es: &mut EntitySystem, dt: f64) -> Result<(), GameError> {
	let moveables = match es.borrow_all_components_of_type::<Moveable>() {
		Ok(m) => m,
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No moveable components in the EntitySystem");
                    return Ok(()); //Not having any moveables isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
	};

	let mut positions = match es.borrow_all_components_of_type_mut::<Position>() {
		Ok(p) => p,
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No position components in the EntitySystem");
                    return Ok(()); //Not having any positions isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
	};

    const MAX_SPEED: f64 = 400.0;

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
			None => return Err(GameError::SystemsError(SystemsError::Moveable(format!("No position for moveable for entity {}", id)))),
		}

	}

	return Ok(());
}

pub fn system_direction(es: &mut EntitySystem, _dt: f64) -> Result<(), GameError> {
	let moveables = match es.borrow_all_components_of_type::<Moveable>() {
		Ok(m) => m,
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No moveable components in the EntitySystem");
                    return Ok(()); //Not having any moveables isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
	};

	let mut directions = match es.borrow_all_components_of_type_mut::<Direction>() {
		Ok(d) => d,
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No direction components in the EntitySystem");
                    return Ok(()); //Not having any directions isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
	};

	for (id, moveable) in moveables.iter() {
		match directions.get_mut(&id) {
			Some(direction) => {
                if moveable.dx != 0.0 && moveable.dy == 0.0 {
                    //println!("Moving in x direction");
                    if moveable.dx < 0.0 && *direction != Direction::Left {
                        //println!("Changing direction to left\n");
                        *direction = Direction::Left;
                    } else if moveable.dx > 0.0 && *direction != Direction::Right {
                        *direction = Direction::Right;
                        //println!("Changing direction to right\n");
                    }
                } else if moveable.dx == 0.0 && moveable.dy != 0.0 {
                    //println!("Moving in y direction");
                    if moveable.dy < 0.0 && *direction != Direction::Up {
                        //println!("Changing direction to up\n");
                        *direction = Direction::Up;
                    } else if moveable.dy > 0.0 && *direction != Direction::Down {
                        //println!("Changing direction to down\n");
                        *direction = Direction::Down;
                    }
                }
			},
			None => return Err(GameError::SystemsError(SystemsError::Direction(format!("No direction for moveable for entity {}", id)))),
		}

	}

	return Ok(());
}

pub fn system_animation(es: &mut EntitySystem, dt: f64) -> Result<(), GameError> {
	let mut animations = match es.borrow_all_components_of_type_mut::<Animations>() {
		Ok(a) => a,
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No Animations components in the EntitySystem");
                    return Ok(()); //Not having any positions isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
	};

	let mut moveables = match es.borrow_all_components_of_type_mut::<Moveable>() {
		Ok(m) => m,
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No Moveable components in the EntitySystem");
                    return Ok(()); //Not having any Moveables isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
	};

	let mut directions = match es.borrow_all_components_of_type_mut::<Direction>() {
		Ok(d) => d,
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No Direction components in the EntitySystem");
                    return Ok(()); //Not having any Directions isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
	};

    for (id, animations) in animations.iter_mut() {
        if animations.current_animation != AnimationType::Empty {
            let old_animation_type = animations.current_animation; //Used to check if we changed animation
            let current_animation_type = &mut animations.current_animation;
            if animations.animations.contains_key(&current_animation_type) {
                //Sigh this is pretty filthy, is it really the animation systems problem to figure out
                //if we're facing the right way? There should probably be a direction component to make
                //this less shit...
                let is_moving = match moveables.get(&id) {
                    Some(m) => {
                        if m.dx != 0.0 || m.dy != 0.0 {
                            true
                        } else {
                            false
                        }
                    },
                    None => false,
                };

                match directions.get(&id) {
                    Some(d) => { //Check to make sure the animation lines up with how the entity is "moving"
                        match d{
                            Direction::Up => {
                                if is_moving {
                                    if *current_animation_type != AnimationType::WalkingUp {
                                        *current_animation_type = AnimationType::WalkingUp;
                                    }
                                } else {
                                    if *current_animation_type != AnimationType::StandingUp {
                                        *current_animation_type = AnimationType::StandingUp;
                                    }
                                }
                            },
                            Direction::Down => {
                                if is_moving {
                                    if *current_animation_type != AnimationType::WalkingDown {
                                        *current_animation_type = AnimationType::WalkingDown;
                                    }
                                } else {
                                    if *current_animation_type != AnimationType::StandingDown {
                                        *current_animation_type = AnimationType::StandingDown;
                                    }
                                }
                            },
                            Direction::Right => {
                                if is_moving {
                                    if *current_animation_type != AnimationType::WalkingRight {
                                        *current_animation_type = AnimationType::WalkingRight;
                                    }
                                } else {
                                    if *current_animation_type != AnimationType::StandingRight {
                                        *current_animation_type = AnimationType::StandingRight;
                                    }
                                }
                            },
                            Direction::Left => {
                                if is_moving {
                                    if *current_animation_type != AnimationType::WalkingLeft {
                                        *current_animation_type = AnimationType::WalkingLeft;
                                    }
                                } else {
                                    if *current_animation_type != AnimationType::StandingLeft {
                                        *current_animation_type = AnimationType::StandingLeft;
                                    }
                                }
                            },
                            _ => (),
                        }
                    },
                    None => (), //Not having a moveable associated with the animation is totally okay!
                };

                if *current_animation_type != old_animation_type { //If we changed animation we need to reset the frame count
                    animations.current_frame = 0;
                }

                let current_animation = &animations.animations[&current_animation_type];

                if current_animation.fps > 0.0 {
                    let time_between_frames = 1.0 / current_animation.fps; //Last_frame_time is accumilative, so we need to know how much time elapses between frames, not how frames there are per second
                    animations.last_frame_time += dt;
                    if animations.last_frame_time >= time_between_frames { //Change frames
                        animations.current_frame = (animations.current_frame + 1) % animations.animations[&animations.current_animation].frames.len();
                        animations.last_frame_time -= time_between_frames;
                    }
                }
            }
        }
    }

    return Ok(());
}

pub fn system_drawable(es: &mut EntitySystem, _dt: f64) -> Result<(), GameError> {
    let mut canvas = es.get_mut_canvas();

    let game_texture = es.get_texture();
    let drawables = match es.borrow_all_components_of_type::<Drawable>() {
        Ok(d) => Some(d),
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    //println!("No Drawable components in the EntitySystem");
                    None //Not having any drawables isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
    };

    let animations = match es.borrow_all_components_of_type::<Animations>() {
        Ok(a) => Some(a),
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No Animations components in the EntitySystem");
                    None //Not having any animations isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
    };

    let positions = match es.borrow_all_components_of_type::<Position>() {
        Ok(p) => p,
		Err(e) => {
            match e {
                EntitySystemError::NoSuchComponent(_) => {
                    println!("No position components in the EntitySystem");
                    return Ok(()); //Not having any positions isn't the end of the world
                },
                _ => return Err(GameError::EntitySystemError(e)),
            }
        },
    };

    canvas.clear();

    if let Some(d) = drawables {
        for (id, drawable) in d.iter() {
            match positions.get(&id) {
                Some(position) => {
                    let center = Point::new(position.x as i32, position.y as i32);
                    match canvas.copy(&game_texture, Some(Rect::new(drawable.x, drawable.y, drawable.w, drawable.h)), Some(Rect::from_center(center, drawable.w*2, drawable.h*2))) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to copy texture:{}", e),
                    }
                },
                None => return Err(GameError::SystemsError(SystemsError::Position(format!("No position for drawable for entity {}", id)))),
            }

        }
    }

    if let Some(a) = animations {
        for (id, animations) in a.iter() {
            if animations.current_animation != AnimationType::Empty { //Empty animations aren't drawn
                match positions.get(&id) {
                    Some(position) => {
                        let center = Point::new(position.x as i32, position.y as i32);
                        let animation = match animations.animations.get(&animations.current_animation) {
                            Some(a) => a,
                            None => continue,
                        };
                        match animation.frames.get(animations.current_frame) {
                            Some(drawable) => {
                                match canvas.copy_ex(&game_texture, Some(Rect::new(drawable.x, drawable.y, drawable.w, drawable.h)), Some(Rect::from_center(center, drawable.w*2, drawable.h*2)), 0.0, None,
                                    animation.flip_horizontal, animation.flip_vertical) {
                                    Ok(_) => (),
                                    Err(e) => println!("Failed to copy texture:{}", e),
                                }
                            },
                            None => println!("Failed to get current frame {} of {} frames", animations.current_frame, animation.frames.len()),
                       }
                    },
                    None => return Err(GameError::SystemsError(SystemsError::Position(format!("No position for animation for entity {}", id)))),
                }
            }
        }
    }

    canvas.present();

    return Ok(());
}
