use std::path::PathBuf;

use godot::classes::{INode, Json, ProjectSettings, RigidBody2D};
use godot::prelude::*;

use crate::ball::Ball;
use crate::course::Hole;
use crate::game::GameManager;

/// File-based control bridge so an external driver (tools/caddy.py) can play
/// the game: publishes state to .caddy/state.json, consumes putt/screenshot
/// commands from .caddy/command.json, and snapshots each hole to course1/.
/// Inert unless the game is launched with PLINK_CADDY=1.
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct Caddy {
    enabled: bool,
    tick: f64,
    last_hole: i64,
    final_shot_done: bool,
    /// Pending delayed screenshots: (seconds remaining, absolute path).
    pending_shots: Vec<(f64, String)>,
    caddy_dir: PathBuf,
    course_dir: PathBuf,
    base: Base<Node>,
}

const TICK: f64 = 0.1;
const SNAP_DELAY: f64 = 0.6;

#[godot_api]
impl INode for Caddy {
    fn ready(&mut self) {
        self.enabled = std::env::var("PLINK_CADDY").is_ok_and(|v| v == "1");
        if !self.enabled {
            return;
        }
        let godot_dir = ProjectSettings::singleton()
            .globalize_path("res://")
            .to_string();
        let repo = PathBuf::from(&godot_dir)
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        self.caddy_dir = repo.join(".caddy");
        self.course_dir = repo.join("course1");
        let _ = std::fs::create_dir_all(&self.caddy_dir);
        let _ = std::fs::create_dir_all(&self.course_dir);
        self.last_hole = -1;
        godot_print!("Caddy bridge active: {}", self.caddy_dir.display());
    }

    fn process(&mut self, delta: f64) {
        if !self.enabled {
            return;
        }

        for shot in &mut self.pending_shots {
            shot.0 -= delta;
        }
        let due: Vec<String> = self
            .pending_shots
            .iter()
            .filter(|(t, _)| *t <= 0.0)
            .map(|(_, p)| p.clone())
            .collect();
        self.pending_shots.retain(|(t, _)| *t > 0.0);
        for path in due {
            self.save_screenshot(&path);
        }

        self.tick += delta;
        if self.tick < TICK {
            return;
        }
        self.tick = 0.0;

        self.schedule_hole_snapshots();
        self.write_state();
        self.handle_command();
    }
}

impl Caddy {
    fn game(&self) -> Option<Gd<GameManager>> {
        self.base().get_parent()?.try_cast::<GameManager>().ok()
    }

    /// Ball and Cup of the currently loaded hole. During a hole swap the old
    /// (queue_freed) instance may still exist, so take the newest child.
    fn current_nodes(&self) -> Option<(Gd<Ball>, Gd<Hole>)> {
        let container = self
            .base()
            .get_parent()?
            .try_get_node_as::<Node2D>("HoleContainer")?;
        let children = container.get_children();
        for i in (0..children.len()).rev() {
            let child = children.at(i);
            if child.is_queued_for_deletion() {
                continue;
            }
            let ball = child.try_get_node_as::<Ball>("Ball");
            let cup = child.try_get_node_as::<Hole>("Cup");
            if let (Some(ball), Some(cup)) = (ball, cup) {
                return Some((ball, cup));
            }
        }
        None
    }

    fn schedule_hole_snapshots(&mut self) {
        let Some(gm) = self.game() else { return };
        let (hole, finished) = {
            let gm = gm.bind();
            (gm.hole_index() as i64, gm.is_finished())
        };
        if finished {
            if !self.final_shot_done {
                self.final_shot_done = true;
                let path = self.course_dir.join("scorecard.png");
                self.pending_shots
                    .push((SNAP_DELAY, path.to_string_lossy().into_owned()));
            }
        } else if hole != self.last_hole {
            self.last_hole = hole;
            let path = self.course_dir.join(format!("hole_{:02}.png", hole + 1));
            self.pending_shots
                .push((SNAP_DELAY, path.to_string_lossy().into_owned()));
        }
    }

    fn write_state(&self) {
        let Some(gm) = self.game() else { return };
        let (hole, strokes, pars, finished) = {
            let gm = gm.bind();
            (
                gm.hole_index(),
                gm.stroke_counts(),
                gm.par_values(),
                gm.is_finished(),
            )
        };
        let ball_json = match self.current_nodes() {
            Some((ball, cup)) => {
                let rb = ball.clone().upcast::<RigidBody2D>();
                let pos = rb.get_global_position();
                let vel = rb.get_linear_velocity();
                let (stopped, sunk) = {
                    let b = ball.bind();
                    (b.is_stopped(), b.is_sunk())
                };
                let cup_pos = cup.upcast::<Node2D>().get_global_position();
                format!(
                    r#""ball":{{"x":{:.1},"y":{:.1},"vx":{:.1},"vy":{:.1},"stopped":{},"sunk":{}}},"cup":{{"x":{:.1},"y":{:.1}}}"#,
                    pos.x, pos.y, vel.x, vel.y, stopped, sunk, cup_pos.x, cup_pos.y
                )
            }
            None => r#""ball":null,"cup":null"#.to_string(),
        };
        let state = format!(
            r#"{{"hole":{},"total_holes":{},"strokes":{:?},"pars":{:?},"strokes_this_hole":{},{},"finished":{}}}"#,
            hole + 1,
            pars.len(),
            strokes,
            pars,
            strokes.get(hole).copied().unwrap_or(0),
            ball_json,
            finished,
        );
        let tmp = self.caddy_dir.join("state.json.tmp");
        let dst = self.caddy_dir.join("state.json");
        if std::fs::write(&tmp, state).is_ok() {
            let _ = std::fs::rename(&tmp, &dst);
        }
    }

    fn handle_command(&mut self) {
        let path = self.caddy_dir.join("command.json");
        let Ok(text) = std::fs::read_to_string(&path) else {
            return;
        };
        let _ = std::fs::remove_file(&path);

        let parsed = Json::parse_string(&GString::from(&text));
        let Ok(dict) = parsed.try_to::<VarDictionary>() else {
            godot_error!("Caddy: bad command JSON: {text}");
            return;
        };

        if let Some(putt) = dict.get("putt") {
            let Ok(putt) = putt.try_to::<VarDictionary>() else {
                return;
            };
            let get = |key: &str| {
                putt.get(key)
                    .and_then(|v| v.try_to::<f64>().ok())
                    .unwrap_or(0.0) as f32
            };
            let dir = Vector2::new(get("dx"), get("dy"));
            let power = get("power");
            if let Some((mut ball, _)) = self.current_nodes() {
                let ok = ball.bind_mut().putt(dir, power);
                godot_print!(
                    "Caddy: putt dir=({:.0},{:.0}) power={power:.2} -> {ok}",
                    dir.x,
                    dir.y
                );
            }
        }

        if let Some(shot) = dict.get("screenshot") {
            if let Ok(path) = shot.try_to::<GString>() {
                self.save_screenshot(&path.to_string());
            }
        }
    }

    fn save_screenshot(&self, path: &str) {
        let Some(viewport) = self.base().get_viewport() else {
            return;
        };
        let Some(texture) = viewport.get_texture() else {
            return;
        };
        let Some(image) = texture.get_image() else {
            return;
        };
        if let Some(parent) = PathBuf::from(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        image.save_png(path);
        godot_print!("Caddy: screenshot -> {path}");
    }
}
