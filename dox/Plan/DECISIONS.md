# LifeOS decision record

Date: 2026-08-11

This file records difficult, surprising, security-sensitive, or hard-to-reverse choices. Reversible implementation details stay in code and tests.

## D-026 — V0.1 scope and F0 completion boundary

Status: accepted

LifeOS now treats the complete documented V0.1 product as the delivery target: all local-first organization, planning, knowledge, safety, Obsidian, optional AI, MCP, bilingual UX, and signed-installer architecture. V0.2+ external integrations, cloud/device sync, accounts, collaboration, mobile, and public plugins remain outside the dependency graph. M1/M2 evidence is retained as a delivered foundation baseline, while their missing reusable capabilities are tracked and completed in a new F0 wave before feature-scale milestones.

## D-027 — Canonical presentation settings

Status: accepted for F0

Locale, theme, timezone, and week-start preferences are canonical workspace settings held by Rust Core in append-only migration 2 and protected by optimistic revisions. The renderer uses a tiny versioned `localStorage` mirror for locale/theme only, solely to set `lang`, `dir`, and resolved color mode before the first paint; it contains no secrets or entity data. Core hydration wins after startup, and every persisted change creates a domain event and audit event. Theme values are `light`, `dark`, or `system`; the renderer stores a resolved `data-theme` separately from that preference.

## D-028 — F0 route migration

Status: accepted for F0

TanStack Router is now the renderer's navigation authority. The Area page remains the root route to preserve the current runnable flow while Home, Today, Projects, Settings, and integration entry points are routable typed surfaces. Route placeholders are explicitly in-progress and do not count as feature delivery. New screens must be implemented as route-level domain slices rather than extending a single application file.

## D-029 — Area lifecycle as the generic-safety proving ground

Status: accepted for F0

Area archive, Trash, restore, and undo now use the same revision-checked Core transaction as create/update. Lifecycle state is included in Area version snapshots, removed from the active lexical projection while inactive, restored to the projection only when active, and recorded through domain/audit events. This is the proving ground for the generic lifecycle module; subsequent entities must reuse the same lifecycle semantics rather than replicate UI-only deletion behavior.

## D-030 — Settings writes keep immediate presentation feedback canonical

Status: accepted for F0

The renderer applies a locale or theme choice immediately so direction and appearance respond without waiting for IPC, then persists the complete settings revision through the existing Core command. Test doubles and future adapters must echo the canonical submitted settings (or return an explicit conflict); a stale response must not be treated as a successful preference update. The full General Settings form persists timezone and week-start through the same command, not a frontend-only store.

## D-031 — Safety policy seam and default posture

Status: accepted for F0

`lifeos-safety` owns deterministic permission evaluation behind `evaluate(actor, operation, policies)`. The Core resolves the actor category; React and external callers cannot forge it. Local user mutations default to Allow so the offline product remains usable; AI, MCP, automation, and Obsidian default to Ask. Exact enabled policy records override defaults and are revision-checked, audited, and emitted as domain events through append-only migration 3. A `deny` decision stops Core before the store mutation; `ask` maps to a stable `CONFIRMATION_REQUIRED` error until a confirmation scope is implemented.

## D-032 — Native credential store and opaque-reference persistence

Status: accepted for F0

Use `keyring` 4.1.6 (MIT OR Apache-2.0) as the Rust-only adapter to the macOS Keychain and Windows native credential store. SQLite migration 4 stores an opaque UUIDv7 credential reference, kind, revision, and revocation timestamp only; it never receives secret bytes. A Core mutation writes the OS secret before creating the reference and compensates by revoking the secret if persistence fails. Revocation removes the native secret before marking the reference revoked. Tauri may receive a user-entered secret for this narrowly scoped write command but never returns, logs, caches, or persists it in the renderer.

## D-033 — Area management remains the lifecycle reference slice

Status: accepted for F0

Areas now have real active, archived, and Trash collections. Archive and restore reuse the existing lifecycle transaction and revision check; no separate archive-only state exists in React. Editing stays inline and calls the typed canonical update command with the entity revision. This completes Areas as the UI reference slice for future generic lifecycle surfaces, while a reusable cross-entity lifecycle module remains an F0 follow-up.

