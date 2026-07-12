use godot::classes::{
    CircleShape2D, CollisionShape2D, IRigidBody2D, InputEvent, InputEventMouseButton, Line2D,
    PhysicsMaterial, RigidBody2D,
};
use godot::global::MouseButton;
use godot::prelude::*;

/// Baseline rolling friction; `Rough` patches raise damping and restore this.
pub const BALL_DAMP: f32 = 1.1;

/// The golf ball. Click near it while it's stopped, drag away to aim
/// (slingshot style), release to putt.
#[derive(GodotClass)]
#[class(init, base=RigidBody2D)]
pub struct Ball {
    /// Visual + collision radius in pixels.
    #[export]
    #[init(val = 12.0)]
    radius: f32,
    /// How close (px) a click must be to the ball to start aiming.
    #[export]
    #[init(val = 60.0)]
    grab_radius: f32,
    /// Maximum drag distance (px); caps shot power.
    #[export]
    #[init(val = 180.0)]
    max_drag: f32,
    /// Impulse applied per pixel of drag.
    #[export]
    #[init(val = 7.0)]
    power: f32,
    /// Below this speed (px/s) the ball is considered stopped.
    #[export]
    #[init(val = 10.0)]
    stop_speed: f32,

    aiming: bool,
    /// True while the current drag is a chip (right-button) aim.
    aim_chip: bool,
    sunk: bool,
    stopped: bool,
    still_time: f32,
    /// Chip flight state: seconds remaining / total flight time.
    air_left: f32,
    air_total: f32,
    aim_line: Option<Gd<Line2D>>,
    base: Base<RigidBody2D>,
}

#[godot_api]
impl Ball {
    /// Emitted every time the player putts.
    #[signal]
    fn stroke_taken();

    /// Emitted when the ball comes to rest after a putt.
    #[signal]
    fn ball_stopped();

    /// Called by the cup when it captures the ball.
    #[func]
    pub fn sink(&mut self) {
        self.sunk = true;
        self.aiming = false;
        self.base_mut().set_visible(false);
        self.base_mut().set_freeze_enabled(true);
    }

    /// Programmatic stroke (used by the Caddy bridge). `power` is 0..1 of
    /// the maximum drag; counts as a normal stroke. Returns false if the
    /// ball isn't ready to be hit.
    #[func]
    pub fn putt(&mut self, dir: Vector2, power: f32) -> bool {
        if self.sunk || !self.stopped || dir.length() < 0.001 {
            return false;
        }
        let impulse = dir.normalized() * (power.clamp(0.0, 1.0) * self.max_drag * self.power);
        self.base_mut().apply_impulse(impulse);
        self.stopped = false;
        self.still_time = 0.0;
        self.signals().stroke_taken().emit();
        true
    }

    /// Lofted stroke: same aiming as `putt`, but the ball flies over walls,
    /// sand, and the cup for a power-scaled time before landing and rolling.
    #[func]
    pub fn chip(&mut self, dir: Vector2, power: f32) -> bool {
        if self.sunk || !self.stopped || dir.length() < 0.001 {
            return false;
        }
        let power = power.clamp(0.0, 1.0);
        let impulse = dir.normalized() * (power * self.max_drag * self.power);
        self.base_mut().apply_impulse(impulse);
        self.air_total = 0.22 + 0.38 * power;
        self.air_left = self.air_total;
        self.base_mut().set_collision_layer(0);
        self.base_mut().set_collision_mask(0);
        self.stopped = false;
        self.still_time = 0.0;
        self.signals().stroke_taken().emit();
        true
    }

    fn airborne(&self) -> bool {
        self.air_left > 0.0
    }

    fn land(&mut self) {
        self.air_left = 0.0;
        self.base_mut().set_collision_layer(1);
        self.base_mut().set_collision_mask(1);
        self.base_mut().set_linear_damp(BALL_DAMP);
        self.base_mut().queue_redraw();
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn is_sunk(&self) -> bool {
        self.sunk
    }

    fn shot_vector(&self) -> Vector2 {
        let pos = self.base().get_global_position();
        let mouse = self.base().get_global_mouse_position();
        let mut v = pos - mouse;
        if v.length() > self.max_drag {
            v = v.normalized() * self.max_drag;
        }
        v
    }
}

#[godot_api]
impl IRigidBody2D for Ball {
    fn ready(&mut self) {
        let mut shape = CircleShape2D::new_gd();
        shape.set_radius(self.radius);
        let mut collider = CollisionShape2D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);

