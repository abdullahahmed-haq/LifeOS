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

On macOS CI or another non-interactive packaging host, use `CI=true pnpm tauri build` so Tauri skips Finder-only DMG decoration. Local development bundles are explicitly ad-hoc signed for integrity checks; trusted release signing and notarization require separately provisioned Apple credentials.
