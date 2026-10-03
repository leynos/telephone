# Developers' guide

This guide records the conventions contributors need that are not in the
README: how the repository builds, and why.

## The build standard

Development, test, lint, and typecheck builds use the parallel `rustc` frontend
(`-Zthreads=8`) and, on Linux, the `mold` linker (`-Clink-arg=-fuse-ld=mold`).
These are defaults in `.cargo/config.toml`, which Cargo discovers on its own,
so a bare `cargo build` gets them. `mold` ships for Linux only, so the linker
flag lives in a Linux-only table and macOS and Windows keep their platform
linker. Cargo selects one `rustflags` source rather than merging them, so every
source repeats the same flags apart from the linker.

An assigned `RUSTFLAGS` replaces the configuration's flags, so the Makefile
recipes that set it compose the standard's flags onto any inherited value (CI's
`setup-rust` exports one). Two builds are deliberately excluded: coverage
assigns `RUSTFLAGS` without the fast flags, because a measurement should not
depend on them, and the release recipe and workflow keep the platform linker,
because they assign `RUSTFLAGS` (even an empty value displaces the
configuration). Cargo has no per-profile `rustflags`, so a direct
`cargo build --release` takes the configuration's flags unless `RUSTFLAGS` is
assigned too.

On Linux, install `mold` before building: the configuration names it, so a build
without it fails at link time. rustc passes `-fuse-ld=mold` through its default
`cc` driver, which must be GCC 12.1 or newer, or clang, so `clang` is needed
only where `cc` is an older GCC (the users' guide shows how to select it). CI
installs `mold` through `setup-rust`'s `install-mold` input.
`tests/build_standard_contract.rs` holds the standard. It reads the
configuration sources, the commands `make -n` prints for each development target
on a Linux host and a macOS host (each keeping the caller's own `RUSTFLAGS`) and
for each coverage and release target on a Linux host, and the `setup-rust` steps
of the CI workflows (each must pass `install-mold`), so a flag lost through a
recipe or workflow edit fails there.

### Decision record

The development builds take the parallel frontend and, on Linux, `mold`, because
frontend time and link time dominate the edit-compile cycle and neither changes
what the code means. Release and coverage are excluded because a shipped
artefact should stay on the platform linker and a measurement should not depend
on the fast flags. The standard is enforced by a contract test rather than
prose, so a recipe or workflow edit that loses a flag fails the build.

### Cranelift

Exception: Cranelift is not the development-profile backend. The estate adopts
it only where the full suite passes under it, and that is not shown here on the
pinned `nightly-2025-09-16` (measured 2026-10-02): the crate has no tests to
measure it with (a build alone proves nothing about miscompilation or
unwinding). Revisit on the next toolchain bump: measure the whole suite under
the backend, with CI's environment, and adopt it if every test passes.
