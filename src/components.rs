use std::fmt::{self, Display};
use std::collections::HashMap;

//serde
use serde::{Deserialize,Serialize};

#[derive(Deserialize,Serialize)]
pub struct Position {
	pub x: f64,
	pub y: f64,
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

pub struct Moveable {
	pub dx: f64,
	pub dy: f64,
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

pub struct Collidable {
	_width: f64,
	_height: f64,
}

#[derive(Deserialize,Serialize)]
pub struct Drawable {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub layer: u32,
}

impl Drawable {
    pub fn new(x: i32, y: i32, w: u32, h: u32, layer: u32) -> Drawable {
        return Drawable {x, y, w, h, layer};
    }
}

#[derive(Serialize,Deserialize)]
pub struct Animation {
    pub frames: Vec<Drawable>,
    pub fps: f64,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub looping: bool,
}

impl Animation {
    pub fn _new() -> Self {
        Animation {frames: Vec::new(), fps: 0.0, flip_horizontal: false, flip_vertical: false, looping: false}
    }

    pub fn new_with_frames(frames: Vec<Drawable>, fps: f64, flip_horizontal: bool, flip_vertical: bool, looping: bool) -> Self {
        Animation {frames, fps, flip_horizontal, flip_vertical, looping}
    }
}

#[derive(PartialEq, Eq, std::hash::Hash, Copy, Clone)]
pub enum AnimationType {
    Empty,
    /*Player & Enemies*/
    StandingDown,
    StandingUp,
    StandingLeft,
    StandingRight,
    WalkingDown,
    WalkingUp,
    WalkingLeft,
    WalkingRight,
    /*Bomb*/
    CountingDown,
    Exploding,
}

pub struct Animations {
    pub animations: HashMap<AnimationType, Animation>,
    pub current_animation: AnimationType,
    pub current_frame: usize,
    pub last_frame_time: f64,
    pub playing: bool,
}

impl Animations {
    pub fn new(current_animation: AnimationType, animations: HashMap<AnimationType, Animation>) -> Self {
        return Self {current_animation, animations: animations, current_frame: 0, last_frame_time: 0.0, playing: false};
    }

    pub fn playing(&mut self, playing: bool) {
        self.playing = playing;
    }
}

#[derive(PartialEq, Eq, std::hash::Hash)]
pub enum Direction {
    Down,
    Up,
    Left,
    Right,
}

pub enum AIState {
    _Idle,
    Searching,
    _Attacking,
    _Fleeing,
}

pub enum AIType {
    Warrior, //Dumb and weak, single bomb kill
    _StrongWarrior, //Dumb and takes multiple bombs to kill
    _Wizard, //Smart and weak, single bomb kill, but can also place bombs
}

pub struct AI {
    pub last_think: f64,
    pub state: AIState,
    pub ai_type: AIType,
}

impl AI {
    pub fn new(ai_type: AIType) -> Self {
        AI {last_think: 0.0, state: AIState::Searching, ai_type}
    }
}

pub enum BombThinkState {
    Spawned,
    Exploding,
    Exploded,
}

pub struct BombThink {
    pub time_since_spawn: f64,
    pub state: BombThinkState,
}

impl BombThink {
    pub fn new() -> BombThink {
        BombThink { time_since_spawn: 0.0, state: BombThinkState::Spawned }
    }
}

/*
#[derive(PartialEq, Eq, std::hash::Hash)]
pub enum BombExplosionSprites {
    HorizontalMid,
    HorizontalLeftTip,
    HorizontalRightTip,
    Middle,
    VerticalMid,
    VerticalLeftTip,
    VerticalRightTip,

}

pub struct BombExplosion {
    pub sprites: HashMap<BombExplosionSprites, Drawable>,
}

impl BombExplosion {
    pub fn new(sprites: HashMap<BombExplosionSprites, Drawable>) -> BombExplosion {
        BombExplosion { sprites }
    }
}
*/

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use glob::glob;

    use ron;
    use super::*;

    /*
    #[derive(Serialize,Deserialize)]
    //#[serde(untagged)]
    enum ComponentRegistry {
        //Position(super::Position),
        Position {
                x: f64,
                y: f64,
        },
        Animation(Animation),
    }
    */

    include!(concat!(env!("OUT_DIR"), "/generated_components.rs"));

    /* Iterates over all *.ron files in /assets and tries to deserialise them
     * to a ComponentRegistry type. The ComponentRegistry type is dynamically created
     * by the build.rs so it is automatically updated whenever a new component is added to
     * this file. This is required because there is no easy way to get the type you want to
     * deserialise from the .ron file. So after going a bit crazy I asked gemini what to do
     * and it recommended a monilithic enum of all types should deserialise because you must
     * know the type when deserialising for serde to work.
     */
    #[test]
    fn test_asset_ron_file_deserialisation() {
        let assets_dir = "assets";
        let glob_pattern = format!("{}/**/*.ron", assets_dir);

        for entry in glob(&glob_pattern).expect("Failed to read glob pattern") {
            let path = entry.expect("Invalid entry");

            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                let content = fs::read_to_string(&path).expect("Failed to read file");

                if let Err(e) = ron::from_str::<ComponentRegistry>(&content) {
                    panic!("Failed to deserialise ron asset:{:?}\nReason:{}\n", path.display(), e);
                } else {
                    println!("Deserialized {} successfully", path.display());
                }
            }

        }
    }
}
