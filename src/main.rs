use std::fs;

fn main() {
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

    println!("Hello, world!");
}

fn load_init_sql() -> std::io::Result<String> {
    fs::read_to_string("./init.sql")
}
