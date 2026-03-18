use std::sync::Arc;

use eframe::egui::{self, FontId};

pub struct Layouter;

impl Layouter {
  pub fn new() -> Self {
    Self
  }

  pub fn layout(&self, ui: &egui::Ui, string: &str, wrap_width: f32, font_id: FontId) -> Arc<eframe::egui::Galley> {
    let mut layout_job = egui::text::LayoutJob::default();

    layout_job.append(
      string,
      0., 
      egui::TextFormat::simple(font_id, ui.visuals().widgets.active.text_color())
    );

    layout_job.wrap.max_width = f32::MAX;
    ui.fonts_mut(|f| f.layout_job(layout_job))
  }
}