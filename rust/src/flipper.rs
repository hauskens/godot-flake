use godot::classes::hinge_joint_3d::Param as HingeParam;
use godot::classes::{HingeJoint3D, IRigidBody3D, Input, RigidBody3D};
use godot::prelude::*;

use crate::Degrees;

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
pub struct Flipper {
    /// Cap on motor target angular velocity (deg/s).
    #[export]
    max_speed_deg: Degrees,
    /// Angle the flipper rests at when the input is not pressed.
    #[export]
    rest_angle_deg: Degrees,
    /// Angle the flipper drives toward while the input is held.
    #[export]
    active_angle_deg: Degrees,
    /// Proportional gain: motor velocity = k_p * angle_error, clamped to max_speed.
    #[export]
    k_p: f32,
    #[export]
    input_action: StringName,

    rest_self_in_hinge: Transform3D,

    base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Flipper {
    fn init(base: Base<RigidBody3D>) -> Self {
        Self {
            max_speed_deg: Degrees(1800.0),
            rest_angle_deg: Degrees(-15.0),
            active_angle_deg: Degrees(15.0),
            k_p: 30.0,
            input_action: StringName::default(),
            rest_self_in_hinge: Transform3D::IDENTITY,
            base,
        }
    }

    fn ready(&mut self) {
        let hinge = self.base().get_node_as::<HingeJoint3D>("HingeJoint3D");
        self.rest_self_in_hinge =
            hinge.get_global_transform().affine_inverse() * self.base().get_global_transform();
    }

    fn physics_process(&mut self, _delta: f64) {
        let pressed = !self.input_action.is_empty()
            && Input::singleton().is_action_pressed(&self.input_action);
        let target_deg = if pressed {
            self.active_angle_deg.0
        } else {
            self.rest_angle_deg.0
        };

        let hinge = self.base().get_node_as::<HingeJoint3D>("HingeJoint3D");
        let self_in_hinge =
            hinge.get_global_transform().affine_inverse() * self.base().get_global_transform();
        // Godot HingeJoint3D rotates around its local Z; extract that component
        // of the body's rotation relative to its rest pose in the hinge frame.
        let rel = self_in_hinge.basis * self.rest_self_in_hinge.basis.inverse();
        let angle_rad = rel.col_a().y.atan2(rel.col_a().x);
        let current_deg = angle_rad.to_degrees();

        let error_deg = target_deg - current_deg;
        let max = self.max_speed_deg.0;
        let target_vel_deg = (self.k_p * error_deg).clamp(-max, max);
        let target_vel_rad = target_vel_deg.to_radians();

        let mut hinge = self.base().get_node_as::<HingeJoint3D>("HingeJoint3D");
        hinge.set_param(HingeParam::MOTOR_TARGET_VELOCITY, target_vel_rad);
    }
}