## D-034 — Bounded, newest-first version history

Status: accepted for F0

Area history is exposed through a typed `AreaHistoryRequest` with a mandatory 1–100 limit. The store sorts snapshots newest-first and binds the cap directly into SQLite; React cannot request an unbounded history dump. The Version History route renders canonical snapshots and operation IDs only, never raw SQL or hidden payloads. Generalized history/audit pagination will reuse this bounded contract shape in later entity slices.

## D-035 — Privacy-safe, bounded audit timeline

Status: accepted for F0

The audit timeline is a separate typed Core query with the same mandatory 1–100 limit as version history. SQLite orders it newest-first and binds the cap in the query. Its public DTO deliberately contains only an audit ID, action key, actor category, and timestamp; affected-entity lists, operation IDs, raw payloads, actor IDs, and all internal diagnostics remain inside Rust Core. This keeps a user-facing safety surface useful without turning the renderer into a general audit-data export channel.

## D-036 — Complete typed route registry before feature implementation

Status: accepted for F0

The full V0.1 information architecture now has concrete TanStack Router route definitions, including detail, review, integration, and settings paths. Functional screens keep their owning domain route; every other route renders the explicit localized in-progress surface. This is navigation infrastructure only: a routable placeholder is never counted as a delivered product screen. The registry uses literal route paths so TypeScript continues to reject links to undefined paths as more domain screens replace placeholders.

## D-037 — Goal foundation uses directional fields, not containment

Status: accepted for first M3 increment

Migration 5 introduces the canonical Goal detail table with the documented horizon, status, optional local start/target dates, constraints, and derived lexical search entry. The first vertical slice creates and lists active Goals and supports undoing creation atomically; it does not claim the remaining Goal lifecycle, Area relation, progress, detail screen, or RelationGraph capabilities. Goals are directional entities, not Area children: their Area link will be a semantic relation when the shared RelationGraph module is introduced. Local dates are ISO calendar dates, validate leap years and order, and are deliberately kept distinct from instants/timezones.

## D-038 — Goal lifecycle extends the Area safety reference

Status: accepted for M3

Goal edit, archive, Trash, restore, and undo use revision-checked ApplicationCore commands and the same single SQLite transaction for canonical state, lexical search projection, entity version, domain event, audit event, undo batch, and inverse operation. Goal snapshots now include lifecycle timestamps so historical records represent the actual state at each revision. Area and Goal share lifecycle semantics but retain typed commands and detail-table writes until the planned generic entity-lifecycle module is extracted; this avoids a premature generic abstraction that would hide required entity-specific invariants.

## D-019 — Verified JavaScript toolchain pin

The running development host is Node `24.14.0` and pnpm `11.16.0`. Pin those exact versions in `.node-version` and `package.json` rather than retaining a stale Node 22 observation that causes every package command to warn. CI uses the same declared versions. Reassess this pin before a supported-platform release.

Status: accepted for M1

## D-021 — Generated binding lint boundary

`tauri-specta` generates a small transport runtime that presently contains three `any` annotations. Application-authored TypeScript remains linted with `no-explicit-any`; the deterministic generated `bindings.ts` transport is excluded from that one lint rule via the contracts package command. The generated file is still strict-typechecked and is regenerated by the Rust contract-export binary. Replace this exception if the upstream generator removes those annotations.

Status: accepted for Spike A evaluation

## D-022 — Binding generator is never a bundle target

The deterministic bindings generator is a developer-only binary. It now requires the explicit `binding-generation` Cargo feature, preventing Tauri's `--bins` release build from accidentally selecting it as the macOS app executable. The production default binary remains `lifeos-desktop`; contract generation is invoked explicitly in development/CI.

Status: accepted for Spike A evaluation

## D-020 — TypeScript compatibility pin

TypeScript `7.0.2` resolved during initial setup but is rejected by the selected `typescript-eslint` `8.67.0`, making the mandatory lint gate impossible. Pin TypeScript `5.9.3` (Apache-2.0, independently verified) in every workspace package. Strict compiler options remain enabled; this is a toolchain compatibility correction, not a relaxation of type safety.

