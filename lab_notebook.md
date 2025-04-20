# Lab Notebook

- We manually ported [Adrian's flatcalc repo](https://github.com/sampsyo/flatcalc) from Rust to OCaml to better understand what it's doing ([OCaml repo](https://github.com/ngernest/ocaml_flatcalc))
- Rewriting the interpreter from scratch in OCaml was helpful, since the distinction between the `Expr` and `ExprRef` types (where the latter is an index into the arena of `Expr`s) becomes much more apparent when you have to get the code to typecheck in OCaml
- The key insight is that when we call `flat_interp` in the Rust code, the `root` parameter
(the `expr` we're passing to `flat_interp`) is actually an index (an `ExprRef`) to the final
element in the arena. This is because:
1. every time we parse a sub-expression, we call `ExprPool::add` to populate the arena from left to right, and
2.  we parse bottom-up (i.e. we have to create child `Expr`s before we create their parent)
These two factors combined mean that the top-level `Expr` (the final `Expr` the parser encounters) corresponds to the final element of the arena. 

To see this in action, I added print statements to [my fork](https://github.com/ngernest/flatcalc/tree/flat) of the Rust flatcalc repo to see how the arena is populated when we parse `5 * 2 + 1`:
```bash
$ echo "5 * 2 + 1" | cargo run flat_interp
Adding Literal(5) to pool
	pool = [Literal(5)], idx = 0
Adding Literal(2) to pool
	pool = [Literal(5), Literal(2)], idx = 1
Adding Binary(Mul, ExprRef(0), ExprRef(1)) to pool
	pool = [Literal(5), Literal(2), Binary(Mul, ExprRef(0), ExprRef(1))], idx = 2
Adding Literal(1) to pool
	pool = [Literal(5), Literal(2), Binary(Mul, ExprRef(0), ExprRef(1)), Literal(1)], idx = 3
Adding Binary(Add, ExprRef(2), ExprRef(3)) to pool
	pool = [Literal(5), Literal(2), Binary(Mul, ExprRef(0), ExprRef(1)), Literal(1), Binary(Add, ExprRef(2), ExprRef(3))], idx = 4

parsed expr = ((5 * 2) + 1)
Entering flat_interp:
root = ((5 * 2) + 1)
	setting state[0] = 5
	setting state[1] = 2
	setting state[2] = 10
	setting state[3] = 1
	setting state[4] = 11
final arena = [5, 2, 10, 1, 11]
final interpreted value = 11
```
