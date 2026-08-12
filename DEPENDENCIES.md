# Dependency inventory

| Dependency                 |            Version | License           | Owner                   | Purpose                                                                    |
| -------------------------- | -----------------: | ----------------- | ----------------------- | -------------------------------------------------------------------------- |
| Tauri                      |             2.11.x | Apache-2.0 OR MIT | desktop host            | secure desktop shell                                                       |
| React / React DOM          |             19.2.8 | MIT               | desktop UI              | rendering                                                                  |
| Vite                       |              7.3.6 | MIT               | desktop UI              | build/dev server; compatible with the pinned React plugin                  |
| esbuild                    |             0.28.2 | MIT               | desktop build           | Vite compiler binary; sole explicitly permitted pnpm install script        |
| React Aria Components      |             1.20.0 | Apache-2.0        | UI                      | accessible interaction                                                     |
| TanStack Router / Query    | 1.170.25 / 5.101.4 | MIT               | desktop UI              | route and query state                                                      |
| React Intl                 |            10.1.20 | BSD-3-Clause      | i18n                    | ICU formatting/catalogs                                                    |
| React Hook Form / Zod      |     7.85.0 / 4.4.3 | MIT               | desktop UI              | forms and untrusted input validation                                       |
| TypeScript                 |              5.9.3 | Apache-2.0        | all TypeScript packages | strict compile-time validation; pinned for typescript-eslint compatibility |
| ESLint / typescript-eslint |    10.8.1 / 8.67.0 | MIT               | all TypeScript packages | static analysis and explicit-`any` guard                                   |
| rusqlite                   |             0.40.2 | MIT               | EntityStore             | bundled SQLite and backup                                                  |
| tauri-specta / specta      |        2.0.0-rc.25 | MIT               | contracts               | typed IPC generation spike                                                 |
| TauRPC                     |              0.8.2 | MIT OR Apache-2.0 | spike only              | typed IPC comparison                                                       |
| uuid                       |             1.24.0 | Apache-2.0 OR MIT | Core                    | UUIDv7 IDs                                                                 |
| sha2                       |             0.11.0 | MIT OR Apache-2.0 | EntityStore             | migration checksums                                                        |
| keyring                    |              4.1.6 | MIT OR Apache-2.0 | Credentials             | native macOS/Windows credential storage; Rust-only opaque reference seam   |
