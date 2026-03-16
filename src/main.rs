use std::io;

use eframe::egui::{self, Id};

use crate::{editor::Editor, fs_manager::{File, Manager}};

mod fs_manager;
mod editor;
mod layouter;

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

    eframe::run_native(
        "egui",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default())))
    )
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum ColorChoice { Red, Green, Blue }

#[derive(PartialEq, Debug, Clone, Copy)]
enum AppMode { View, Edit, Settings }

struct MyApp {
    label: String,
    value: f32,
    show_extra_info: bool,
    selected_color: ColorChoice,
    counter: i32,
    current_mode: AppMode,
    manager: Manager,
    editor: Editor,
    opening_file: bool
}

impl Default for MyApp {
    fn default() -> Self {
        let manager = Manager::new();
        let editor = Editor::new();

        Self {
            label: "Initial text".to_string(),
            value: 42.2,
            show_extra_info: false,
            selected_color: ColorChoice::Red,
            counter: 0,
            current_mode: AppMode::View,
            manager,
            editor,
            opening_file: false
        }
    }
}

impl MyApp {
    fn render_files_recursively(&mut self, ui: &mut egui::Ui, files: Vec<File>) {
        for file in files {
            if file.is_dir() && file.name().is_some() {
                ui.collapsing(file.name().unwrap(), |collapsing_ui| {
                    self.render_files_recursively(collapsing_ui, file.children());
                });
            } else if file.name().is_some() {
                let response = ui.add(egui::SelectableLabel::new(false, file.name().unwrap()).truncate());

                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if response.clicked() {
                    self.editor.new_instance(format!("{}/{}", file.path().to_string(), file.name().unwrap())).unwrap();
                }
            }
        } 
    }

