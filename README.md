# core_affinity

Set CPU affinity for Rust threads on multiple platforms.

This repository is the LaunchWindow-maintained fork of the original
core_affinity_rs project.

The crate is currently configured as internal (`publish = false`) and is not
published to crates.io.

## Credit

This work is based on the original project by Philip Woods
<elzairthesorcerer@gmail.com>. The upstream license terms remain in effect and
are preserved in `LICENSE-APACHE` and `LICENSE-MIT`.

## Installation

Because this crate is not on crates.io, use a git dependency:

```toml
[dependencies]
core_affinity = { git = "https://github.com/launchwindow-software/core_affinity_rs" }
```

If you want typed errors, enable the `errors` feature:

```toml
[dependencies]
core_affinity = { git = "https://github.com/launchwindow-software/core_affinity_rs", features = ["errors"] }
```

## Features

- No default features (`default = []`).
- Optional feature: `errors`.

## API

Default API:

- `get_core_ids() -> Option<Vec<CoreId>>`
- `set_for_current(CoreId) -> bool`

Error API (`errors` feature):

- `try_get_core_ids() -> Result<Vec<CoreId>, AffinityError>`
- `try_set_for_current(CoreId) -> Result<(), AffinityError>`

## Example (Default API)

```rust
use std::thread;

fn main() {
    let Some(core_ids) = core_affinity::get_core_ids() else {
        eprintln!("No affinity information available for this target/process");
        return;
    };

    let handles = core_ids
        .into_iter()
        .map(|id| {
            thread::spawn(move || {
                if core_affinity::set_for_current(id) {
                    // Do CPU-bound work here.
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        if let Err(err) = handle.join() {
            eprintln!("thread join failed: {err:?}");
        }
    }
}
```

## Example (`errors` Feature)

```rust
use core_affinity::{try_get_core_ids, try_set_for_current};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core_ids = try_get_core_ids()?;

    let handles = core_ids
        .into_iter()
        .map(|id| {
            thread::spawn(move || {
                if let Err(err) = try_set_for_current(id) {
                    eprintln!("failed to pin thread: {err}");
                    return;
                }

                // Do CPU-bound work here.
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        if let Err(err) = handle.join() {
            eprintln!("thread join failed: {err:?}");
        }
    }

    Ok(())
}
```

## Supported Platforms

- Linux
- Android
- Windows
- macOS
- FreeBSD
- NetBSD

On unsupported targets:

- `get_core_ids()` returns `None`
- `set_for_current(...)` returns `false`

## CI

GitHub Actions validates:

- Native build/test on Linux, Windows, and macOS (stable)
- Cross-target test compilation for Linux and Apple target sets
- Clippy checks

## Development

Useful commands:

```bash
cargo test
cargo test --features errors
cargo clippy --all-targets --features errors --no-default-features -- -D warnings
cargo fmt --all -- --config=reorder_imports=true --config=imports_granularity=Item --config=group_imports=StdExternalCrate
cargo clippy --fix --all-targets --allow-dirty --allow-staged --no-default-features --no-deps --keep-going -- -D clippy::pedantic
```
