use anyhow::{anyhow, Result};
use eframe::egui;
use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};

struct PetApp {
    app_state: AppState,
    background_event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
}

#[derive(Debug)]
struct AppState {
    selected_pet: Option<Pet>,
    pets: Vec<Pet>,
}

impl PetApp {
    fn new(background_event_sender: Sender<Event>, event_receiver: Receiver<Event>) -> Self {
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
            background_event_sender,
            event_receiver,
        }
    }
}

enum Event {
    GetPetImage(egui::Context),
    SetPetImage(Option<String>),
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
                    // TODO: fetch image and show in a box, similar to https://github.com/emilk/egui/blob/master/crates/egui_demo_app/src/apps/http_app.rs
                });
            });
        });
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let (background_event_sender, background_event_receiver) = channel::<Event>();
    let (event_sender, event_receiver) = channel::<Event>();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    std::thread::spawn(move || {
        while let Ok(event) = background_event_receiver.recv() {
            // TODO: handle event
        }
    });

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
        Box::new(|_context| Box::new(PetApp::new(background_event_sender, event_receiver))),
    )
    .map_err(|e| anyhow!("eframe error: {}", e))
}

fn load_init_sql() -> std::io::Result<String> {
    fs::read_to_string("./init.sql")
}
