use anyhow::{anyhow, Result};
use eframe::egui;
use serde::Deserialize;
use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

const GET_PET_BY_ID: &str = "SELECT id, name, age, kind FROM pets where id = ?";
const DELETE_PET_BY_ID: &str = "DELETE FROM pets where id = ?";
const INSERT_PET: &str =
    "INSERT INTO pets (name, age, kind) VALUES (?, ?, ?) RETURNING id, name, age, kind";
const GET_PETS: &str = "SELECT id, name, age, kind FROM pets";

struct PetApp {
    app_state: AppState,
    background_event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
    db_con: Arc<Mutex<sqlite::Connection>>,
}

#[derive(Debug, Clone)]
struct AppState {
    selected_pet: Option<Pet>,
    pets: Vec<Pet>,
    pet_image: Option<String>,
    add_form: AddForm,
}

#[derive(Debug, Clone)]
struct AddForm {
    show: bool,
    name: String,
    age: String,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct CatJSON {
    #[serde(alias = "0")]
    item: CatJSONInner,
}

#[derive(Debug, Deserialize)]
struct CatJSONInner {
    url: String,
}

#[derive(Debug, Deserialize)]
struct DogJSON {
    message: String,
}

impl PetApp {
    fn new(
        background_event_sender: Sender<Event>,
        event_receiver: Receiver<Event>,
        db_con: sqlite::Connection,
    ) -> Result<Box<Self>> {
        let db_con = Arc::new(Mutex::new(db_con));
        let pets = get_pets_from_db(db_con.clone())?;
        Ok(Box::new(Self {
            app_state: AppState {
                selected_pet: None,
                pets,
                pet_image: None,
                add_form: AddForm {
                    show: false,
                    name: String::default(),
                    age: String::default(),
                    kind: String::default(),
                },
            },
            background_event_sender,
            event_receiver,
            db_con,
        }))
    }

    fn handle_gui_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                Event::SetPetImage(pet_image) => {
                    self.app_state.pet_image = pet_image;
                }
                Event::SetSelectedPet(pet) => self.app_state.selected_pet = pet,
                Event::SetPets(pets) => {
                    if let Some(ref selected_pet) = self.app_state.selected_pet {
                        if !pets.iter().any(|p| p.id == selected_pet.id) {
                            self.app_state.selected_pet = None;
                        }
                    }
                    self.app_state.pets = pets;
                }
                _ => (),
            };
        }
    }
}

enum Event {
    SetPets(Vec<Pet>),
    GetPetImage(egui::Context, PetKind),
    SetPetImage(Option<String>),
    GetPetFromDB(egui::Context, Arc<Mutex<sqlite::Connection>>, i64),
    SetSelectedPet(Option<Pet>),
    InsertPetToDB(egui::Context, Arc<Mutex<sqlite::Connection>>, Pet),
    DeletePetFromDB(egui::Context, Arc<Mutex<sqlite::Connection>>, i64),
}

#[derive(Debug, PartialEq, Clone)]
struct PetKind(String);

#[derive(Debug, PartialEq, Clone)]
struct Pet {
    id: i64,
    name: String,
    age: i64,
    kind: PetKind,
}

