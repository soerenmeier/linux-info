# Linux Info

[![Crates.io](https://img.shields.io/crates/v/linux-info)](https://crates.io/crates/linux-info)
[![Documentation](https://docs.rs/linux-info/badge.svg)](https://docs.rs/linux-info)
[![GitHub issues](https://img.shields.io/github/issues/soerenmeier/linux-info)](https://github.com/soerenmeier/linux-info/issues)

`linux-info` is a Rust crate that allows you to retrieve information about your Linux system. It provides various modules to access different aspects of your system.

## Modules

The `linux-info` crate currently provides the following modules:

- `cpu`: Retrieves information about the CPU.
- `memory`: Retrieves information about the system memory.
- `system`: Retrieves general system information.
- `storage`: Retrieves information about storage devices.
- `bios`: Retrieves BIOS information.
- `network`: Retrieves network-related information. (Requires the `network` feature)

The crate also includes Serde support, which can be enabled with the `serde` feature.

## Installation

To use `linux-info` in your Rust project, add the following line to your `Cargo.toml` file:

```toml
linux-info = "0.1"
```

## Contribution

Contributions to this crate are welcome! If you have any ideas, bug reports, or feature requests, please open an issue on the [GitHub repository](https://github.com/soerenmeier/linux-info).