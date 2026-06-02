use std::fmt::Display;
use std::ops::Deref;

use godot::classes::{
    INode3D, IRigidBody3D, Input, Node3D, PackedScene, RigidBody3D, TextureProgressBar, Timer,
};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
pub struct Ball {
    base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Ball {
    fn init(base: Base<RigidBody3D>) -> Self {
        Self { base }
    }

    // fn ready(&mut self) {
    //     let timer = self.base().get_node_as::<Timer>("Timer");
    //     timer.signals().timeout().connect_other(self, Self::kill);
    // }
}

impl Ball {
    pub fn kill(&mut self) {
        self.base_mut().queue_free();
    }
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct BallSpawnPoint {
    base: Base<Node3D>,
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct Plunger {
    #[export]
    ball_scene: OnEditor<Gd<PackedScene>>,

    #[export]
    #[init(val = 100.0)]
    force: f32,

    #[export]
    #[init(val = 100.0)]
    charge_rate: f64,

    #[export]
    #[init(val = 100.0)]
    charge_max: f64,

    #[export]
    #[init(val = 0.0)]
    charge_min: f64,

    charge: f64,
    active_balls: Array<Gd<Ball>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for Plunger {
    fn process(&mut self, _delta: f64) {
        if !Input::singleton().is_action_pressed(&StringName::from("reset_ball")) {
            if self.charge > self.charge_min {
                self.shoot();
            }
            return;
        }
        self.charge =
            (self.charge + self.charge_rate * _delta).clamp(self.charge_min, self.charge_max);
        self.base()
            .get_node_as::<TextureProgressBar>("ProgressBarMesh/Viewport/ProgressBar")
            .set_value(self.charge);
    }

    fn ready(&mut self) {
        if !self.ball_scene.can_instantiate() {
            godot_error!(
                "Plunger: failed to instantiate ball_scene. Select a BallScene in the editor.."
            );
            return;
        };
        self.base()
            .get_node_as::<Node3D>("BallSpawnPoint/SpawnPreview")
            .set_visible(false);
        self.base()
            .get_node_as::<TextureProgressBar>("ProgressBarMesh/Viewport/ProgressBar")
            .set_value(0.0);
    }
}

impl Plunger {
    fn shoot(&mut self) {
        let strength = ShotStrength::from(self.deref());
        godot_print!("Strength: {}", strength);

        if let Some(mut ball) = self.active_balls.back() {
            ball.bind_mut().kill();
        }

        // Reset the charge
        self.charge = self.charge_min;

        // Reset the progress bar
        self.base()
            .get_node_as::<TextureProgressBar>("ProgressBarMesh/Viewport/ProgressBar")
            .set_value(0.0);

        // Spawn the ball
        let spawn_point = self.base().get_node_as::<BallSpawnPoint>("BallSpawnPoint");
        let mut ball = self.ball_scene.instantiate_as::<Ball>();
        self.active_balls.push(&ball);
        ball.set_position(spawn_point.get_position());
        self.base_mut().add_child(&ball);
        ball.apply_impulse(strength.into());
    }
}

struct ShotStrength {
    charge: f64,
    max_charge: f64,
    force: f32,
}

impl From<&Plunger> for ShotStrength {
    fn from(plunger: &Plunger) -> Self {
        Self {
            charge: plunger.charge,
            max_charge: plunger.charge_max,
            force: plunger.force,
        }
    }
}

impl Into<Vector3> for ShotStrength {
    fn into(self) -> Vector3 {
        Vector3::FORWARD * self.get_strength()
    }
}

impl ShotStrength {
    fn get_strength(&self) -> f32 {
        (self.charge / self.max_charge) as f32 * self.force
    }
}

impl Display for ShotStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Strength: {} (charge: {}, max_charge: {}, force: {})",
            self.get_strength(),
            self.charge,
            self.max_charge,
            self.force
        )
    }
}
