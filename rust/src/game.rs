use godot::classes::{CanvasLayer, Label, Object, PackedScene};
use godot::prelude::*;

use crate::ball::Ball;
use crate::course::Hole;

/// Root of the round: loads hole scenes in sequence, counts strokes,
/// drives the HUD, and shows the scorecard after the last hole.
///
/// Expected children: HoleContainer (Node2D), HUD (CanvasLayer with
/// HoleLabel/ParLabel/StrokeLabel), Scorecard (CanvasLayer with
/// Panel/Results label).
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GameManager {
    #[export]
    hole_scenes: Array<GString>,
    #[export]
    pars: PackedInt32Array,

    current: usize,
    strokes: Vec<i32>,
    finished: bool,
    base: Base<Node>,
}

#[godot_api]
impl GameManager {
    #[func]
    fn on_stroke_taken(&mut self) {
        if let Some(count) = self.strokes.get_mut(self.current) {
            *count += 1;
        }
        self.update_hud();
    }

    #[func]
    fn on_ball_sunk(&mut self) {
        let mut timer = self.base().get_tree().create_timer(0.9);
        timer.connect("timeout", &self.to_gd().callable("advance_hole"));
    }

    #[func]
    fn advance_hole(&mut self) {
        self.current += 1;
        if self.current < self.hole_scenes.len() {
            self.load_hole();
        } else {
            self.show_scorecard();
        }
    }

    #[func]
    fn on_back_to_menu(&mut self) {
        let _ = self
            .base()
            .get_tree()
            .change_scene_to_file("res://scenes/menu.tscn");
    }

    fn load_hole(&mut self) {
        let mut container = self.base().get_node_as::<Node2D>("HoleContainer");
        for mut child in container.get_children().iter_shared() {
            child.queue_free();
        }

        let path = self.hole_scenes.at(self.current);
        let scene: Gd<PackedScene> = load(&path);
        let hole_root = scene.instantiate().unwrap();
        container.add_child(&hole_root);

        let ball = hole_root.get_node_as::<Ball>("Ball");
        let mut ball_obj = ball.upcast::<Object>();
        ball_obj.connect("stroke_taken", &self.to_gd().callable("on_stroke_taken"));

        let cup = hole_root.get_node_as::<Hole>("Cup");
        let mut cup_obj = cup.upcast::<Object>();
        cup_obj.connect("ball_sunk", &self.to_gd().callable("on_ball_sunk"));

        self.update_hud();
    }

    fn update_hud(&self) {
        let total = self.hole_scenes.len();
        let par = self.par_for(self.current);
        let strokes = self.strokes.get(self.current).copied().unwrap_or(0);

        self.set_label(
            "HUD/HoleLabel",
            &format!("Hole {}/{}", self.current + 1, total),
        );
        self.set_label("HUD/ParLabel", &format!("Par {par}"));
        self.set_label("HUD/StrokeLabel", &format!("Strokes: {strokes}"));
    }

    fn show_scorecard(&mut self) {
        self.finished = true;
        let mut text = String::from("Scorecard\n\n");
        for (i, strokes) in self.strokes.iter().enumerate() {
            let par = self.par_for(i);
            let diff = strokes - par;
            let diff_str = match diff {
                d if d > 0 => format!("+{d}"),
                0 => "E".to_string(),
                d => d.to_string(),
            };
            text.push_str(&format!(
                "Hole {}:  {strokes}  (par {par}, {diff_str})\n",
                i + 1
            ));
        }
        let total: i32 = self.strokes.iter().sum();
        let total_par: i32 = self.pars.as_slice().iter().sum();
        text.push_str(&format!("\nTotal: {total}  (par {total_par})"));

        self.set_label("Scorecard/Panel/Results", &text);
        self.base()
            .get_node_as::<CanvasLayer>("Scorecard")
            .set_visible(true);
    }

    fn set_label(&self, path: &str, text: &str) {
        self.base()
            .get_node_as::<Label>(path)
            .set_text(&GString::from(text));
    }

    fn par_for(&self, hole: usize) -> i32 {
        self.pars.as_slice().get(hole).copied().unwrap_or(0)
    }

    pub fn hole_index(&self) -> usize {
        self.current
    }

    pub fn stroke_counts(&self) -> Vec<i32> {
        self.strokes.clone()
    }

    pub fn par_values(&self) -> Vec<i32> {
        self.pars.as_slice().to_vec()
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

#[godot_api]
impl INode for GameManager {
    fn ready(&mut self) {
        if self.hole_scenes.is_empty() {
            godot_error!("GameManager: no hole scenes configured");
            return;
        }
        self.strokes = vec![0; self.hole_scenes.len()];
        self.current = 0;
        self.load_hole();
    }
}