        let mut mat = PhysicsMaterial::new_gd();
        mat.set_bounce(0.8);
        mat.set_friction(0.1);
        self.base_mut().set_physics_material_override(&mat);

        // Rolling friction is faked with damping; rotation is irrelevant
        // for a drawn circle and would rotate child nodes.
        self.base_mut().set_lock_rotation_enabled(true);
        self.base_mut().set_linear_damp(BALL_DAMP);
        // No CCD: Godot 2D's ray-cast CCD kills velocity on fast impacts
        // (~3% retention instead of the bounce coefficient). Tunneling is
        // prevented by the 120Hz physics tick in project.godot instead.

        let mut line = Line2D::new_alloc();
        line.set_width(4.0);
        line.set_default_color(Color::from_rgba(1.0, 0.9, 0.3, 0.9));
        line.set_visible(false);
        self.base_mut().add_child(&line);
        self.aim_line = Some(line);

        self.stopped = true;
    }

    fn draw(&mut self) {
        let mut radius = self.radius;
        if self.airborne() {
            // Swell mid-flight to sell the height.
            let progress = 1.0 - self.air_left / self.air_total;
            radius *= 1.0 + 0.9 * (std::f32::consts::PI * progress).sin();
            let shadow = Color::from_rgba(0.0, 0.0, 0.0, 0.35);
            self.base_mut()
                .draw_circle(Vector2::new(6.0, 10.0), radius * 0.8, shadow);
        }
        self.base_mut()
            .draw_circle(Vector2::ZERO, radius, Color::from_rgb(0.95, 0.95, 0.9));
    }

    fn process(&mut self, _delta: f64) {
        if self.aiming {
            let shot = self.shot_vector();
            if let Some(line) = &mut self.aim_line {
                line.set_visible(true);
                line.clear_points();
                line.add_point(Vector2::ZERO);
                line.add_point(shot);
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if self.sunk || self.stopped {
            return;
        }
        if self.airborne() {
            self.air_left -= delta as f32;
            // Rough's body_exited handler restores rolling damp when the
            // collision layer clears; keep flight friction-free regardless.
            self.base_mut().set_linear_damp(0.0);
            self.base_mut().queue_redraw();
            if self.air_left <= 0.0 {
                self.land();
            }
            return;
        }
        if self.base().get_linear_velocity().length() < self.stop_speed {
            self.still_time += delta as f32;
            if self.still_time > 0.15 {
                self.base_mut().set_linear_velocity(Vector2::ZERO);
                self.stopped = true;
                self.signals().ball_stopped().emit();
            }
        } else {
            self.still_time = 0.0;
        }
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if self.sunk {
            return;
        }
        let Ok(button) = event.try_cast::<InputEventMouseButton>() else {
            return;
        };
        let index = button.get_button_index();
        if index != MouseButton::LEFT && index != MouseButton::RIGHT {
            return;
        }
        if button.is_pressed() {
            let pos = self.base().get_global_position();
            let mouse = self.base().get_global_mouse_position();
            if self.stopped && !self.aiming && mouse.distance_to(pos) <= self.grab_radius {
                self.aiming = true;
                self.aim_chip = index == MouseButton::RIGHT;
                let color = if self.aim_chip {
                    Color::from_rgba(1.0, 0.55, 0.2, 0.9) // orange: chip
                } else {
                    Color::from_rgba(1.0, 0.9, 0.3, 0.9) // yellow: putt
                };
                if let Some(line) = &mut self.aim_line {
                    line.set_default_color(color);
                }
            }
        } else if self.aiming {
            self.aiming = false;
            if let Some(line) = &mut self.aim_line {
                line.set_visible(false);
            }
            let shot = self.shot_vector();
            if shot.length() > 2.0 {
                let power = shot.length() / self.max_drag;
                if self.aim_chip {
                    self.chip(shot, power);
                } else {
                    self.putt(shot, power);
                }
            }
        }
    }
}
