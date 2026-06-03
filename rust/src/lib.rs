mod ball;
mod flipper;
//mod gm;

use derive_more::{Add, AddAssign, Display, From, Into, Mul, Sub};
use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(
    GodotConvert,
    Var,
    Export,
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    PartialOrd,
    Display,
    From,
    Into,
    Add,
    Sub,
    Mul,
    AddAssign,
)]
#[godot(transparent)]
pub struct Degrees(pub f32);

impl Degrees {
    pub fn to_radians(self) -> f32 {
        self.0.to_radians()
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    pub fn signum(self) -> f32 {
        self.0.signum()
    }
}
