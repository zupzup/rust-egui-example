# rust-egui-example

Simple example for building an egui application in Rust

## Run

```bash
RUST_LOG=info cargo run
```

## TODOs

* [x] Set up SQLite
* Create basics GUI window with title
* Create Panel-Layout
    * Left panel: List of pets
    * Right panel: Detail view
* Implement Read
    * Get random image from https://api.thecatapi.com/v1/images/search for cats
    * Get random image from https://dog.ceo/api/breeds/image/random for dogs
* Create Add button
* Create Update/Delete buttons on Detail view
* Implement Delete
* Implement Add
* Implement Update (inline in detail view)
