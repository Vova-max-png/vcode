use std::{cmp::{max, min}, f64::MAX, io};

use eframe::egui::{self, Id, text::{CCursor, CCursorRange}, text_edit::TextEditState};

use crate::{editor::Editor, fs_manager::{File, Manager}, layouter::Layouter, ui::MyApp};

mod fs_manager;
mod editor;
mod layouter;
mod ui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VCode")
            .with_inner_size([1200., 800.])
            .with_min_inner_size([900., 700.])
            .with_resizable(true)
            .with_transparent(false),
        ..Default::default()
    };

    let app: MyApp = MyApp::default();

    eframe::run_native(
        "VCode",
        options,
        Box::new(|_cc| Ok(Box::new(app)))
    )
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum ColorChoice { Red, Green, Blue }

#[derive(PartialEq, Debug, Clone, Copy)]
enum AppMode { Welcome, Editing, Settings }