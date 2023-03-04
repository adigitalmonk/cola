# Cola

> **CO**nfiguration **L**o**A**der

Simplify the boilerplate for loading your configuration.

![unit tests workflow](https://github.com/adigitalmonk/cola/actions/workflows/test.yml/badge.svg)

## Installation

```toml
[dependencies]
cola = { git = "https://github.com/adigitalmonk/cola", branch = "main" }
```

## Usage

The provided `make_conf!` macro will generate a struct called `Configuration`.

The macro takes a string that represents an environment variable, an identifier, and a type.

- The identifier and type will be used as fields on the `Configuration` struct
- The environment variable will be loaded at runtime and cast into the provided type for the field.

You can then implement any functions you want for the struct to provide any functionality that you need.

Now, let's say you have some environment that looks like the following.

```shell
export YOUR_NAME=Brad
export YOUR_AGE=20
```

You can then use the `make_conf!` macro to generate the Configuration struct.

```rust

// This will also generate some simple docs for the struct that gets created
cola::make_conf! [
    "YOUR_NAME" => your_name: String,
    "YOUR_AGE" => your_age: u32
];

impl Configuration {
    fn hello(&self) -> String {
        format!("Hello, {}", self.your_name)
    }

    fn age(&self) -> String {
      match self.your_age {
        age if age >= 18 => {
          "Voting age".to_string()
        },
        age => {
          "Too young to vote".to_string()
        }
      }
    }
}

// A default implementation will be generated that sets the values from those environment values.
let my_conf = Configuration::default();
assert_eq!(my_conf.hello(), "Hello, Brad");
assert_eq!(my_conf.age(), "Voting age");
assert_eq!(my_conf.your_name, "Brad");
assert_eq!(my_conf.your_age, 20u32);
```

The generated macro looks something like this:

```rust
struct Configuration {
  your_name: String,
  your_age: u32
}
```