impl eframe::App for PetApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.handle_gui_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("left panel")
                .resizable(false)
                .default_width(200.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Pets");
                        ui.separator();
                        if ui.button("Add new Pet").clicked() {
                            self.app_state.add_form.show = !self.app_state.add_form.show;
                        }
                        if self.app_state.add_form.show {
                            ui.separator();

                            ui.vertical_centered(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("name:");
                                        ui.label("age");
                                        ui.label("kind");
                                    });
                                    ui.end_row();
                                    ui.vertical(|ui| {
                                        ui.text_edit_singleline(&mut self.app_state.add_form.name);
                                        ui.text_edit_singleline(&mut self.app_state.add_form.age);
                                        ui.text_edit_singleline(&mut self.app_state.add_form.kind);
                                    });
                                });

                                if ui.button("Submit").clicked() {
                                    let add_form = &mut self.app_state.add_form;
                                    let age = add_form.age.parse::<i64>().unwrap_or(0);
                                    let kind = match add_form.kind.as_str() {
                                        "cat" => PetKind(String::from("cat")),
                                        _ => PetKind(String::from("dog")),
                                    };
                                    let name = add_form.name.to_owned();
                                    if !name.is_empty() && age > 0 {
                                        let _ = self.background_event_sender.send(
                                            Event::InsertPetToDB(
                                                ctx.clone(),
                                                self.db_con.clone(),
                                                Pet {
                                                    id: -1,
                                                    name,
                                                    age,
                                                    kind: kind.clone(),
                                                },
                                            ),
                                        );
                                        let _ = self
                                            .background_event_sender
                                            .send(Event::GetPetImage(ctx.clone(), kind));
                                        add_form.name = String::default();
                                        add_form.age = String::default();
                                        add_form.kind = String::default();
                                    }
                                }
                            });
                        }

                        ui.separator();
                        self.app_state.pets.iter().for_each(|pet| {
                            if ui
                                .selectable_value(
                                    &mut self.app_state.selected_pet,
                                    Some(pet.to_owned()),
                                    pet.name.clone(),
                                )
                                .changed()
                            {
                                let _ = self.background_event_sender.send(Event::GetPetFromDB(
                                    ctx.clone(),
                                    self.db_con.clone(),
                                    pet.id,
                                ));
                                let _ = self
                                    .background_event_sender
                                    .send(Event::GetPetImage(ctx.clone(), pet.kind.clone()));
                            }
                        });
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Details");
                    if let Some(pet) = &self.app_state.selected_pet {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Delete").clicked() {
                                    let _ =
                                        self.background_event_sender.send(Event::DeletePetFromDB(
                                            ctx.clone(),
                                            self.db_con.clone(),
                                            pet.id,
                                        ));
                                }
                            });
                            ui.separator();
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("id:");
                                        ui.label("name:");
                                        ui.label("age");
                                        ui.label("kind");
                                    });
                                    ui.end_row();
                                    ui.vertical(|ui| {
                                        ui.label(pet.id.to_string());
                                        ui.label(&pet.name);
                                        ui.label(pet.age.to_string());
                                        ui.label(&pet.kind.0);
                                    });
                                });
                                ui.separator();
                                if let Some(ref pet_image) = self.app_state.pet_image {
                                    ui.add(egui::Image::from_uri(pet_image).max_width(200.0));
                                }
                            });
                        });
                    } else {
                        ui.label("No pet selected.");
                    }
                });
            });
        });
    }
}

fn insert_pet_to_db(db_con: Arc<Mutex<sqlite::Connection>>, pet: Pet) -> Result<Pet> {
    let con = db_con
        .lock()
        .map_err(|_| anyhow!("error while locking db connection"))?;
    let mut stmt = con.prepare(INSERT_PET)?;
    stmt.bind((1, pet.name.as_str()))?;
    stmt.bind((2, pet.age))?;
    stmt.bind((3, pet.kind.0.as_str()))?;

    if stmt.next()? == sqlite::State::Row {
        let id = stmt.read::<i64, _>(0)?;
        let name = stmt.read::<String, _>(1)?;
        let age = stmt.read::<i64, _>(2)?;
        let kind = stmt.read::<String, _>(3)?;

        return Ok(Pet {
            id,
            name,
            age,
            kind: PetKind(kind),
        });
    }

    Err(anyhow!("error while inserting pet"))
}

fn delete_pet_from_db(db_con: Arc<Mutex<sqlite::Connection>>, pet_id: i64) -> Result<()> {
    let con = db_con
        .lock()
        .map_err(|_| anyhow!("error while locking db connection"))?;
    let mut stmt = con.prepare(DELETE_PET_BY_ID)?;
    stmt.bind((1, pet_id))?;

    if stmt.next()? == sqlite::State::Done {
        Ok(())
    } else {
        Err(anyhow!("error while deleting pet with id {}", pet_id))
    }
}

