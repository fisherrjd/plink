use godot::classes::{
    Area2D, CollisionShape2D, IArea2D, INode2D, IStaticBody2D, RectangleShape2D, RigidBody2D,
    StaticBody2D,
};
use godot::prelude::*;

use crate::ball::Ball;

/// A rectangular bumper wall. Size/rotation come from the scene; collision
/// shape and visuals are generated so hole scenes stay tiny.
#[derive(GodotClass)]
#[class(init, base=StaticBody2D)]
pub struct Wall {
    #[export]
    #[init(val = Vector2::new(200.0, 30.0))]
    size: Vector2,
    #[export]
    #[init(val = Color::from_rgb(0.45, 0.31, 0.18))]
    color: Color,
    base: Base<StaticBody2D>,
}

#[godot_api]
impl IStaticBody2D for Wall {
    fn ready(&mut self) {
        let mut shape = RectangleShape2D::new_gd();
        shape.set_size(self.size);
        let mut collider = CollisionShape2D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);
    }

    fn draw(&mut self) {
        let rect = Rect2::new(-self.size * 0.5, self.size);
        let color = self.color;
        self.base_mut().draw_rect(rect, color);
    }
}

/// Decorative green: a plain drawn rectangle, no collision.
#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Felt {
    #[export]
    #[init(val = Vector2::new(900.0, 300.0))]
    size: Vector2,
    #[export]
    #[init(val = Color::from_rgb(0.24, 0.54, 0.28))]
    color: Color,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Felt {
    fn draw(&mut self) {
        let rect = Rect2::new(-self.size * 0.5, self.size);
        let color = self.color;
        self.base_mut().draw_rect(rect, color);
    }
}

/// The cup. Captures the ball when it overlaps while moving slowly enough,
/// so fast shots roll over instead of sinking.
#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Hole {
    #[export]
    #[init(val = 16.0)]
    radius: f32,
    /// Ball must be slower than this (px/s) to drop in.
    #[export]
    #[init(val = 260.0)]
    capture_speed: f32,

    captured: bool,
    base: Base<Area2D>,
}

#[godot_api]
impl Hole {
    /// Emitted once when the ball drops in.
    #[signal]
    fn ball_sunk();
}

#[godot_api]
impl IArea2D for Hole {
    fn ready(&mut self) {
        let mut shape = godot::classes::CircleShape2D::new_gd();
        shape.set_radius(self.radius);
        let mut collider = CollisionShape2D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);
    }

    fn draw(&mut self) {
        let radius = self.radius;
        self.base_mut().draw_circle(
            Vector2::ZERO,
            radius + 3.0,
            Color::from_rgb(0.16, 0.38, 0.19),
        );
        self.base_mut()
            .draw_circle(Vector2::ZERO, radius, Color::from_rgb(0.05, 0.05, 0.06));
    }

    fn physics_process(&mut self, _delta: f64) {
        if self.captured {
            return;
        }
        for body in self.base().get_overlapping_bodies().iter_shared() {
            let Ok(mut ball) = body.try_cast::<Ball>() else {
                continue;
            };
            let rb = ball.clone().upcast::<RigidBody2D>();
            if rb.get_linear_velocity().length() > self.capture_speed {
                continue;
            }
            self.captured = true;
            ball.bind_mut().sink();
            self.signals().ball_sunk().emit();
            break;
        }
    }
}
