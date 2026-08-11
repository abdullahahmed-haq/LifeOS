# LifeOS implementation plan

Status: reviewed implementation baseline  
Date: 2026-08-11  
Delivery target for this workstream: repository foundation, mandatory Spikes A-C, and the first real Area vertical slice

## 1. Product requirements summary

LifeOS is a local-first, single-user-first desktop personal operating system for Windows and macOS. Its two primary outcomes are to make the next useful action obvious and make real progress visible. V0.1 ultimately covers planning, nested projects/tasks, progress, focus, reviews, knowledge, safe Obsidian synchronization, optional AI, MCP, extension points, backup/export, and English/Arabic parity.

The implementation contract is stricter than a screen prototype:

- Rust Core is the canonical authority. UI, AI, MCP, automation, and sync callers use the same typed application interface.
- SQLite is local and private. React and the Agent Runtime never receive SQL access or the database path.
- Mutations validate permissions and revisions and atomically create current state, domain events, audit events, versions, and undo records.
- Core organization remains usable offline and with AI, MCP, Agent Runtime, and Obsidian disabled.
- English LTR and Arabic RTL are equal from the first screen; canonical values are locale-neutral.
- User-owned Markdown must be patched loss-preservingly and never normalized through a generic YAML serializer.
- Secrets live in the native OS credential store and never in source, frontend storage, logs, or SQLite.
- The product targets at least 100,000 entities and 1,000,000 audit/domain events without architectural replacement.

## 2. Source and UI V001 inventory

### DOX contracts inspected

All 12 files in `../DOX` were inspected: `PRD.md`, `SCREEN_ARCHITECTURE.md`, `ARCHITECTURE.md`, `DATA_MODEL.md`, `AI.md`, `MCP.md`, `OBSIDIAN.md`, `I18N.md`, `UX_FLOWS.md`, `TEST_PLAN.md`, `IMPLEMENTATION_PLAN.md`, and `TECHNOLOGY_RESEARCH.md`.

Their combined requirements define:

- 44 core/management screens and 16 global overlays or shared surfaces;
- a hybrid relational entity model with stable identity, typed detail tables, generic relations, custom definitions, revisions, tombstones, events, audit, versions, undo, jobs, and derived indexes;
- deep modules for ApplicationCore, EntityStore, SearchIndex, VaultSync, Scheduler, Safety/Audit/Undo, AgentGateway, AgentRuntime, and Localization;
- local-first security boundaries, capability scoping, CSP, credential isolation, auditable external actions, and no public listener by default;
- explicit English/Arabic catalogs, logical layout, bidi isolation, locale-aware formatting, keyboard access, reduced motion, and translated states;
- staged milestones and nine mandatory spikes before their dependent feature-scale work.

### UI V001 current implementation

`../UI V001` contains exactly two files:

- `ElMessiri-VariableFont_wght.ttf`, SHA-256 `f0d2af38f9fe5612bfc20d1a9b26c4bbbcbf571842882f64e638c441e2cfedba`;
- `ui install.md`, which is zero bytes.

There is no `DESIGN.md`, `README.md`, `package.json`, HTML, CSS, JavaScript, TypeScript, React code, image/icon set, interaction code, responsive implementation, brand specification, or LTR/RTL reference implementation in the supplied UI directory. Therefore no deterministic reference screenshots or behavior recordings can be produced from UI V001. The available DOX visual language—muted blue-gray shell, warm paper workspace, teal action/progress, peach insight surfaces, rounded geometry, sparse charts, generous whitespace—and El Messiri are the only usable visual inputs. The first UI slice will implement that documented visual language without claiming pixel fidelity to missing source.

### Reusable assets and code

- Reusable code: none supplied.
- Candidate asset: El Messiri variable font. It will be copied only after its exact embedded/upstream license is verified and an asset-manifest entry is created.
- Existing repository work: none. The required GitHub remote is an empty repository with no branches or tags and has been cloned directly into this directory.

### Missing infrastructure

Everything under `Full App` must be created: workspaces, Tauri host, React app, Rust crates, migrations, contracts, localization, tests, CI, security configuration, release metadata, and documentation. Node 24.14.0 and pnpm are installed; Rust and the Apple native build toolchain must be installed/validated before the Rust and packaged-build gates can pass.

## 3. Proposed repository architecture

