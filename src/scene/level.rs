use crate::graphics::*;
use crate::graphics::context::GraphicsContext;
use crate::ecs::component::Component;
use crate::ecs::storage::NullStorage;
use crate::ecs::entity::Entity;
use crate::ecs::storage::WriteStorage;
use crate::ecs::storage::ReadStorage;
use crate::ecs::system::System;
use crate::ecs::join::Joinable;
use crate::scene::*;
use crate::scene::stack::*;
use crate::graphics::sprite::Sprite;

pub struct LevelSpaceBuilder {
    pub(crate) level_scene: LevelSpace,
    pub(crate) is_built: bool,
}

impl LevelSpaceBuilder {
    pub fn new() -> Self {
        Self {
            level_scene: LevelSpace::default(),
            is_built: false,
        }
    }

    pub fn from_conf(&mut self) {}

    pub fn with(&mut self) {}

    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.level_scene.entities.push(entity);
        self
    }

    pub fn from_conf_file(&mut self) {}

    pub fn build(self) -> LevelSpace {
        self.level_scene
    }
}

#[derive(Default, Clone)]
pub struct LevelSpace {
    pub entities: Vec<Entity>,
}

impl Space<World> for LevelSpace {
    fn update(&mut self, _context: &mut GraphicsContext) -> SpaceSwitch<World> {
        SpaceSwitch::None
    }

    fn draw(&mut self, world: &mut World, context: &mut GraphicsContext) -> GameResult<()> {
        let mut sprite_system = SpriteDraw {
            context
        };

        sprite_system.run((world.write_comp_storage::<Sprite>(), world.read_comp_storage::<Visible>()));

        Ok(())
    }

    fn input(&mut self, _gameworld: &mut World, _started: bool) {

    }

    fn current_scene(&self) -> bool {
        true
    }
}

#[derive(Default, Clone)]
pub struct Visible {}

impl Component for Visible {
    type Storage = NullStorage<Self>;
}

struct SpriteDraw<'a> {
    context: &'a mut GraphicsContext,
}

impl<'a> System<'a> for SpriteDraw<'a> {
    type SystemData = (WriteStorage<'a, Sprite>, ReadStorage<'a, Visible>);

    fn run(&mut self, (mut sprite, scene): Self::SystemData) {
        for (sprite, _) in (&mut sprite, &scene).join() {
            sprite.draw(&mut self.context);
        }
    }
}