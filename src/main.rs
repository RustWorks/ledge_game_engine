mod lib;
mod graphics;
mod interface;
mod conf;
mod game;
mod event;
mod error;
mod ecs;

// use interface::*;
// use game::*;
use ecs::World;
use ecs::system::System;
use ecs::component::Component;
use ecs::storage::VecStorage;
use ecs::storage::ReadStorage;
use ecs::storage::WriteStorage;
// use std::any::type_name;
use ecs::entity::Entities;

// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

fn main() {
    let mut test_world = World::new();

    let mut test_system0 = TestSystem1{};
    let mut test_system1 = TestSystem{};

    let test_resource = TestRes::new();

    // test_world.insert::<Entities>(Entities {});
    test_world.insert::<u8>(8);
    test_world.insert::<TestRes>(test_resource);

    test_world.register::<TestComp>();

    test_world.create_entity().with(TestComp{test:(0,0)}).build();
    test_world.create_entity().with(TestComp{test:(10,20)}).build();

    {
        let write_storage: WriteStorage<TestComp> = test_world.write_comp_storage::<TestComp>();
        test_system0.run(write_storage);
    }
    {
        let read_storage: ReadStorage<TestComp> = test_world.read_comp_storage::<TestComp>();
        test_system1.run(read_storage);
    }

    test_world.remove::<u8>();
    test_world.remove::<TestRes>();

    // let (interface, event_loop) = InterfaceBuilder::new("test", "Dan").build().unwrap();

    // let game = Game {
    //     world: Vec::new()
    // };

    // event::run(interface, event_loop, game);
}

pub struct TestComp {
    test: (u8, u8),
}

impl TestComp {
    pub fn test(&self) -> (u8, u8) {
        return self.test;
    }
}

impl Component for TestComp {
    type Storage = VecStorage<TestComp>;
}

#[derive(Debug)]
pub struct TestRes {
    pub test: u8
}

impl TestRes {
    pub fn new() -> Self {
        Self {
            test: 0,
        }
    }
}

struct TestSystem1 {}

impl<'a> System<'a> for TestSystem1 {
    type SystemData = WriteStorage<'a, TestComp>;

    fn run(&mut self, mut data0: Self::SystemData) {
        for data in (*data0.data).inner.inner.iter_mut() {
            (*data).test.0 += 10;
            (*data).test.1 += 20;
        }
    }
}

struct TestSystem {}

impl<'a> System<'a> for TestSystem {
    type SystemData = ReadStorage<'a, TestComp>;

    fn run(&mut self, data0: Self::SystemData) {
        for test in (*data0.data).inner.inner.iter() {
            println!("{:?}", test.test);
        }
    }
}