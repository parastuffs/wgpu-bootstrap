# WGPU Bootstrap

This project is designed for educational purposes. It is probably not suitable for a production application.
The goal is to reduce the boilerplate needed to create a new WebGPU project.

## Usage

Add the library to your `cargo.toml`. Use the `tag` key to specify the version.

```toml
[dependencies]
wgpu-bootstrap = { git = "https://github.com/qlurkin/wgpu-bootstrap", tag = "v0.4.1" }
bytemuck = { version = "1.18", features = ["derive"] }
```

## Example

You can find examples of project in [the example directory](https://github.com/qlurkin/wgpu-bootstrap/tree/main/examples)

You can run the examples with

```shell
cargo run --example triangle
```

## Credits

Heavily inspired by the ["Learn Wgpu" tutorial](https://sotrh.github.io/learn-wgpu)
