# Historical Research Controllers

`SHA256SUMS` pins the completed V10/V11 one-shot controllers and injected test
modules that remain at the top level for receipt compatibility.

Verify the archive from the repository root:

```bash
sha256sum --check scripts/research/SHA256SUMS
```

These files are reproducibility evidence, not active runtime entrypoints. Do
not edit or dispatch them from operational scripts. New research controllers
belong in a topic subdirectory unless a preregistered contract freezes another
path.
