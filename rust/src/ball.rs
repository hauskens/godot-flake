use std::fmt::Display;
use std::ops::Deref;

use godot::classes::{
    Area3D, BoxShape3D, CollisionShape3D, INode3D, IRigidBody3D, InputEvent, Node3D, PackedScene,
    RigidBody3D, TextureProgressBar, Timer,
};
use godot::prelude::*;

const RESET_BALL_ACTION: &str = "reset_ball";

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

    fn ready(&mut self) {
        let timer = self.base().get_node_as::<Timer>("Timer");
        timer.signals().timeout().connect_other(self, Self::kill);
    }
}

#[godot_api]
impl Ball {
    #[signal]
    fn killed();

    pub fn kill(&mut self) {
        godot_print!("Ball killed");
        self.signals().killed().emit();
        self.base_mut().queue_free();
    }
    pub fn _boost(&mut self, force: f32) {
        self.base_mut().apply_impulse(Vector3::FORWARD * force);
    }
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct BallSpawnPoint {
    base: Base<Node3D>,
}

#[derive(GodotClass)]
#[class(tool,init, base=Node3D)]
pub struct Plunger {
    #[export]
    ball_scene: OnEditor<Gd<PackedScene>>,

    #[var(get, set)]
    #[export]
    safe_zone: Aabb,

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
    pending_ball: Option<Gd<Ball>>,
    active_ball: Option<Gd<Ball>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for Plunger {
    fn input(&mut self, event: Gd<InputEvent>) {
        if !self.can_charge() {
            return;
        }
        if event.is_action_pressed(&StringName::from(RESET_BALL_ACTION)) {
            godot_print!("Reset ball");
            self.charge = self.charge_min + 1.0;
            if self.pending_ball.is_none() {
                self.spawn_ball();
            }
            if let Some(ball) = self.pending_ball.as_mut() {
                ball.set_freeze_enabled(true);
            }
        }
        if event.is_action_released(&StringName::from(RESET_BALL_ACTION)) {
            godot_print!("Reset ball released");
            if self.charge >= self.charge_min {
                self.shoot();
            }
        }
    }
    fn process(&mut self, _delta: f64) {
        if self.can_charge() && self.charge != self.charge_min {
            self.charge =
                (self.charge + self.charge_rate * _delta).clamp(self.charge_min, self.charge_max);
            self.base()
                .get_node_as::<TextureProgressBar>("ProgressBarMesh/Viewport/ProgressBar")
                .set_value(self.charge);
        }
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
        self.base()
            .get_node_as::<Area3D>("SafeZone")
            .signals()
            .body_exited()
            .connect_other(self, Self::ball_exited_safe_zone);
    }
}

#[godot_api]
impl Plunger {
    #[func]
    fn set_safe_zone(&mut self, safe_zone: Aabb) {
        self.safe_zone = safe_zone;
        let mut shape = BoxShape3D::new_gd();
        shape.set_size(safe_zone.size);
        let mut collision = self
            .base()
            .get_node_as::<CollisionShape3D>("SafeZone/Collision");
        collision.set_shape(&shape);
        collision.set_position(safe_zone.position);
    }

    #[func]
    fn get_safe_zone(&self) -> Aabb {
        self.safe_zone
    }

    fn can_charge(&self) -> bool {
        if self.active_ball.is_some() {
            return false;
        }
        match self.pending_ball.as_ref() {
            Some(ball) => ball.get_linear_velocity().length_squared() < 0.01,
            None => true,
        }
    }

    fn ball_exited_safe_zone(&mut self, body: Gd<Node3D>) {
        let Ok(ball) = body.try_cast::<Ball>() else {
            return;
        };
        let Some(pending) = self.pending_ball.as_ref() else {
            return;
        };
        if pending.instance_id() != ball.instance_id() {
            return;
        }
        godot_print!("Ball exited safe zone — promoting to active");
        self.active_ball = self.pending_ball.take();
    }

    fn spawn_ball(&mut self) {
        let spawn_point = self.base().get_node_as::<BallSpawnPoint>("BallSpawnPoint");
        let mut ball = self.ball_scene.instantiate_as::<Ball>();
        self.pending_ball = Some(ball.clone());
        ball.set_position(spawn_point.get_position());
        self.base_mut().add_child(&ball);
        ball.signals()
            .killed()
            .connect_other(self, Self::destroy_ball);
    }

    fn shoot(&mut self) {
        let strength = ShotStrength::from(self.deref());
        godot_print!("Strength: {}", strength);
        let impulse: Vector3 = strength.into();

        self.charge = self.charge_min;
        self.base()
            .get_node_as::<TextureProgressBar>("ProgressBarMesh/Viewport/ProgressBar")
            .set_value(0.0);

        let Some(mut ball) = self.pending_ball.clone() else {
            return;
        };
        ball.set_freeze_enabled(false);
        ball.set_linear_velocity(Vector3::ZERO);
        ball.apply_impulse(impulse);
    }

    fn destroy_ball(&mut self) {
        self.pending_ball = None;
        self.active_ball = None;
        godot_print!("Ball destroyed");
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
