use std::fmt::Display;
use std::ops::Deref;

use crate::ball::Ball;

use godot::classes::{
    INode3D, IRigidBody3D, Input, Node3D, PackedScene, RigidBody3D, TextureProgressBar, Timer,
};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GameManager {
    ball_tracker: Array<Gd<Ball>>,
    base: Base<Node>,
}