Status: accepted for M1

## D-001 — Source precedence and missing UI reference

Status: accepted for foundation

DOX controls behavior, data, and security. UI V001 would control visual/interactions, but the supplied directory has no UI implementation: only one font and an empty Markdown file. The foundation uses the visual language explicitly repeated in ARCHITECTURE/UX_FLOWS and creates deterministic screenshots for review. It will not claim pixel equivalence or invent missing interaction evidence. If the intended UI source is supplied later, it becomes the visual reference and differences are reviewed rather than overwritten automatically.

## D-002 — Repository topology

Status: accepted

Goal edit, archive, Trash, restore, and undo use the same Core-owned revision,
version, audit, domain-event, and inverse-operation pattern proven by Areas.
The Goal schema remains migration 5 and is append-only; lifecycle state lives
on the common entity record so it remains consistent with the shared archive
and Trash posture.

## D-039 — Project creation begins the structural hierarchy spine

Status: accepted for the first Project slice

Migration 6 introduces the canonical Project table. A Project has at most one
optional `parent_project_id`, and Core validates that a requested parent is an
active Project in the same canonical database before the transaction creates
the child. Creation cannot introduce a cycle because the new Project does not
yet exist. Project creation, its lexical projection, event, audit entry,
version snapshot, receipt, and reversible typed undo are atomic.

This slice deliberately does not permit moving an existing Project: that
operation needs a bounded recursive cycle check and revision checks for both
the moved Project and affected parents, which will arrive in the next Project
hierarchy slice. Goals and Areas are not structural parents; their links remain
future semantic RelationGraph relations.

## D-040 — Project lifecycle retains its structural parent

Status: accepted for M3

Project edit, archive, Trash, restore, and undo use the same canonical safety
records as the Area and Goal reference slices. Edit intentionally changes only
the Project's own descriptive planning fields (title, priority, and local
dates); it does not move its parent. Lifecycle transitions retain the
structural parent ID, so a restored Project returns to the same hierarchy.
Existing-tree moves remain deferred until bounded recursive cycle detection and
multi-entity revision rules are implemented.

Use a pnpm workspace plus Cargo workspace with `apps/desktop`, deep Rust modules under `crates`, and shared TypeScript packages under `packages`. The Tauri host remains a thin adapter over `ApplicationCore`. The user-proposed structure is refined by separating domain, Core orchestration, store, search, safety, vault sync, and test support so dependencies point inward without proliferating pass-through layers.

## D-003 — Rust-to-TypeScript contracts

Status: accepted after Spike A

Both `tauri-specta` `2.0.0-rc.25` and TauRPC `0.8.2` compile against the pinned Tauri/Specta versions and generate a query, revision-checked mutation, tagged error, and typed event. Select `tauri-specta`: it preserves ordinary explicit Tauri commands, keeps `ApplicationCore` independent of the transport, and needs no additional frontend runtime. TauRPC's generated client is compact and contains no `any`, but adopting it would replace the command adapter with an async macro/router abstraction, require Tokio at the adapter seam, and add the `taurpc` JavaScript runtime. The isolated reproducible probe remains under `spikes/typed-ipc`; TauRPC is not shipped.

The selected generator is checked deterministically in CI. Its upstream transport runtime currently contains three `any` annotations, governed by D-021; no authored domain or application contract uses `any`.

## D-004 — SQLite access, migrations, and coordination

Status: accepted after Spike B

Use `rusqlite` with bundled SQLite and explicit SQL. React never receives `tauri-plugin-sql`. ApplicationCore owns one coordinated write path; bounded reads use separate connections only after concurrency tests. Start with a small append-only embedded migration runner; adopt `rusqlite_migration` only if its current exact version adds checksum/failure leverage without weakening recovery. Domain transactions do not wait on UI, network, AI, or filesystem operations.

## D-005 — WAL, integrity, backup, and restore

Status: accepted after Spike B