fn get_pet_from_db(db_con: Arc<Mutex<sqlite::Connection>>, pet_id: i64) -> Result<Option<Pet>> {
    let con = db_con
        .lock()
        .map_err(|_| anyhow!("error while locking db connection"))?;
    let mut stmt = con.prepare(GET_PET_BY_ID)?;
    stmt.bind((1, pet_id))?;

    if stmt.next()? == sqlite::State::Row {
        let id = stmt.read::<i64, _>(0)?;
        let name = stmt.read::<String, _>(1)?;
        let age = stmt.read::<i64, _>(2)?;
        let kind = stmt.read::<String, _>(3)?;

        return Ok(Some(Pet {
            id,
            name,
            age,
            kind: PetKind(kind),
        }));
    }
    Ok(None)
}

fn get_pets_from_db(db_con: Arc<Mutex<sqlite::Connection>>) -> Result<Vec<Pet>> {
    let con = db_con
        .lock()
        .map_err(|_| anyhow!("error while locking db connection"))?;
    let mut pets: Vec<Pet> = vec![];
    let mut stmt = con.prepare(GET_PETS)?;

    for row in stmt.iter() {
        let row = row?;
        let id = row.read::<i64, _>(0);
        let name = row.read::<&str, _>(1);
        let age = row.read::<i64, _>(2);
        let kind = row.read::<&str, _>(3);

        pets.push(Pet {
            id,
            name: name.to_owned(),
            age,
            kind: PetKind(kind.to_owned()),
        });
    }
    Ok(pets)
}

fn fetch_pet_image(ctx: egui::Context, pet_kind: PetKind, sender: Sender<Event>) {
    let url = if pet_kind.0 == "dog" {
        "https://dog.ceo/api/breeds/image/random"
    } else {
        "https://api.thecatapi.com/v1/images/search"
    };
    ehttp::fetch(
        ehttp::Request::get(url),
        move |result: ehttp::Result<ehttp::Response>| {
            if let Ok(result) = result {
                let image_url = if pet_kind.0 == "dog" {
                    if let Ok(json) = result.json::<DogJSON>() {
                        Some(json.message)
                    } else {
                        None
                    }
                } else if let Ok(json) = result.json::<CatJSON>() {
                    Some(json.item.url)
                } else {
                    None
                };
                let _ = sender.send(Event::SetPetImage(image_url));
                ctx.request_repaint();
            }
        },
    );
}

fn handle_events(event: Event, sender: Sender<Event>) {
    match event {
        Event::GetPetImage(ctx, pet_kind) => {
            fetch_pet_image(ctx, pet_kind, sender);
        }
        Event::GetPetFromDB(ctx, db_con, pet_id) => {
            if let Ok(Some(pet)) = get_pet_from_db(db_con, pet_id) {
                let _ = sender.send(Event::SetSelectedPet(Some(pet)));
                ctx.request_repaint();
            }
        }
        Event::DeletePetFromDB(ctx, db_con, pet_id) => {
            if delete_pet_from_db(db_con.clone(), pet_id).is_ok() {
                if let Ok(pets) = get_pets_from_db(db_con) {
                    let _ = sender.send(Event::SetPets(pets));
                    ctx.request_repaint();
                }
            }
        }
        Event::InsertPetToDB(ctx, db_con, pet) => {
            if let Ok(new_pet) = insert_pet_to_db(db_con.clone(), pet) {
                if let Ok(pets) = get_pets_from_db(db_con) {
                    let _ = sender.send(Event::SetPets(pets));
                    let _ = sender.send(Event::SetSelectedPet(Some(new_pet)));
                    ctx.request_repaint();
                }
            }
        }
        _ => (),
    }
}

fn load_init_sql() -> std::io::Result<String> {
    fs::read_to_string("./init.sql")
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
            let sender = event_sender.clone();
            handle_events(event, sender);
        }
    });

    let init_query = load_init_sql().expect("can load init query");
    let db_con = sqlite::open(":memory:").expect("can create sqlite db");
    db_con
        .execute(init_query)
        .expect("can initialize sqlite db");

    eframe::run_native(
        "PetApp",
        options,
        Box::new(|context| {
            egui_extras::install_image_loaders(&context.egui_ctx);
            Ok(PetApp::new(
                background_event_sender,
                event_receiver,
                db_con,
            )?)
        }),
    )
    .map_err(|e| anyhow!("eframe error: {}", e))
}
