# hotbolt
Lightweight set of tools for hot-reloading in Rust.

## Examples
To run the examples, first build the root workspace, then build the examples workspace and finally run whichever example you want with `hotbolt_runner`:
```bash
cargo build
pushd examples
cargo build
popd
cargo run examples/target/debug/counter.dll
```