    fn set_current_path(&mut self, path: String) -> Result<(), io::Error> {
        self.manager.set_path(path).load()?;
        Ok(())
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::top("menu_bar").exact_height(35.).frame(egui::Frame::new().inner_margin(0)).exact_height(35.).show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(4, 9, 26))
                    .show(ui, |ui| {
                        ui.set_height(ui.available_height());
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add_space(8.);
                            egui::menu::bar(ui, |ui| {
                                ui.set_height(ui.available_height());
                                ui.menu_button("File", |ui| {
                                    if ui.button("Open File").clicked() { 
                                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                                            println!("{:#?}", path);
                                            self.set_current_path(path.to_str().unwrap().to_string()).unwrap();
                                        }
                                        self.opening_file = false;
                                        ctx.request_repaint();
                                    }
                                    if ui.button("Open Folder").clicked() { 
                                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                            println!("{}", path.to_str().unwrap().to_string());
                                            self.set_current_path(path.to_str().unwrap().to_string()).unwrap();
                                        }
                                        self.opening_file = false;
                                    }
                                    if ui.button("Save").clicked() { 
                                        match self.editor.save_current_instance() {
                                            Ok(_) => {},
                                            Err(e) => {
                                                println!("Current instance: {:#?}", e);
                                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                                    println!("Choosing path to create new file: {}", path.to_str().unwrap().to_string());
                                                    self.set_current_path(path.to_str().unwrap().to_string()).unwrap();
                                                    self.editor.new_instance(path.to_str().unwrap().to_string()).unwrap();
                                                }
                                            }
                                        }
                                        }
                                    if ui.button("Quit").clicked() {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                });

                                ui.menu_button("Edit", |ui| {
                                    if ui.radio_value(&mut self.current_mode, AppMode::View, "View").clicked() { ui.close(); }
                                    if ui.radio_value(&mut self.current_mode, AppMode::Edit, "Edit").clicked() { ui.close(); }
                                    if ui.radio_value(&mut self.current_mode, AppMode::Settings, "Settings").clicked() { ui.close(); }
                                })
                            });
                        });
                    });
            });

            egui::SidePanel::left("info_panel")
                .default_width(250.)
                .min_width(200.)
                .max_width(400.)
                .frame(egui::Frame::new().inner_margin(0.0))
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(5, 10, 28))
                        .inner_margin(10.)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Files Explorer").size(16.));
                            ui.separator();
                            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                                ui.set_max_width(ui.available_width()); 

                                if self.manager.files().len() > 0 {
                                    let parent_name = self.manager.path().unwrap().file_name().unwrap().to_str().unwrap().to_string();
                                    ui.collapsing(parent_name, |ui| {
                                        self.render_files_recursively(ui, self.manager.files());
                                    });
                                } else {
                                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                        ui.heading("Empty");
                                    });
                                }
                            });
                        });
                });

            egui::CentralPanel::default().frame(egui::Frame::new().inner_margin(0)).show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(4, 9, 26))
                    .inner_margin(10.)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.set_min_height(24.);
                        ui.horizontal(|ui| {
                            for (name, path, saved) in self.editor.instances_data().unwrap() {
                                let response = egui::Frame::new()
                                    .inner_margin(4.)
                                    .corner_radius(4.)
                                    .show(ui, |ui| {
                                        ui.style_mut().interaction.selectable_labels = false;
                                        ui.set_width(20.);
                                        ui.horizontal(|ui| {
                                            if !saved {
                                                let (rect, _) = ui.allocate_exact_size(egui::vec2(10., 10.), egui::Sense::hover());
                                                ui.painter().circle_filled(
                                                    rect.center(), 2., egui::Color32::WHITE);
                                            }

                                            ui.label(name);
                                        });
                                    }).response.interact(egui::Sense::click());

                                if response.clicked() {
                                    self.editor.new_instance(path).unwrap();
                                    println!("Clicked")
                                }

                                if response.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                ui.separator();
                            };
                        });
                    });
                let (rect, _) = ui.allocate_at_least(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                ui.painter().hline(
                    rect.left()..=rect.right(),
                    rect.center().y,
                    ui.visuals().widgets.noninteractive.bg_stroke
                );
                match self.current_mode {
                    AppMode::View => {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(6, 11, 31))
                            .inner_margin(10.)
                            .show(ui, |ui| {
                                ui.allocate_ui(ui.available_size(), |ui| {
                                    let text_style = egui::TextStyle::Monospace;
                                    let row_height = ui.text_style_height(&text_style);
                                    let total_rows = self.editor.current_content.len_lines();
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false; 2]) 
                                        .min_scrolled_height(ui.available_height())
                                        .id_salt("editor_scroll")
                                        .show_rows(ui, row_height, total_rows, |ui, rows_range| {
                                            ui.horizontal(|ui| {
                                                ui.vertical( |ui| {
                                                    ui.set_width(total_rows.to_string().len() as f32*8.);
                                                    ui.set_min_width(16.);
                                                    ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                                                        for i in rows_range.clone() {
                                                            ui.label((i + 1).to_string());
                                                        }
                                                    });
                                                });
                                                ui.separator();
                                                let visible_rows = rows_range.end - rows_range.start;
                                                let start_idx = self.editor.current_content.line_to_char(rows_range.start);
                                                let end_idx = self.editor.current_content.line_to_char(rows_range.end);
                                                let mut buf = match self.editor.current_content.len_lines() {
                                                    0..=1 => String::new(),
                                                    2.. => {
                                                        let slice = self.editor.current_content.slice(start_idx..end_idx);
                                                        slice.to_string()
                                                    }
                                                };
                                                let response = ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut buf)
                                                    .font(text_style)
                                                    .code_editor()
                                                    .lock_focus(true)
                                                    .desired_width(f32::INFINITY)
                                                    .desired_rows(visible_rows)
                                                    .id(Id::new("editor input"))
                                                    .hint_text("Start typing something...")
                                                    .frame(false)
                                                );

                                                if response.changed() {
                                                    self.editor.update_instance_content(start_idx..end_idx, buf).unwrap();
                                                }

                                                if response.hovered() {
                                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                                                }
                                            });
                                    });
                                });
                            });
                    },
                    AppMode::Edit => {
                        ui.heading("Editing data");
                        egui::Grid::new("edit_grid")
                            .num_columns(2)
                            .spacing([20., 8.])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Edit Label: ");
                                ui.text_edit_singleline(&mut self.label);
                                ui.end_row();

                                ui.label("Adjust Value: ");
                                ui.add(egui::Slider::new(&mut self.value, 0.0..=10.));
                                ui.end_row();

                                ui.label("Counter: ");
                                ui.horizontal(|ui| {
                                    if ui.button("+").clicked() { self.counter += 1 }
                                    ui.label(format!("{}", self.counter)).highlight();
                                    if ui.button("-").clicked() { self.counter -= 1 }
                                });
                                ui.end_row();
                            });
                    },
                    AppMode::Settings => {
                        ui.heading("Settings");
                        ui.group(|ui| {
                            ui.checkbox(&mut self.show_extra_info, "Show Advanced Info");
                            ui.separator();
                            ui.label("Color scheme: ");
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut self.selected_color, ColorChoice::Red, "Red");
                                ui.radio_value(&mut self.selected_color, ColorChoice::Green, "Green");
                                ui.radio_value(&mut self.selected_color, ColorChoice::Blue, "Blue");
                            });
                        });
                        ui.separator();

                        if ui.button("Reset All State").clicked() {
                            *self = MyApp::default();
                        }
                    }
                }
            })
        });
    }
}