```text
Full App/
├── apps/
│   └── desktop/
│       ├── src/                    # React route/shell implementation
│       ├── public/                 # reviewed copied assets only
│       └── src-tauri/              # minimal Tauri adapter/host
├── crates/
│   ├── lifeos-domain/              # locale/transport/storage-free invariants and types
│   ├── lifeos-core/                # deep ApplicationCore command/query interface
│   ├── lifeos-store/               # deep EntityStore SQLite adapter, migrations, backup
│   ├── lifeos-search/              # normalization and search interface
│   ├── lifeos-vault-sync/          # loss-preserving Markdown seam (fixture-first)
│   ├── lifeos-safety/              # policy/receipt vocabulary where reusable
│   └── lifeos-test-support/        # temp DBs, IDs, clocks, fixtures
├── packages/
│   ├── contracts/                  # generated TypeScript contract client/types
│   ├── ui/                         # LifeOS-owned visual modules/tokens
│   ├── i18n/                       # catalogs, direction, formatters, bidi policy
│   ├── agent-runtime/              # optional sidecar; no canonical authority
│   └── test-fixtures/
├── fixtures/                       # search, database, hostile vault fixtures
├── scripts/                        # generation and static checks
├── migrations/                     # append-only embedded SQL source
└── .github/workflows/
```

The external seam is `ApplicationCore`: callers submit versioned commands/queries and receive canonical DTOs or tagged errors/receipts. Tauri is a thin adapter. Storage and search are internal seams with SQLite and deterministic test adapters. Tests use the same interfaces as production callers.

## 4. Ordered milestones, dependencies, and acceptance criteria

### M0 — Reference and repository baseline

Deliverables: source inventory, decisions/progress records, monorepo, pinned toolchains and lockfiles, dependency/license inventory, formatting/lint/type/test scripts, security-focused ignore rules, CI, asset manifest, exact commands, and initial threat boundaries.

Depends on: source inspection and safe Git reconciliation.

Acceptance:

- no DOX/UI source was modified;
- all copied assets have source hash and license evidence;
- minimal TypeScript, React, and Rust tests execute in CI configuration;
- dependency purpose, owner, version, and license are recorded;
- `origin` points to the required repository and work occurs on `codex/full-app-foundation`.

### M1 — Desktop, locale, and persistence foundation

Deliverables: Tauri 2 host, React 19/Vite strict TypeScript app, explicit Tauri capabilities/CSP, React Aria interaction behavior, TanStack Router/Query seams, FormatJS catalogs, pre-paint `lang`/`dir`, locale persistence, shell and overlay containers, Rust workspace, SQLite open/migration/integrity policy, and typed IPC spike.

Depends on: M0 and Spikes A/B/D portions needed by the shell.

Acceptance:

- app launches in English and Arabic and switches without restart;
- shell keyboard navigation and logical RTL/LTR placement work;
- no frontend SQLite/native-secret surface exists;
- the real Core health query crosses typed Tauri IPC;
- packaged build succeeds on the current supported platform or an exact toolchain blocker is retained.

### M2 — Entity and safety spine / Area vertical slice

Deliverables: default Workspace/User/Device, core Area type seed, `entities` and `areas`, revision validation, domain event, audit event, entity version, undo batch/operation, FTS projection, list/create/undo commands, receipts, localized Area UI, error/loading/empty/offline states, and database restart persistence.

Depends on: M1 and passing Spikes A-C.

Acceptance:

- React creates an Area through the typed Core interface and lists the canonical SQLite result;
- mutation state + event + audit + version + undo commit atomically;
- a stale expected revision returns tagged `CONFLICT_REVISION`;
- undo runs through ApplicationCore, returns a receipt, and removes/reverses the canonical Area safely;
- reopening the Core against the same development DB preserves state;
- temporary-database integration tests prove migration, persistence, atomicity, backup/restore, integrity, WAL, search, and undo;
- English LTR and Arabic RTL component/flow tests pass.

### M3 — Projects, tasks, goals, and relations

Deliverables: recursive projects/tasks, goals, milestones, semantic relations, safe moves, multiple basic views, manual Capture.

Depends on: M2.

Acceptance: recursive creation/moves reject cycles and stale revisions; standalone/multi-related tasks work; views share canonical queries; bulk changes are receipted/undoable; bilingual keyboard flows and large-tree tests pass.

### M4 — Today, calendar, habits, people/requests, events, and focus

Depends on: M3 and full RTL interaction spike.

Acceptance: date-only/timed/custom-anchor planning, backlog, recurrence, Day/Week/Month/Year, explicit restart-safe focus, and unfinished resolution work offline with undo and no time drift/double count.

