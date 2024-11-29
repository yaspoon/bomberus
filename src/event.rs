use crate::Entity;

pub enum Event {
    Stub,
    DestroyEntity(Entity),
}

impl Event {
    fn _new() -> Event {
        Event::Stub
    }

    fn _spawn() -> Event {
        Event::Stub
    }

    pub fn destroy_entity(entity: Entity) -> Event {
        Event::DestroyEntity(entity)
    }
}
