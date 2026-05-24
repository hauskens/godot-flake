use godot::classes::{INode3D, IRigidBody3D, Input, Node3D, PackedScene, RigidBody3D, Timer};
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

    fn ready(&mut self) {
        let timer = self.base().get_node_as::<Timer>("Timer");
        timer.signals().timeout().connect_other(self, Self::kill);
    }
}

impl Ball {
    fn kill(&mut self) {
        self.base_mut().queue_free();
    }
}

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct BallSpawn {
    #[export]
    ball_scene: Option<Gd<PackedScene>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BallSpawn {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            ball_scene: None,
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        if !Input::singleton().is_action_just_pressed(&StringName::from("reset_ball")) {
            return;
        }

        let Some(scene) = self.ball_scene.as_ref() else {
            godot_warn!("BallSpawn: ball_scene not set");
            return;
        };

        let Some(instance) = scene.instantiate() else {
            godot_warn!("BallSpawn: failed to instantiate ball_scene");
            return;
        };

        let spawn_xform = self.base().get_global_transform();
        let mut parent = self
            .base()
            .get_parent()
            .expect("BallSpawn must have a parent");
        parent.add_child(&instance);

        if let Ok(mut node3d) = instance.try_cast::<Node3D>() {
            node3d.set_global_transform(spawn_xform);

            if let Ok(mut rb) = node3d.try_cast::<RigidBody3D>() {
                rb.set_linear_velocity(Vector3::ZERO);
                rb.set_angular_velocity(Vector3::ZERO);
            }
        }
    }
}
