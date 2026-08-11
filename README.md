# LifeOS

Local-first desktop foundation for LifeOS.

## Commands

    corepack enable
    pnpm install
    pnpm dev
    pnpm tauri dev
    pnpm typecheck
    pnpm lint
    pnpm test
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    pnpm tauri build

The application database is created only by the Rust Core. Never point tests at a user database or an Obsidian vault.
