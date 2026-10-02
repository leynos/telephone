# Telephone

This is a generated project using [Copier](https://copier.readthedocs.io/).

## Building from source

The repository's Cargo defaults and the Makefile development targets
(`make test`, `make lint`, `make typecheck` and the debug build) use the
parallel `rustc` frontend (`-Zthreads=8`) and, on Linux, the `mold` linker. On
Linux, install `mold` and `clang` before building, because the configuration
names them and a build without them fails at link time. A release build
(`make release`) and the coverage build use neither: they assign their own
`RUSTFLAGS`, which displaces the configuration's flags, so a shipped artefact
keeps the platform linker. To build with the platform linker directly, assign
an empty `RUSTFLAGS`:

```bash
RUSTFLAGS="" cargo build --release
```