### M5 — Progress, Home, reports, and reviews

Depends on: M3-M4.

Acceptance: deterministic progress and Smart Score goldens pass; configurable logical widgets render in both directions; daily/weekly reviews work without AI; large dashboard remains within measured budgets.

### M6 — Knowledge, customization, automation, backup/import/export

Depends on: M2 plus relevant domain slices.

Acceptance: mixed-language knowledge is searchable; custom definitions remain typed; automations cannot loop/bypass Core; backup/restore and open-format export pass corruption/interruption tests.

### M7 — Obsidian

Depends on: M6 and Spike E.

Acceptance: at least 100 hostile fixtures preserve unknown bytes/formatting; safe reserved-field patch, rename, conflict, watcher reconciliation, crash recovery, and disconnect pass on supported platforms before a real vault is connected.

### M8 — Agent Runtime and controlled AI

Depends on: M2, search/domain prerequisites, Spikes F/G, and native credential adapter.

Acceptance: packaged supervised sidecar uses authenticated bounded IPC, has no DB/vault path, two remote plus one fake/local provider satisfy contract tests, permissions/egress/receipts/undo work, and core remains complete with sidecar disabled/crashed.

### M9 — AI intelligence

Depends on: M8.

Acceptance: grounded analyses distinguish evidence and inference; memory is inspectable/removable; what-if is no-write; proactive suggestions throttle; subagents cannot expand authority or confirm mutations.

### M10 — MCP server/client

Depends on: M8 and Spike H.

Acceptance: official SDK current-protocol conformance, secure stdio, read resource/tool, confirmed revision-checked mutation, cancellation/limits/revocation/audit, fake external server, and Hermes compatibility pass with no public/raw SQL/path surface.

### M11 — Release hardening

Depends on: all accepted V0.1 slices and Spike I.

Acceptance: signed/installable Windows/macOS matrix, migration and update, offline daily loop, restore, failure campaigns, manual accessibility/visual reviews, and no unresolved Critical/High defects.

## 5. Mandatory technical spikes

| Spike | Gate | Timing |
|---|---|---|
| A: `tauri-specta` vs TauRPC | query, revision mutation, tagged error, event; deterministic no-`any` client; explicit capability | before M2 |
| B: bundled SQLite | migrations, FK, WAL, FTS5, concurrent reads/bounded writes, backup/restore/integrity, interruption recovery | before M2 |
| C: Arabic/English search | original preserved; exact/prefix/FTS ranking and controlled normalization goldens | before M2 |
| D: RTL interaction kit | React Aria/FormatJS and later DnD/calendar/grid keyboard equivalence in both directions | before broad UI/M4 |
| E: Obsidian round trip | 100+ hostile fixtures and byte-safe reserved patch/conflicts | before M7 |
| F: Agent sidecar | packaged/supervised/cancellable/versioned on target matrix | before M8 advanced work |
| G: provider normalization | streaming/tool/structured/cancel/quota/usage parity; SDK never executes Core tools | before M8 |
| H: MCP | official 2026-07-28 SDK resource/read/write/consent/cancel/limits/revoke/audit | before M10 |
| I: native E2E/release | WebdriverIO and signed-like install/update/restore matrix | before M11 |

## 6. Technical and product risks

- Missing UI implementation: exact visual fidelity cannot be measured. Mitigation: preserve documented visual language, log the source gap, use deterministic production screenshots as a new review baseline, and never claim prototype equivalence.
- Rust/Apple toolchain absent or incomplete: blocks native checks/build. Mitigation: install stable minimal Rust, verify Xcode Command Line Tools, and retain exact failing evidence if external installation is required.
- Typed-IPC ecosystem churn: `tauri-specta`/TauRPC compatibility can lag Tauri. Mitigation: bounded compile spike; generated checked-in TypeScript remains transport tooling, not domain authority.
- Scope explosion: V0.1 is extensive. Mitigation: enforce milestone gates and deliver only complete vertical slices.
- SQLite crash/backup errors: WAL copying can lose data. Mitigation: online Backup API, integrity checks, one write coordinator, temp DB tests, never raw-copy an active DB.
- Arabic over-normalization/bidi errors: can change meaning or reorder identifiers. Mitigation: derived versioned search text, original preservation, golden false-positive and mixed-bidi tests.
- Obsidian data loss: normal YAML emitters rewrite user content. Mitigation: raw-byte surgical patch module and fixture gate before real vault access.
- Sidecar/provider/MCP authority creep: could bypass policy or leak data. Mitigation: no DB/path, authenticated bounded IPC, Core-only tools, egress manifest, credential store, audit.
- Signing/updater credentials are external and irreversible release inputs. Mitigation: architecture/config only until certificates and protected CI secrets are explicitly supplied.

