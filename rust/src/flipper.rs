use godot::classes::{IRigidBody3D, Input, RigidBody3D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
pub struct Flipper {
    #[export]
    input_action: StringName,

    #[export]
    torque: f32,

    #[export]
    torque_hold: f32,

    /// How quickly the hold torque ramps up to full, in units per second.
    /// Higher values reach full torque faster; lower values feel more gradual.
    #[export]
    torque_accel: f32,

    #[export]
    reverse_direction: bool,

    /// Current ramp factor (0.0..=1.0) applied to the hold torque.
    torque_ramp: f32,

    /// Tracks the previous input state so the ramp can reset when direction flips.
    was_pressed: bool,

    base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Flipper {
    fn init(base: Base<RigidBody3D>) -> Self {
        Self {
            input_action: StringName::default(),
            torque: 800.0,
            torque_hold: 8000.0,
            torque_accel: 1.0,
            reverse_direction: false,
            torque_ramp: 0.0,
            was_pressed: false,
            base,
        }
    }

    // fn ready(&mut self) {

    // }

    fn physics_process(&mut self, delta: f64) {
        let just_pressed = !self.input_action.is_empty()
            && Input::singleton().is_action_just_pressed(&self.input_action);
        let pressed = !self.input_action.is_empty()
            && Input::singleton().is_action_pressed(&self.input_action);

        let torque_basis = self.base().get_global_basis();
        let torque = self.torque;
        let torque_ramp = self.torque_ramp;

        // Reset and climb again whenever the direction flips, so torque always
        // eases in from zero toward full strength in the new direction.
        if pressed != self.was_pressed {
            self.torque_ramp = 0.0;
        }
        self.was_pressed = pressed;

        if torque_ramp != 1.0 {
            godot_print!("torque_ramp: {}", self.torque_ramp);
        }

        if just_pressed {
            let local_axis = match self.reverse_direction {
                true => Vector3::DOWN,
                false => Vector3::UP,
            };

            self.base_mut()
                .apply_torque_impulse(torque_basis * local_axis * torque * torque_ramp); // .apply_torque(torque_axis * torque);
        }

        let local_axis = match (pressed, self.reverse_direction) {
            (true, false) | (false, true) => Vector3::UP,
            (false, false) | (true, true) => Vector3::DOWN,
        };

        let ramp_step = self.torque_accel * delta as f32;
        self.torque_ramp = (self.torque_ramp + ramp_step).min(1.0);

        // `torque_hold` is the max torque applied to the rotation; the ramp eases
        // it in from zero so it builds up toward that maximum.
        let torque_hold = self.torque_hold;
        let torque_ramp = self.torque_ramp;
        self.base_mut()
            .apply_torque(torque_basis * local_axis * torque_hold * torque_ramp)
    }
}
