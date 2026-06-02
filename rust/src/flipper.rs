use godot::classes::{IRigidBody3D, Input, RigidBody3D};
use godot::prelude::*;

use crate::Degrees;

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
pub struct Flipper {
    #[export]
    input_action: StringName,

    #[export]
    torque: f32,

    #[export]
    reverse_direction: bool,

    base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Flipper {
    fn init(base: Base<RigidBody3D>) -> Self {
        Self {
            input_action: StringName::default(),
            torque: 100.0,
            reverse_direction: false,
            base,
        }
    }

    // fn ready(&mut self) {

    // }

    fn physics_process(&mut self, _delta: f64) {
        let pressed = !self.input_action.is_empty()
            && Input::singleton().is_action_pressed(&self.input_action);

        let torque = self.torque;
        // The hinge rotates the flipper about its local Y axis; apply_torque works in
        // global space, so rotate that local axis into world space.
        let local_axis = match (pressed, self.reverse_direction) {
            (true, false) | (false, true) => Vector3::UP,
            (false, false) | (true, true) => Vector3::DOWN,
        };
        let torque_axis = self.base().get_global_basis() * local_axis;

        self.base_mut().apply_torque(torque_axis * torque);
    }
}