At open, enable and verify foreign keys, WAL, bounded busy timeout, and `synchronous=NORMAL` for ordinary local operation. Use SQLite's Online Backup API for consistent snapshots. Backup refuses to overwrite an existing path and records validation metadata. Restore validates schema/checksums, `integrity_check`, and foreign keys, then uses Online Backup into a newly created candidate path; it never replaces the active database. Candidate activation remains a later UI/application-lifecycle operation. Concurrent revision writes, interrupted initial migration rollback, checksum tampering, reopen persistence, backup, and candidate restore have temporary-database tests.

## D-006 — React routing, query caching, forms, and UI state

Status: accepted

Use TanStack Router for typed routes and TanStack Query for canonical query cache/invalidation; neither owns entities. Use React Hook Form for non-trivial forms and Zod only at untrusted/external TypeScript seams. Start with React context/state for locale and overlay presentation. Do not add Zustand until at least two independent surfaces demonstrably need shared ephemeral state. Use TanStack Table/Virtual when the first large collection requires them, not as shallow wrappers in the Area slice.

## D-007 — Localization and search normalization

Status: accepted after Spike C

Use React Intl/FormatJS catalogs. Set `lang`/`dir` before React's first paint and persist locale separately from canonical domain data. Preserve original user strings. Search maintains a derived, versioned lowercase matching projection that removes tatweel and Arabic combining marks and normalizes common Alef forms and Alef Maqsura/Ya only for matching. Do not use this normalization for identity, equality, authorization, or stored display. Prefix behavior and false-positive cases are golden-tested.

The Spike C suite covers English prefix search, Arabic exact search, mixed Arabic/English exact search, original display preservation, and FTS ranking through `ApplicationCore`. Normalized text remains derived and rebuildable.

## D-008 — Visual foundation and font

Status: provisional

Use the documented muted blue-gray/warm-paper/teal/peach visual vocabulary with CSS variables, logical properties, restrained motion, and El Messiri only if its exact license is verified. The UI source conflict cited in DOX cannot be resolved against missing files. A later supplied design or explicit brand decision may change tokens without changing module interfaces.

## D-009 — Obsidian Markdown/YAML preservation

Status: accepted architecture; implementation deferred to Spike E

VaultSync retains raw bytes and parses a semantic validation view, then surgically patches only reserved `lifeos_*` fields or one delimited LifeOS block. It preserves BOM, newline style, comments, key order, quoting, unknown fields, and body bytes. No generic YAML parse-and-serialize write path is allowed. Each write hashes/rechecks, writes and flushes a sibling temporary file, atomically replaces, re-reads/verifies, and records a receipt; otherwise it creates a conflict and leaves the original untouched.

## D-010 — Agent Runtime packaging and IPC

Status: architecture accepted; packaging deferred to Spike F

Use a separate TypeScript package bundled as a self-contained sidecar only after target-platform proof. Rust owns lifecycle and supplies a one-time credential. Use length-bounded framed JSON over private stdio initially; stdout is protocol-only and stderr is redacted logs. Handshake includes schema/version/capabilities. Enforce timeouts, cancellation, concurrency and message-size limits. Never pass SQLite path, unrestricted filesystem authority, provider secrets, or vault paths by default.

## D-011 — AI provider abstraction

Status: architecture accepted; implementation deferred to Spikes F/G

LifeOS owns a provider-neutral `AiProvider` interface and permission/tool orchestration. Vercel AI SDK Core may normalize streams/tool proposals; official provider SDKs are adapters for missing capabilities. No SDK executes Core tools. Remote/local provider availability and model capabilities are runtime data. At least two remote adapters and one fake/local adapter must pass the same contract before acceptance.

## D-012 — MCP SDK and compatibility

Status: architecture accepted; implementation deferred to Spike H

Use the official `modelcontextprotocol/typescript-sdk` and target MCP 2026-07-28 stateless self-contained requests with per-request version/capabilities. Let the SDK own backward compatibility; do not reproduce an older session handshake. Start with stdio, one read-only resource, one read tool, and one revision mutation with consent, cancellation, limits, revocation, and audit. No public unauthenticated listener or raw SQL/path/shell tool.

