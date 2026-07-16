use std::sync::atomic::Ordering;

use godot::classes::{Button, Control, GridContainer, IControl};
use godot::prelude::*;

use crate::game::SELECTED_HOLE;

/// Title screen; Start/Quit buttons are wired in menu.tscn, the practice
/// grid (one button per hole) is built here.
#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct MainMenu {
    /// Number of practice buttons; keep in sync with main.tscn's hole list.
    #[export]
    #[init(val = 18)]
    hole_count: i64,
    base: Base<Control>,
}

#[godot_api]
impl MainMenu {
    #[func]
    fn on_start_pressed(&mut self) {
        SELECTED_HOLE.store(-1, Ordering::Relaxed);
        self.start_game();
    }

    #[func]
    fn on_hole_selected(&mut self, hole: i64) {
        SELECTED_HOLE.store(hole as i32, Ordering::Relaxed);
        self.start_game();
    }

    #[func]
    fn on_quit_pressed(&mut self) {
        self.base().get_tree().quit();
    }

    fn start_game(&mut self) {
        let _ = self
            .base()
            .get_tree()
            .change_scene_to_file("res://scenes/main.tscn");
    }
}

#[godot_api]
impl IControl for MainMenu {
    fn ready(&mut self) {
        let Some(mut grid) = self.base().try_get_node_as::<GridContainer>("HoleGrid") else {
            return;
        };
        for hole in 1..=self.hole_count {
            let mut button = Button::new_alloc();
            button.set_text(&GString::from(format!("{hole}").as_str()));
            button.set_custom_minimum_size(Vector2::new(52.0, 40.0));
            let callable = self
                .to_gd()
                .callable("on_hole_selected")
                .bindv(&varray![hole]);
            button.connect("pressed", &callable);
            grid.add_child(&button);
        }
    }
}
