use godot::prelude::*;

mod ball;
mod course;
mod game;
mod menu;

struct PlinkExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PlinkExtension {}