## D-013 — Tauri security

Status: accepted

Use explicit minimal capabilities per window and a strict CSP that permits only bundled application assets and Tauri IPC requirements. Do not enable broad shell, filesystem, HTTP, SQL, or opener permissions. Every added plugin/capability requires a threat/dependency decision. Secrets are accessed only in Rust through the native credential-store module; a fake adapter is used in tests.

## D-014 — Updates, signing, and release

Status: architecture accepted; external credentials deferred

Use official Tauri updater/action with mandatory signature verification, HTTPS metadata, migration compatibility, staged rollout, and retained provenance/SBOM/checksums. Do not create or commit private keys. macOS signing/notarization and Windows signing need external certificates and protected CI secrets and therefore remain blocked until explicitly provisioned. Unsigned local builds are development evidence, not published updates.

Local macOS development bundles use the explicit ad-hoc identity `-` so bundle resources are sealed and independently verifiable. Ad-hoc signing has no trusted publisher identity and is never release evidence; Developer ID signing and notarization remain externally blocked.

## D-015 — License and dependency policy

Status: accepted

Do not add a repository license because none is supplied or authorized. Each dependency requires purpose, owning module, exact version, exact package license, maintenance/security review, and non-duplication check. Commit lockfiles and a dependency inventory. FullCalendar Premium/AGPL code, generic YAML emitters for vault writes, direct frontend SQL, arbitrary scripting engines, and AGPL application code reuse are excluded.

## D-016 — Initial identity and development database

Status: provisional for M2

Seed one local Workspace, User, Device, and core Area type with deterministic stable IDs in the first migration; entity IDs are Core-generated UUIDv7. Production DB location is resolved by Tauri app-data and never exposed. CLI/integration tests require explicit temporary paths. A checked-in development command may use a `.local/` ignored DB, never a user vault/database.

## D-017 — First Area undo semantics

Status: accepted for M2

Creating an Area produces an inverse `area.trash`/remove-style typed undo operation. For the foundation slice, undoing a brand-new Area marks the entity deleted (retaining history) rather than physically deleting audit/event/version records. The UI list excludes deleted entities. Undo is a new validated, audited, revision-checked transaction and cannot erase history.

## D-018 — Temporary assumptions requiring review

Status: open

- Initial supported native proof platform is the present macOS ARM64 host; Windows/macOS Intel evidence belongs to CI/release matrices.
- The user has not supplied final brand specifications, source screenshots, signing credentials, updater endpoints, or a repository license.
- Optional database encryption, semantic vector adapter, final typography, cloud sync/CRDT, mobile framework, and public plugin distribution remain deliberately undecided.
- The exact credential-store fallback on platforms without a native service awaits its platform spike.

## D-023 — Frozen initial migration and append-only runner

Status: accepted

Migration 1 was already built and distributed in development evidence, so its exact SQL bytes and checksum are frozen even though it creates more foundation tables than the preferred incremental pattern. Rewriting it would reject existing databases. The runner now iterates an ordered append-only migration table, verifies every applied checksum, and applies each missing migration in an exclusive transaction. All future schema work must be a new numbered migration with focused integration tests.

## D-024 — Release database isolation and error disclosure

Status: accepted

`LIFEOS_DEV_DATABASE` is compiled into debug behavior only; release builds always resolve the canonical database beneath Tauri's application-data directory. Desktop setup propagates errors instead of panicking. SQLite and serialization errors crossing the Core boundary are redacted to `INTERNAL` plus a UUIDv7 operation ID; only controlled validation, conflict, not-found, and integrity explanations are exposed. This prevents local paths, SQL, or database contents from leaking through React IPC.

## D-025 — Compatible Vite line and explicit build-script allowlist

Status: accepted

Pin Vite `7.3.6` because `@vitejs/plugin-react` `5.1.0` officially supports Vite 4–7, not Vite 8. Both are MIT-licensed. pnpm 11's workspace-level `allowBuilds` permits only `esbuild` `0.28.2` (MIT), the Vite compiler binary; no other dependency install scripts are authorized. CI and local development use the same Node `24.14.0` pin.
