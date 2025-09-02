# suck

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/suck.svg)](https://crates.io/crates/suck)
[![Github Actions](https://img.shields.io/github/actions/workflow/status/callumio/suck/ci.yml?branch=main)](https://github.com/callumio/suck/actions?workflow=ci)
[![Documentation](https://docs.rs/suck/badge.svg)](https://docs.rs/suck)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

</div>

Suck data up through a channel

## Features

- Pull-based communication: Consumers request values on-demand
- Contextual values: Designed for current state rather than event streams
- Flexible sources: Support both static values and dynamic closures

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
suck = "*"
```

## Quick Start

```rust
use suck::SuckPair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a pair
    let (sucker, sourcer) = SuckPair::<i32>::pair();

    // Start producer in a thread
    let producer = std::thread::spawn(move || {
        // Set a static value
        sourcer.set_static(42).unwrap();

        // Or set a dynamic closure
        sourcer.set(|| {
            // Generate fresh values each time
            42 * 2
        }).unwrap();

        // Run the producer loop
        sourcer.run().unwrap();
    });

    // Consumer pulls values
    let value = sucker.get()?;
    println!("Got value: {}", value);

    // Clean up
    sucker.close()?;
    producer.join().unwrap();

    Ok(())
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file
for details.
