use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use bevy_schedule_dispatch::prelude::*;

#[derive(ScheduleLabel, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct ExampleDispatcher;

fn example_system(input: NonSend<dispatch_input::DispInA<ExampleDispatcher, bool>>) {
    println!("example dispatcher called! {:?}", input);
}

fn example_system_ret(mut output: NonSendMut<DispOut<ExampleDispatcher, i32>>) {
    println!("example dispatcher called, adjusting return value!");
    output.ret = 10;
}

pub fn main() {
    let mut dispatch_schedule = Schedule::new();
    // We set it to `SingleThreaded` otherwise main will exit before the schedules thread has exited (important for miri).
    dispatch_schedule.set_executor_kind(bevy_ecs::schedule::ExecutorKind::SingleThreaded);
    App::new()
        .add_plugins(DispatchPlugin)
        .add_schedule(ExampleDispatcher, dispatch_schedule)
        .add_systems(ExampleDispatcher, (example_system, example_system_ret))
        .set_runner(|mut app| {
            app.finish();
            app.cleanup();

            let _global_app = DispatchPlugin::globalize_app(app);

            let ret = <fn(bool) -> i32 as Dispatchable>::dispatcher::<ExampleDispatcher>()(true);
            println!("example dispatcher returned {}!", ret);
        })
        .run();
}
