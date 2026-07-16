use std::sync::atomic::Ordering;

use godot::classes::{Button, Control, GridContainer, IControl};
use godot::prelude::*;

use crate::game::{SELECTED_COURSE, SELECTED_HOLE};

/// Title screen. Start/Quit buttons are wired in menu.tscn; each course's
/// practice grid (one button per hole) is built here.
#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct MainMenu {
    /// Practice buttons per course; keep in sync with the course tables.
    #[export]
    #[init(val = 18)]
    hole_count: i64,
    base: Base<Control>,
}

/// (grid node in menu.tscn, course id passed to the game).
const COURSE_GRIDS: [(&str, i64); 3] = [("ClassicGrid", 0), ("IceGrid", 1), ("DesertGrid", 2)];

#[godot_api]
impl MainMenu {
    #[func]
    fn on_start_classic(&mut self) {
        self.start_round(0);
    }

    #[func]
    fn on_start_ice(&mut self) {
        self.start_round(1);
    }

    #[func]
    fn on_start_desert(&mut self) {
        self.start_round(2);
    }

    #[func]
    fn on_hole_selected(&mut self, course: i64, hole: i64) {
        SELECTED_COURSE.store(course as i32, Ordering::Relaxed);
        SELECTED_HOLE.store(hole as i32, Ordering::Relaxed);
        self.start_game();
    }

    #[func]
    fn on_quit_pressed(&mut self) {
        self.base().get_tree().quit();
    }

    fn start_round(&mut self, course: i64) {
        SELECTED_COURSE.store(course as i32, Ordering::Relaxed);
        SELECTED_HOLE.store(-1, Ordering::Relaxed);
        self.start_game();
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
        for (grid_name, course) in COURSE_GRIDS {
            let Some(mut grid) = self.base().try_get_node_as::<GridContainer>(grid_name) else {
                continue;
            };
            for hole in 1..=self.hole_count {
                let mut button = Button::new_alloc();
                button.set_text(&GString::from(format!("{hole}").as_str()));
                button.set_custom_minimum_size(Vector2::new(42.0, 36.0));
                let callable = self
                    .to_gd()
                    .callable("on_hole_selected")
                    .bindv(&varray![course, hole]);
                button.connect("pressed", &callable);
                grid.add_child(&button);
            }
        }
    }
}
