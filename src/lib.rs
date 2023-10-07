#![feature(fn_ptr_trait)]

use std::{
    cell::OnceCell,
    marker::{FnPtr, PhantomData},
    sync::{Arc, Mutex},
};

use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};

pub mod prelude {
    pub mod input {
        pub use crate::{
            DispIn, DispInA, DispInAB, DispInABC, DispInABCD, DispInABCDE, DispInABCDEF,
            DispInABCDEFG, DispInABCDEFGH, DispInABCDEFGHI, DispInABCDEFGHIJ, DispInABCDEFGHIJK,
            DispInABCDEFGHIJKL, DispInABCDEFGHIJKLM, DispInABCDEFGHIJKLMN,
        };
    }
    pub use crate::{prelude::input as dispatch_input, DispOut, DispatchPlugin, Dispatchable};
}

// TODO: Safety docs.
// TODO: Move this to the user...
static mut GLOBAL_APP: OnceCell<Arc<Mutex<App>>> = OnceCell::new();

pub struct DispatchPlugin;

impl DispatchPlugin {
    #[must_use = "dispatchers will panic if GLOBAL_APP is empty"]
    pub fn globalize_app(app: App) -> Arc<Mutex<App>> {
        let arc_app = Arc::new(Mutex::new(app));
        unsafe {
            match GLOBAL_APP.set(arc_app) {
                Err(app) => app,
                _ => GLOBAL_APP
                    .get()
                    .expect("GLOBAL_APP cell should NOT be empty")
                    .clone(),
            }
        }
    }
}

impl Plugin for DispatchPlugin {
    fn build(&self, _app: &mut bevy_app::App) {}
}

// Currently just the return value.
#[derive(Default)]
pub struct DispOut<S, Ret: 'static + FromWorld> {
    _marker: PhantomData<S>,
    pub ret: Ret,
}

impl<S: ScheduleLabel, Ret: 'static + FromWorld> DispOut<S, Ret> {
    pub fn new(ret: Ret) -> Self {
        Self {
            _marker: PhantomData,
            ret,
        }
    }
}

pub trait Dispatchable: Sized + Sync + 'static {
    type Func: FnPtr;

    fn dispatcher<S: ScheduleLabel + Default + AsRef<(dyn ScheduleLabel + 'static)>>() -> Self::Func;
}

macro_rules! impl_dispatchable {
    (@recurse () ($($nm:ident : $ty:ident),*)) => {
        impl_dispatchable!(@impl_all ($($nm : $ty),*));
    };

    (@recurse
        ($hd_nm:ident : $hd_ty:ident $(, $tl_nm:ident : $tl_ty:ident)*)
        ($($nm:ident : $ty:ident),*)) => {
        impl_dispatchable!(@impl_all ($($nm : $ty),*));
        impl_dispatchable!(@recurse ($($tl_nm : $tl_ty),*) ($($nm : $ty,)* $hd_nm : $hd_ty));
    };

    (@impl_all ($($nm:ident : $ty:ident),*)) => {
        ::paste::item! {
            #[derive(Debug)]
            pub struct [<DispIn $($ty)* >]<S, $($ty),*> {
                _marker: PhantomData<S>,
                $(
                    pub $nm: $ty,
                )*
            }

            impl<S: ScheduleLabel, $($ty: 'static + std::fmt::Debug ),*> [<DispIn $($ty)* >]<S, $($ty),*> {
                pub fn new($($nm: $ty,)*) -> Self {
                    Self {
                        _marker: PhantomData,
                        $(
                            $nm,
                        )*
                    }
                }
            }
        }

        impl_dispatchable!(@impl_pair ($($nm : $ty),*) () (                                   fn($($ty),*) -> Ret));
        impl_dispatchable!(@impl_pair ($($nm : $ty),*) (extern "cdecl")    (extern "cdecl"    fn($($ty),*) -> Ret));
        impl_dispatchable!(@impl_pair ($($nm : $ty),*) (extern "stdcall")  (extern "stdcall"  fn($($ty),*) -> Ret));
        impl_dispatchable!(@impl_pair ($($nm : $ty),*) (extern "fastcall") (extern "fastcall" fn($($ty),*) -> Ret));
        impl_dispatchable!(@impl_pair ($($nm : $ty),*) (extern "win64")    (extern "win64"    fn($($ty),*) -> Ret));
        impl_dispatchable!(@impl_pair ($($nm : $ty),*) (extern "C")        (extern "C"        fn($($ty),*) -> Ret));
        impl_dispatchable!(@impl_pair ($($nm : $ty),*) (extern "system")   (extern "system"   fn($($ty),*) -> Ret));
        //impl_dispatchable!(@impl_pair ($($nm : $ty),*) (extern "thiscall") (extern "thiscall" fn($($ty),*) -> Ret));
    };

    (@impl_pair ($($nm:ident : $ty:ident),*) ($($modifier:tt)*) ($($fn_t:tt)*)) => {
        impl_dispatchable!(@impl_fun ($($nm : $ty),*) ($($modifier),*) ($($fn_t)*) (unsafe $($fn_t)*));
    };

    (@impl_fun ($($nm:ident : $ty:ident),*) ($($modifier:tt),*) ($safe_type:ty) ($unsafe_type:ty)) => {
        impl_dispatchable!(@impl_core ($($nm : $ty),*) ($($modifier),*) ($safe_type));
        impl_dispatchable!(@impl_core ($($nm : $ty),*) ($($modifier),*) ($unsafe_type));
    };

    (@impl_core ($($nm:ident : $ty:ident),*) ($($modifier:tt),*) ($fn_type:ty)) => {
        ::paste::item! {
            impl<Ret: Copy + 'static + Default, $($ty: 'static + std::fmt::Debug ),*> Dispatchable for $fn_type {
                type Func = $fn_type;

                fn dispatcher<S: ScheduleLabel + Default + AsRef<(dyn ScheduleLabel + 'static)>>() -> Self::Func {
                    $($modifier) * fn __disp<S: ScheduleLabel + Default + AsRef<(dyn ScheduleLabel + 'static)>, Ret: Copy + 'static + Default, $($ty: 'static + std::fmt::Debug ),*>($($nm : $ty),*) -> Ret {
                        let scoped_world = unsafe { GLOBAL_APP.get_mut().expect("GLOBAL_APP cell should NOT be empty").clone() };
                        let world = &mut scoped_world.lock().unwrap().world;
                        world.insert_non_send_resource([<DispIn $($ty)* >]::<S, $($ty),*>::new($($nm),*));
                        world.init_non_send_resource::<DispOut<S, Ret>>();
                        world.run_schedule(S::default());
                        world.get_non_send_resource::<DispOut<S, Ret>>().unwrap().ret
                    }
                    __disp::<S, Ret, $($ty),*>
                }
            }
        }
    };

    ($($nm:ident : $ty:ident),*) => {
        impl_dispatchable!(@recurse ($($nm : $ty),*) ());
    };
}

impl_dispatchable! {
    __arg_0:  A, __arg_1:  B, __arg_2:  C, __arg_3:  D, __arg_4:  E, __arg_5:  F, __arg_6:  G,
    __arg_7:  H, __arg_8:  I, __arg_9:  J, __arg_10: K, __arg_11: L, __arg_12: M, __arg_13: N
}
