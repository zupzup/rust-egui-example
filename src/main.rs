use anyhow::{anyhow, Result};
use eframe::egui;
use std::fs;

struct PetApp {
    app_state: AppState,
}

struct AppState {}

impl PetApp {
    fn new() -> Self {
        Self {
            app_state: AppState {},
        }
    }
}

impl eframe::App for PetApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("left panel")
                .resizable(false)
                .default_width(200.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Left");
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("center");
                });
            });
        });
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    let init_query = load_init_sql().unwrap();
    let connection = sqlite::open(":memory:").unwrap();

    connection.execute(init_query).unwrap();

    let query = "SELECT * FROM pet_kinds";

    connection
        .iterate(query, |pairs| {
            for &(name, value) in pairs.iter() {
                println!("{} = {}", name, value.unwrap());
            }
            true
        })
        .unwrap();

    let query = "SELECT * FROM pets";

    connection
        .iterate(query, |pairs| {
            for &(name, value) in pairs.iter() {
                println!("{} = {}", name, value.unwrap());
            }
            true
        })
        .unwrap();

    eframe::run_native(
        "PetApp",
        options,
        Box::new(|context| Box::new(PetApp::new())),
    )
    .map_err(|e| anyhow!("eframe error: {}", e))
}

fn load_init_sql() -> std::io::Result<String> {
    fs::read_to_string("./init.sql")
}
