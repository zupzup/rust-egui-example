use anyhow::{anyhow, Result};
use eframe::egui;
use std::fs;

struct PetApp {
    app_state: AppState,
}

#[derive(Debug)]
struct AppState {
    selected_pet: Option<Pet>,
    pets: Vec<Pet>,
}

impl PetApp {
    fn new() -> Self {
        let pets = vec![
            Pet {
                id: 1,
                name: "minka".to_string(),
                age: 9,
                kind_id: 1,
            },
            Pet {
                id: 2,
                name: "nala".to_string(),
                age: 7,
                kind_id: 2,
            },
        ];
        Self {
            app_state: AppState {
                selected_pet: None,
                pets,
            },
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
struct Pet {
    id: u64,
    name: String,
    age: u16,
    kind_id: u64,
}

#[derive(Debug, PartialEq, Clone)]
struct PetKind {
    id: u64,
    name: String,
}

impl eframe::App for PetApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("left panel")
                .resizable(false)
                .default_width(200.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Pets");
                        self.app_state.pets.iter().for_each(|pet| {
                            ui.selectable_value(
                                &mut self.app_state.selected_pet,
                                Some(pet.to_owned()),
                                pet.name.clone(),
                            );
                        });
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Details");
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
        Box::new(|_context| Box::new(PetApp::new())),
    )
    .map_err(|e| anyhow!("eframe error: {}", e))
}

fn load_init_sql() -> std::io::Result<String> {
    fs::read_to_string("./init.sql")
}
