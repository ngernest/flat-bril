# A Flattened Representation for Bril

- Run `cargo build` to compile. 
- Example usage:
```bash
bril2json < test/tiny.bril | cargo run
```
- Run `cargo test` to run unit tests

Repo structure:
- [`main.rs`](./src/main.rs): converts JSON to our flattened representation, WIP

Other stuff: 
- [`bril-rs`](./bril-rs/): existing JSON to Bril infra
- [`brilirs`](./brilirs/): existing Bril interpreter 

