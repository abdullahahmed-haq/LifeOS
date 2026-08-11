# Typed IPC comparison spike

This bounded probe compares the selected `tauri-specta` adapter in the desktop host with TauRPC `0.8.2`. Both candidates cover a query, a revision-checked mutation, a tagged error, an event, and generated TypeScript. The probe is isolated from the production Cargo workspace and does not add TauRPC to the shipped dependency graph.

Run:

```sh
cargo run --locked --manifest-path spikes/typed-ipc/taurpc-probe/Cargo.toml
```

The command regenerates `taurpc-probe/generated.ts`. The final comparison and selection are recorded in `dox/Plan/DECISIONS.md`.
