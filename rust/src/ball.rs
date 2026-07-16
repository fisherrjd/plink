use godot::classes::physics_server_2d::BodyState;
use godot::classes::{
    CircleShape2D, CollisionShape2D, IRigidBody2D, InputEvent, InputEventMouseButton, Line2D,
    PhysicsMaterial, PhysicsServer2D, RigidBody2D,
};
use godot::global::MouseButton;
use godot::prelude::*;

/// Baseline rolling friction; `Rough` patches raise damping and restore this.
pub const BALL_DAMP: f32 = 1.1;

/// Seconds the quicksand swallow animation takes.
const SWALLOW_TIME: f32 = 0.7;
/// Pause between vanishing into the sand and reappearing at the shot origin.
const RESPAWN_DELAY: f32 = 0.45;

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
    /// Where the last stroke was played from; water hazards reset here.
    last_stroke_pos: Vector2,
    /// Quicksand swallow animation: seconds remaining (0 = inactive), the
    /// grab point, and the pool center the ball is dragged toward.
    swallow_t: f32,
    swallow_from: Vector2,
    swallow_center: Vector2,
    /// Countdown to reappearing at the shot origin after being swallowed.
    respawn_t: f32,
    /// Draw-scale factor; shrinks to zero while the sand pulls the ball under.
    #[init(val = 1.0)]
    sink_scale: f32,
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
        self.last_stroke_pos = self.base().get_global_position();
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
        self.last_stroke_pos = self.base().get_global_position();
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

    /// Water hazard: one penalty stroke, then replay from where the last
    /// stroke was taken. Only a rolling ball can splash — the reset sets
    /// `stopped`, which also debounces the area-overlap frames that follow.
    pub fn splash(&mut self) {
        if self.sunk || self.stopped || self.airborne() {
            return;
        }
        self.signals().stroke_taken().emit();
        let reset = self.last_stroke_pos;
        self.teleport_to(reset);
    }

    /// Quicksand: the ball is dragged under (shrinking toward `center`),
    /// then respawns where the last stroke was played, with a one-stroke
    /// penalty. Returns false if the ball can't be swallowed right now.
    /// Like `splash`, only a rolling ball can be eaten — the respawn sets
    /// `stopped`, which debounces the stale area-overlap frames after the
    /// teleport (without it the pool re-swallows the ball from the tee,
    /// forever).
    pub fn swallow(&mut self, center: Vector2) -> bool {
        if self.sunk || self.stopped || self.airborne() || self.mid_swallow() {
            return false;
        }
        self.aiming = false;
        if let Some(line) = &mut self.aim_line {
            line.set_visible(false);
        }
        self.stopped = false;
        self.swallow_t = SWALLOW_TIME;
        self.swallow_from = self.base().get_global_position();
        self.swallow_center = center;
        // Zero velocity before freezing so unfreezing can't restore it.
        self.base_mut().set_linear_velocity(Vector2::ZERO);
        // Freeze so the sink animation owns the transform.
        self.base_mut().set_freeze_enabled(true);
        self.signals().stroke_taken().emit();
        true
    }

    /// True from the moment the sand grabs the ball until it's back in play.
    pub fn mid_swallow(&self) -> bool {
        self.swallow_t > 0.0 || self.respawn_t > 0.0
    }

    /// Hard-teleport through the physics server: a plain set_global_position
    /// on a moving RigidBody2D gets overwritten by the body's integrated
    /// transform at the end of the step.
    fn teleport_to(&mut self, pos: Vector2) {
        let mut transform = self.base().get_global_transform();
        transform.origin = pos;
        let rid = self.base().get_rid();
        let mut server = PhysicsServer2D::singleton();
        server.body_set_state(rid, BodyState::TRANSFORM, &transform.to_variant());
        server.body_set_state(rid, BodyState::LINEAR_VELOCITY, &Vector2::ZERO.to_variant());
        self.base_mut().set_global_position(pos);
        self.base_mut().set_linear_velocity(Vector2::ZERO);
        self.base_mut().set_linear_damp(BALL_DAMP);
        self.stopped = true;
        self.still_time = 0.0;
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

        self.last_stroke_pos = self.base().get_global_position();
        self.stopped = true;
    }

    fn draw(&mut self) {
        let mut radius = self.radius * self.sink_scale;
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
        if self.swallow_t > 0.0 {
            self.swallow_t -= delta as f32;
            let t = (1.0 - self.swallow_t.max(0.0) / SWALLOW_TIME).clamp(0.0, 1.0);
            // Ease in: the sand grips slowly, then drags the ball under.
            let pos = self.swallow_from.lerp(self.swallow_center, t * t);
            self.base_mut().set_global_position(pos);
            self.sink_scale = 1.0 - t;
            self.base_mut().queue_redraw();
            if self.swallow_t <= 0.0 {
                self.base_mut().set_visible(false);
                self.respawn_t = RESPAWN_DELAY;
            }
            return;
        }
        if self.respawn_t > 0.0 {
            self.respawn_t -= delta as f32;
            if self.respawn_t > 0.0 {
                return;
            }
            let reset = self.last_stroke_pos;
            self.base_mut().set_freeze_enabled(false);
            self.teleport_to(reset);
            self.sink_scale = 1.0;
            self.base_mut().set_visible(true);
            self.base_mut().queue_redraw();
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
        // Out of bounds (a chip that cleared the outer walls): treat like
        // water — penalty stroke and replay from the last stroke position.
        let pos = self.base().get_global_position();
        if !(110.0..=1170.0).contains(&pos.x) || !(100.0..=620.0).contains(&pos.y) {
            self.splash();
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
