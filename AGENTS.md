# PROJECT KNOWLEDGE BASE

**Generated:** 2026-05-29
**Edition:** Rust 2024
**Crate:** `frontendmap` (0.1.0) — binary only, no lib.rs

## OVERVIEW

CLI tool that indexes a frontend project (React/Vue/Svelte/etc.), producing a `map.json` of components, routes, API calls, and state stores. Supports query, navigation, and static analysis subcommands.

## STRUCTURE

```
./
├── src/
│   ├── main.rs        # CLI dispatcher (197 lines)
│   ├── cli.rs         # clap derive CLI definition (202 lines)
│   ├── model.rs       # data types: FrontendMap, Component, Route, Store, etc.
│   ├── store.rs       # save/load FrontendMap to .frontendmap/map.json
│   ├── commands/      # handler functions split by command group
│   │   ├── mod.rs     # module declarations: query, nav, analyze
│   │   ├── query.rs   # query_entries, query_similar, query_deps, query_impact, etc.
│   │   ├── nav.rs     # nav_guide, nav_quality, nav_health, nav_report, nav_map
│   │   └── analyze.rs # analyze_deps, analyze_fanout, analyze_tests
│   └── scanner/       # frontend project scanner subsystem
│       ├── mod.rs     # facade: pub fn scan() → FrontendMap
│       ├── project.rs # package.json parsing, framework detection
│       ├── component.rs  # JSX/TSX/Vue/Svelte component discovery
│       ├── route.rs      # router pattern detection (React Router, Vue Router, etc.)
│       ├── api.rs        # HTTP call site detection (fetch, axios, ky, etc.)
│       └── store_scan.rs # state store detection (zustand, redux, pinia, etc.)
├── .frontendmap/      # app output dir (map.json written here)
├── .crabmap/          # crabmap index cache (not gitignored — see anti-patterns)
└── Cargo.toml
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add new CLI command | `src/cli.rs` (define) → `src/commands/<group>.rs` (implement) → wire in `src/main.rs` | handlers in commands/ |
| Add new scanner pattern | `src/scanner/<domain>.rs` | each scanner file handles one concern |
| Change data model | `src/model.rs` | serialized to map.json via serde |
| Output format | `src/store.rs` | save/load JSON |
| Frontend framework support | `src/scanner/project.rs` (detection), route/component/api/store scanners (syntax) | |

## CODE MAP

**Entry points:**
| Entry | File | Description |
|-------|------|-------------|
| `main()` | `src/main.rs:11-200` | CLI dispatch — match on Commands enum |
| `scanner::scan()` | `src/scanner/mod.rs:11` | single entry into scanner subsystem |
| `store::save()` | `src/store.rs:6` | serialize FrontendMap to JSON |
| `store::load()` | `src/store.rs:20` | deserialize from `.frontendmap/map.json` |

**Hot symbols (most connected):**
| Symbol | Kind | File | Degree | Role |
|--------|------|------|--------|------|
| `QueryCommands::Path` | enum variant | `src/cli.rs:135` | 47 | heaviest CLI command |
| `FrontendMap` | struct | `src/model.rs:6` | 33 | root data model |
| `main` | function | `src/main.rs:11` | 22 | CLI dispatch |
| `Store` | struct | `src/model.rs:12` | 20 | state store model |

**Feature clusters:**
| Cluster | Files | Symbols | Degree |
|---------|-------|---------|--------|
| `model` | 1 | 77 | 255 |
| `main` | 1 | 23 | 174 |
| `cli` | 1 | 33 | 128 |
| `scanner/store_scan` | 1 | 14 | 116 |

## HEALTH

- **Health score:** 94/100
- **Cycles:** none
- **God modules:** `src/scanner` (49 meaningful symbols)
- **Dead code:** none detected

## CONVENTIONS

- **No lib.rs** — pure binary crate. All logic lives in `src/main.rs` (query, nav, analyze functions) and `src/scanner/`.
- **Scanner facade** — `src/scanner/mod.rs` exposes single `pub fn scan()`. All subscanner modules are `mod` (private).
- **Editions 2024** — uses latest Rust edition.
- **No custom lint/format rules** — default rustfmt + clippy.
- **No tests** — 0 test coverage. No `#[cfg(test)]`, no `tests/`, no dev-dependencies.
- **No CI** — no GitHub Actions, no Makefile, no justfile.

## ANTI-PATTERNS (THIS PROJECT)

- **main.rs is a dispatcher** (197 lines) — business logic lives in `src/commands/` (772 lines) and `src/scanner/` (~2000 lines). All `.unwrap()` replaced with `.expect()`.
- **Code duplication (partially fixed)** — `store_scan.rs` reduced from 611→241 lines via generic helper. `route.rs` 3 identical branches merged. `project.rs` `all_deps` extracted to `get_all_deps()` helper.
- **`scanner/store_scan.rs` naming** — inconsistent with sibling files (`route.rs`, `api.rs`). Named `store_scan` only because `store.rs` already taken at crate root.
- **Fragile enum display** — uses `serde_json::to_string(&kind).unwrap_or_default().trim_matches('"')` at 6 sites. Implement `Display` trait instead.
- **`.gitignore` incomplete** — only ignores `/target`. `.frontendmap/` and `.crabmap/` are not ignored and will be committed.
- **104 `.to_string()` calls, 35 `.clone()` calls** — consider `Cow<str>`, `&str` keys, or `Copy` derives where possible.

## UNIQUE STYLES

- `scanner/` uses regex-based source analysis (not AST) — patterns are strings matched against frontend source files.
- All CLI subcommands (Query, Nav, Analyze) dispatch to free functions in `main.rs`, not methods on structs.

## COMMANDS

```bash
cargo build                          # debug build
cargo build --release                # release build
cargo run -- index                  # index current directory
cargo run -- query summary          # show project summary
cargo run -- nav map                # compact project overview
cargo run -- analyze fanout         # fan-in/fan-out analysis
```

## NOTES

- `edition = "2024"` requires Rust ≥1.85.
- Output directory `.frontendmap/` is created at project root (not in target/).
- Framework detection relies on `package.json` dependencies; unknown frameworks fall back to directory structure heuristics.
