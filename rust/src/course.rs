use godot::classes::{
    AnimatableBody2D, Area2D, CollisionShape2D, IAnimatableBody2D, IArea2D, INode2D, IStaticBody2D,
    RectangleShape2D, RigidBody2D, StaticBody2D,
};
use godot::prelude::*;

use crate::ball::{Ball, BALL_DAMP};

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

/// Sand/rough patch: drags the ball down hard while it rolls through.
#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Rough {
    #[export]
    #[init(val = Vector2::new(200.0, 150.0))]
    size: Vector2,
    #[export]
    #[init(val = Color::from_rgb(0.78, 0.70, 0.45))]
    color: Color,
    /// Linear damping applied while the ball is inside.
    #[export]
    #[init(val = 4.5)]
    damp: f32,
    base: Base<Area2D>,
}

#[godot_api]
impl Rough {
    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        if let Ok(ball) = body.try_cast::<Ball>() {
            ball.upcast::<RigidBody2D>().set_linear_damp(self.damp);
        }
    }

    #[func]
    fn on_body_exited(&mut self, body: Gd<Node2D>) {
        if let Ok(ball) = body.try_cast::<Ball>() {
            ball.upcast::<RigidBody2D>().set_linear_damp(BALL_DAMP);
        }
    }
}

#[godot_api]
impl IArea2D for Rough {
    fn ready(&mut self) {
        let mut shape = RectangleShape2D::new_gd();
        shape.set_size(self.size);
        let mut collider = CollisionShape2D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);

        let entered = self.to_gd().callable("on_body_entered");
        let exited = self.to_gd().callable("on_body_exited");
        self.base_mut().connect("body_entered", &entered);
        self.base_mut().connect("body_exited", &exited);
    }

    fn draw(&mut self) {
        let rect = Rect2::new(-self.size * 0.5, self.size);
        let color = self.color;
        self.base_mut().draw_rect(rect, color);
    }
}

/// Ice sheet: near-zero rolling friction while the ball is on it, so shots
/// slide way past where they'd stop on felt.
#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Ice {
    #[export]
    #[init(val = Vector2::new(300.0, 200.0))]
    size: Vector2,
    #[export]
    #[init(val = Color::from_rgba(0.45, 0.68, 0.92, 0.9))]
    color: Color,
    /// Linear damping while sliding; keep well under BALL_DAMP.
    #[export]
    #[init(val = 0.12)]
    damp: f32,
    base: Base<Area2D>,
}

#[godot_api]
impl Ice {
    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        if let Ok(ball) = body.try_cast::<Ball>() {
            ball.upcast::<RigidBody2D>().set_linear_damp(self.damp);
        }
    }

    #[func]
    fn on_body_exited(&mut self, body: Gd<Node2D>) {
        if let Ok(ball) = body.try_cast::<Ball>() {
            ball.upcast::<RigidBody2D>().set_linear_damp(BALL_DAMP);
        }
    }
}

#[godot_api]
impl IArea2D for Ice {
    fn ready(&mut self) {
        let mut shape = RectangleShape2D::new_gd();
        shape.set_size(self.size);
        let mut collider = CollisionShape2D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);

        let entered = self.to_gd().callable("on_body_entered");
        let exited = self.to_gd().callable("on_body_exited");
        self.base_mut().connect("body_entered", &entered);
        self.base_mut().connect("body_exited", &exited);
    }

    fn draw(&mut self) {
        let rect = Rect2::new(-self.size * 0.5, self.size);
        let color = self.color;
        self.base_mut().draw_rect(rect, color);
        // Diagonal sheen streaks so ice reads as slick, not just blue felt.
        let streak = Color::from_rgba(1.0, 1.0, 1.0, 0.4);
        let h = self.size.y * 0.3;
        for i in 0..3 {
            let off = (i as f32 - 1.0) * self.size.x * 0.27;
            let a = Vector2::new(off - 26.0, h);
            let b = Vector2::new(off + 26.0, -h);
            self.base_mut().draw_line_ex(a, b, streak).width(5.0).done();
        }
    }
}

/// Quicksand: pulls a rolling ball toward its center and, once it slows
/// down, swallows it — one-stroke penalty and replay from where the stroke
/// was played. Chip over it, or cross with plenty of speed.
#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Quicksand {
    #[export]
    #[init(val = Vector2::new(240.0, 180.0))]
    size: Vector2,
    #[export]
    #[init(val = Color::from_rgb(0.60, 0.48, 0.28))]
    color: Color,
    /// Suction force toward the pool center while the ball wades through.
    #[export]
    #[init(val = 650.0)]
    pull: f32,
    /// Extra velocity-proportional drag (like added linear damp).
    #[export]
    #[init(val = 2.2)]
    drag: f32,
    /// Below this speed (px/s) the sand wins and eats the ball.
    #[export]
    #[init(val = 150.0)]
    sink_speed: f32,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Quicksand {
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
        // Swirl arcs spiraling toward a dark middle: this pit is hungry.
        let dark = Color::from_rgba(0.2, 0.13, 0.05, 0.5);
        let max_r = self.size.x.min(self.size.y) * 0.5 - 10.0;
        let mut r = max_r;
        let mut start = 0.0f32;
        while r > 14.0 {
            self.base_mut()
                .draw_arc_ex(Vector2::ZERO, r, start, start + 4.6, 24, dark)
                .width(4.0)
                .done();
            start += 1.9;
            r -= max_r * 0.3;
        }
        self.base_mut()
            .draw_circle(Vector2::ZERO, 10.0, Color::from_rgb(0.25, 0.17, 0.07));
    }

    fn physics_process(&mut self, _delta: f64) {
        let center = self.base().get_global_position();
        for body in self.base().get_overlapping_bodies().iter_shared() {
            let Ok(mut ball) = body.try_cast::<Ball>() else {
                continue;
            };
            // Leave frozen (mid-swallow) and resting balls alone: forces on
            // them would fling the ball when it unfreezes / never decay.
            if ball.bind().mid_swallow() || ball.bind().is_stopped() {
                continue;
            }
            let mut rb = ball.clone().upcast::<RigidBody2D>();
            let vel = rb.get_linear_velocity();
            if vel.length() < self.sink_speed && ball.bind_mut().swallow(center) {
                continue;
            }
            let to_center = center - rb.get_global_position();
            let suction = if to_center.length() > 1.0 {
                to_center.normalized() * self.pull
            } else {
                Vector2::ZERO
            };
            rb.apply_central_force(suction - vel * self.drag);
        }
    }
}

