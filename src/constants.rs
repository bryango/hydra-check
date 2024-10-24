//! Useful constants shared across the program.

/// Currently supported systems on [hydra.nixos.org][hydra].
/// This may change in the future.
///
/// [hydra]: https://hydra.nixos.org
///
pub const KNOWN_ARCHITECTURES: [&str; 4] = [
    "x86_64-linux",
    "aarch64-linux",
    "x86_64-darwin",
    "aarch64-darwin",
];