## 7. Development, test, preview, build, and release commands

Expected after M0/M1 initialization:

```bash
corepack enable
pnpm install --frozen-lockfile
pnpm dev                 # renderer preview
pnpm tauri dev           # native development app
pnpm typecheck
pnpm lint
pnpm test
pnpm test:e2e            # after Spike I
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm tauri build
```

Release builds run through a pinned GitHub Actions OS matrix only after signing/notarization credentials are provisioned. No unsigned artifact is published as an update.

## 8. Git and GitHub strategy

- Preserve the empty remote's history (there is currently none) and never rewrite future history.
- Work on `codex/full-app-foundation`; make small commits by repository setup, storage/Core, desktop/i18n, and tests/documentation.
- Run the full relevant gate before every commit that is described as verified.
- Never force-push, delete unknown files, commit secrets/databases/vaults/logs/build output/signing keys, or add/change a license without authorization.
- Push the working branch to `origin`; only merge/push `main` when compatibility/protection state is known and safe.

## 9. Requirement traceability

| DOX requirement family | Milestone(s) | Primary evidence |
|---|---|---|
| PRD 0-7 principles/scope | M0-M11 | plan, dependency rules, offline gates |
| PRD 8-10 / DATA_MODEL 1-6 identity/entities/relations | M2-M4 | migrations and Core integration tests |
| PRD 11-12 progress/score | M5 | deterministic golden/property tests |
| PRD 13-25 Home/Today/Calendar/Project/Task/Focus/Review/Capture/Search/Knowledge | M3-M6 | bilingual component and E2E flows |
| PRD 26 / OBSIDIAN | M7 | hostile round-trip and recovery matrix |
| PRD 27-34 views/widgets/templates/customization/automation/notifications/reports | M3-M6 | shared query, registry, safety tests |
| PRD 35-51 / AI | M8-M9 | provider/permission/egress/receipt contracts |
| PRD 52 / MCP | M10 | protocol conformance and interoperability |
| PRD 54-61 storage/events/security/backup/import/export | M1-M6 | DB, safety, restore, export suites |
| PRD 62-69 / I18N / SCREEN 61-99 | every UI milestone | catalog, RTL/LTR, bidi, keyboard/a11y tests |
| SCREEN 2-60 shell/screens/overlays | M1 and owning domain milestone | route/component acceptance matrix |
| SCREEN 100-105 relationships/families/DoD | M1-M11 | route map and reusable module tests |
| TEST_PLAN full matrix | every milestone/M11 | retained command and platform results |
| IMPLEMENTATION_PLAN sequence | M0-M11 | PROGRESS gates and decision records |
| TECHNOLOGY_RESEARCH Spikes A-I | before dependent milestone | spike reports and executable tests |

## 10. Definition of done for every vertical slice

A slice is complete only when all applicable items are present:

1. versioned typed contract with stable tagged errors;
2. domain validation and permission/revision rules in Rust;
3. append-only migration/repository work against a temporary real SQLite database;
4. atomic canonical state, event, audit, version, undo, and receipt for mutations;
5. production caller and tests both cross the same module interface;
6. React UI with loading, empty, error, dependency-disabled/offline, and success/undo states;
7. English and Arabic messages, pre-paint direction, mixed-bidi isolation, keyboard access, accessible labels, and reduced-motion behavior;
8. deterministic unit, contract, integration, and component coverage plus E2E when the platform spike permits;
9. diagnostics contain operation metadata but no personal content, secrets, raw SQL, or local paths;
10. fresh typecheck, lint, JS tests, Rust format, Clippy, Rust tests, and native build evidence, with every skip/platform limit stated;
11. updated `DECISIONS.md`, `PROGRESS.md`, and exact commands/recovery notes;
12. small logical commit pushed without secrets or generated build output.

## 11. Plan review result

This plan was cross-checked against every DOX contract and the user's requested sequencing. It keeps Spikes A-C ahead of M2 feature-scale code, defers real vault/AI/MCP authority until their safety gates, accounts for the absent UI prototype, makes the first Area slice real and reversible, and includes every required delivery command and traceability category. Temporary assumptions are isolated in `DECISIONS.md` for later review.
