use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use bevy_schedule_dispatch::prelude::*;

#[derive(ScheduleLabel, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct ExampleHook;

fn example_original_fn(p0: bool) -> i32 {
    println!("original has been called with: {}", p0);
    return 5;
}

fn example_system(input: NonSend<dispatch_input::DispInA<ExampleHook, bool>>) {
    println!("example hook called! {:?}", input);
}

fn example_system_orig(
    input: NonSend<dispatch_input::DispInA<ExampleHook, bool>>,
    mut output: NonSendMut<DispOut<ExampleHook, i32>>,
) {
    output.ret = EXAMPLE.call(input.__arg_0);
}

fn example_system_ret(mut output: NonSendMut<DispOut<ExampleHook, i32>>) {
    println!("example hook called, multiplying return value!");
    output.ret = output.ret * 2;
}

retour::static_detour! {
    static EXAMPLE: fn(bool) -> i32;
}

pub fn main() {
    unsafe {
        let _ = EXAMPLE.initialize(
            example_original_fn,
            <fn(bool) -> i32 as Dispatchable>::dispatcher::<ExampleHook>(),
        );
        let _ = EXAMPLE.enable();
    };

    App::new()
        .add_plugins(DispatchPlugin)
        .init_schedule(ExampleHook)
        .add_systems(
            ExampleHook,
            (
                example_system,
                example_system_ret,
                example_system_orig.before(example_system_ret),
            ),
        )
        .set_runner(|mut app| {
            app.finish();
            app.cleanup();

            let _global_app = DispatchPlugin::globalize_app(app);

            std::thread::spawn(|| {
                let ret = example_original_fn(true);
                println!("example hook returned {}!", ret);
            })
            .join()
            .unwrap();
        })
        .run();
}
