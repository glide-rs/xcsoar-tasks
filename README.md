# xcsoar-tasks

A Rust library for parsing [XCSoar](https://xcsoar.org/) task files. 

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
xcsoar-tasks = "0.0.0"
```

## Usage

```rust,no_run
let xml = std::fs::read_to_string("task.tsk").unwrap();
let task = xcsoar_tasks::parse(&xml).unwrap();

println!("Task type: {:?}", task.task_type);
for point in &task.points {
    println!("  {}: {:?}", point.waypoint.name, point.point_type);
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dually licensed as above, without any additional terms or conditions.
