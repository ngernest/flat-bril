# A Flattened Representation for Bril

- Run `cargo build` to compile. 
- Example usage:
```bash
bril2json < test/tiny.bril | cargo run
```
This outputs: 
```bash
@main
	op: const		dest: v 		type: int		value: 4
	op: const		dest: b 		type: bool		value: false
	op: br   		args: ["b"]		labels: ["there", "here"]
.here
	op: const		dest: v 		type: int		value: 2
.there
	op: print		args: ["v"]
```

- Run `cargo test` to run unit tests
- Run `turnt -e roundtrip test/*.bril` to check whether `brili` returns the same output after a JSON -> flattened format -> JSON round trip

Repo structure:
- [`main.rs`](./src/main.rs): Reads in a JSON Bril file from `stdin`
- [`flatten.rs`](./src/flatten.rs): Converts a JSON Bril file to a flattened instruction format 
- [`types.rs`](./src/flatten.rs): Type definitions & pretty-printers
- [`mk_json.sh`](./mk_json.sh): Bash script, invokes `bril2json` on every `.bril` files in the `test` subdirectory and converts them to `.json` files 

Other stuff: 
- [`bril-rs`](./bril-rs/): existing JSON to Bril infra
- [`brilirs`](./brilirs/): existing Bril interpreter 