/// Rotating bar hazard: spins about the node origin and bats the ball
/// around; time your shot past it.
#[derive(GodotClass)]
#[class(init, base=AnimatableBody2D)]
pub struct Spinner {
    #[export]
    #[init(val = Vector2::new(200.0, 22.0))]
    size: Vector2,
    /// Angular speed in radians/second; negative spins the other way.
    #[export]
    #[init(val = 1.8)]
    speed: f32,
    #[export]
    #[init(val = Color::from_rgb(0.33, 0.52, 0.27))]
    color: Color,
    base: Base<AnimatableBody2D>,
}

#[godot_api]
impl IAnimatableBody2D for Spinner {
    fn ready(&mut self) {
        let mut shape = RectangleShape2D::new_gd();
        shape.set_size(self.size);
        let mut collider = CollisionShape2D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);
    }

    fn physics_process(&mut self, delta: f64) {
        let step = self.speed * delta as f32;
        self.base_mut().rotate(step);
    }

    fn draw(&mut self) {
        let rect = Rect2::new(-self.size * 0.5, self.size);
        let color = self.color;
        let hub = Color::from_rgb(0.18, 0.28, 0.15);
        let hub_r = self.size.y * 0.8;
        self.base_mut().draw_rect(rect, color);
        self.base_mut().draw_circle(Vector2::ZERO, hub_r, hub);
    }
}

/// Water hazard: a rolling ball that touches it takes a one-stroke penalty
/// and is returned to where the stroke was played. Airborne chips fly over.
#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Water {
    #[export]
    #[init(val = Vector2::new(220.0, 120.0))]
    size: Vector2,
    #[export]
    #[init(val = Color::from_rgb(0.16, 0.38, 0.62))]
    color: Color,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Water {
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
        let ripple = Color::from_rgba(1.0, 1.0, 1.0, 0.25);
        self.base_mut().draw_rect(rect, color);
        // A few ripple dashes so water reads differently from felt.
        let w = self.size.x * 0.22;
        for (dx, dy) in [(-0.25, -0.2), (0.15, 0.05), (-0.1, 0.3)] {
            let center = Vector2::new(self.size.x * dx as f32, self.size.y * dy as f32);
            self.base_mut()
                .draw_line_ex(
                    center - Vector2::new(w * 0.5, 0.0),
                    center + Vector2::new(w * 0.5, 0.0),
                    ripple,
                )
                .width(3.0)
                .done();
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        for body in self.base().get_overlapping_bodies().iter_shared() {
            let Ok(mut ball) = body.try_cast::<Ball>() else {
                continue;
            };
            ball.bind_mut().splash();
        }
    }
}

/// Boost pad: fires the ball along the pad's +X axis (rotate the node to
/// aim it) once per entry.
#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct SpeedPad {
    #[export]
    #[init(val = Vector2::new(90.0, 90.0))]
    size: Vector2,
    /// Speed (px/s) added along the pad's facing when the ball rolls on.
    #[export]
    #[init(val = 700.0)]
    boost: f32,

    pending: Option<Gd<Ball>>,
    base: Base<Area2D>,
}

#[godot_api]
impl SpeedPad {
    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        if let Ok(ball) = body.try_cast::<Ball>() {
            self.pending = Some(ball);
        }
    }
}

#[godot_api]
impl IArea2D for SpeedPad {
    fn ready(&mut self) {
        let mut shape = RectangleShape2D::new_gd();
        shape.set_size(self.size);
        let mut collider = CollisionShape2D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);

        let entered = self.to_gd().callable("on_body_entered");
        self.base_mut().connect("body_entered", &entered);
    }

    fn draw(&mut self) {
        let rect = Rect2::new(-self.size * 0.5, self.size);
        let base_color = Color::from_rgba(0.15, 0.65, 0.75, 0.9);
        let chevron = Color::from_rgba(1.0, 1.0, 1.0, 0.85);
        let h = self.size.y * 0.28;
        self.base_mut().draw_rect(rect, base_color);
        for offset in [-14.0, 14.0] {
            let tip = Vector2::new(offset + 14.0, 0.0);
            let top = Vector2::new(offset - 14.0, -h);
            let bottom = Vector2::new(offset - 14.0, h);
            self.base_mut()
                .draw_line_ex(top, tip, chevron)
                .width(6.0)
                .done();
            self.base_mut()
                .draw_line_ex(bottom, tip, chevron)
                .width(6.0)
                .done();
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        let Some(ball) = self.pending.take() else {
            return;
        };
        let dir = self.base().get_global_transform().a.normalized();
        let mut rb = ball.upcast::<RigidBody2D>();
        let vel = rb.get_linear_velocity();
        rb.set_linear_velocity(vel + dir * self.boost);
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
