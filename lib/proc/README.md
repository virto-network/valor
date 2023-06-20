# Procedural Macros for Virto

`virto_proc` is a procedural macros library that powers the Virto developer interfaces.

It provides the procedural macros for the Virto crate, including `#[virto::module]`, `#[virto::method]`, and `#[virto::extensions]`.

## Features

- Procedural macros that enable easy definition of Virto modules and methods.
- Convenient extensions attribute for adding custom functionality to your modules and methods.
- Built with `no_std` compatibility in mind, making it suitable for embedded systems and WebAssembly targets.

## Quick Start

```rust
use virto::*;

#[virto::module]
pub mod my_module {
    #[virto::method]
    #[virto::extensions(http_verb = "GET", http_path = "/")]
    pub fn hello_world(_req: &Request) -> Result<Response, ResponseError> {
        Response::new("Hello, world!")
    }
}
```

This defines a module with a single method that responds to HTTP GET requests at the root path ("/") with the message "Hello, world!".

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
virto_proc = "0.1.0"
```

<!-- ## Documentation
Visit our [documentation](http://www.example.com/documentation) for detailed instructions on using Virto Proc. -->

## Examples

Check out the `/examples` directory for example usage of Virto Proc.

## License

This project is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.
