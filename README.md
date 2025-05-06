# A Flattened Representation for Bril

- Run `cargo build` to compile. 
- Example usage:
```bash
# This does a JSON -> flattened representation -> JSON round trip and 
# runs the Bril interpreter on the resultant JSON
bril2json < test/tiny.bril | cargo run | brili
```
- Run `cargo test` to run unit tests
- Run `turnt -e roundtrip test/*.bril` to check whether `brili` returns the same output after a JSON -> flattened format -> JSON round trip

Repo structure:
- [`main.rs`](./src/main.rs): Reads in a JSON Bril file from `stdin`
- [`flatten.rs`](./src/flatten.rs): Converts a JSON Bril file to a flattened instruction format 
- [`unflatten.rs`](./src/unflatten.rs): Converts a flattened Bril instruction back to JSON
- [`types.rs`](./src/flatten.rs): Type definitions & pretty-printers
- [`mk_json.sh`](./mk_json.sh): Bash script, invokes `bril2json` on every `.bril` files in the `test` subdirectory and converts them to `.json` files 

Other stuff: 
- [`bril-rs`](./bril-rs/): existing JSON to Bril infra
- [`brilirs`](./brilirs/): existing Bril interpreter 

