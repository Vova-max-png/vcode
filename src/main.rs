use std::io;

use eframe::egui;

use crate::{editor::Editor, fs_manager::{File, Manager}};

mod fs_manager;
mod editor;

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
            if file.is_dir() && file.name().is_some() && file.children().len() > 0 {
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
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
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
                            self.editor.save_current_instance().unwrap();
                        }
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button("Mode", |ui| {
                        if ui.radio_value(&mut self.current_mode, AppMode::View, "View").clicked() { ui.close(); }
                        if ui.radio_value(&mut self.current_mode, AppMode::Edit, "Edit").clicked() { ui.close(); }
                        if ui.radio_value(&mut self.current_mode, AppMode::Settings, "Settings").clicked() { ui.close(); }
                    })
                })
            });

            egui::SidePanel::left("info_panel")
                .default_width(250.)
                .min_width(200.)
                .max_width(400.)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        ui.set_max_width(ui.available_width()); 

                        if self.manager.files().len() > 0 {
                            self.render_files_recursively(ui, self.manager.files());
                        } else {
                            ui.heading("Choose file/folder first");
                        }
                    });
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (name, path) in self.editor.instances_data().unwrap() {
                        let response = ui.add(egui::SelectableLabel::new(false, name));

                        if response.clicked() {
                            self.editor.new_instance(path).unwrap();
                        }
                    }
                });
                ui.separator();
                match self.current_mode {
                    AppMode::View => {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(45, 45, 45))
                            .corner_radius(8.)
                            .inner_margin(10.)
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    let response = ui.add_sized(ui.available_size_before_wrap(), egui::TextEdit::multiline(&mut self.editor.current_content)
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_rows(20)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("Start typing something...")
                                        .frame(false)
                                    ); 

                                    if response.changed() {
                                        self.editor.update_instance_content(self.editor.current_content.clone()).unwrap();
                                    }
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