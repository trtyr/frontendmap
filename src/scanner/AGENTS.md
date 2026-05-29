# SCANNER SUBSYSTEM

## OVERVIEW

Facade pattern: `mod.rs` exposes single `pub fn scan() -> Result<FrontendMap>`. All submodules are private (`mod`, not `pub mod`). Scanning is regex-based, not AST — patterns matched directly against source file text.

## STRUCTURE

```
scanner/
├── mod.rs          # facade: orchestrates scan pipeline, returns FrontendMap
├── project.rs      # package.json parsing, framework detection (12+1)
├── component.rs    # JSX/TSX/Vue/Svelte component discovery + props + edges
├── route.rs        # 7 router pattern detectors (v5/v6/vue/angular/svelte/next/nuxt/sveltekit)
├── api.rs          # 8 HTTP call pattern detectors (fetch/axios/ky/got/etc.)
└── store_scan.rs   # 12 state store pattern detectors (zustand/redux/pinia/etc.)
```

## FLOW

```
scan(project_path)
  → project::scan_project()        # detect framework, parse package.json
  → component::scan_components()    # discover components, extract props, build edges
  → route::scan_routes()            # detect router patterns, build route tree
  → api::scan_api_calls()           # find HTTP call sites, extract method+endpoint
  → store_scan::scan_stores()       # find store definitions, track subscribers
  → assemble FrontendMap
```

Each sub-scanner receives `project_path` + framework context. All use `ignore::WalkBuilder` with `git_ignore=true` for file traversal.

## HOW TO ADD A NEW PATTERN

**New HTTP client (api.rs):**
1. Add regex pattern for the client's call syntax (e.g., `client.get(...)`)
2. Add arm to match block extracting method + endpoint from captures
3. Push `ApiCall` struct with detected metadata

**New store library (store_scan.rs):**
1. Add `StoreKind` variant to `model.rs`
2. Copy an existing `scan_*` function → rename, swap regex + `StoreKind`
3. Wire into `scan_stores()` with a file glob for the library's typical paths

**New router pattern (route.rs):**
1. Add regex for route definition syntax
2. Add arm extracting path + component from captures
3. If framework has file-based routing (next/nuxt/sveltekit), add branch to `file_to_route_path()`

## CONVENTIONS

- **Regex literals inline** — patterns defined as `Regex::new(r"...")` at call site, not extracted to constants
- **`scan_*` naming** — every scanner module exports one `pub fn scan_*(...)` entry point
- **Framework-aware** — scanners receive detected framework and branch behavior accordingly
- **File walking shared** — all scanners use `ignore::WalkBuilder`, not `std::fs::read_dir`
- **Naming quirk** — `store_scan.rs` not `store.rs` because `store.rs` already taken at crate root for save/load

## KEY ANTI-PATTERNS

**store_scan.rs: 12 near-identical functions.** Each `scan_zustand`, `scan_redux`, `scan_context`, etc. differs only in regex pattern and `StoreKind` variant. Fix: extract a generic `scan_store_by_pattern(path, regex, kind)` helper, reduce 611 lines to ~150.

**route.rs: 3 identical `file_to_route_path` branches.** Next.js, Nuxt, and SvelteKit file-to-route conversion logic is copy-pasted. Fix: unify into one function parameterized by prefix stripping rules.

**41 `Regex::new()` calls — all use `.expect("invalid regex pattern")`.** Previously `.unwrap()`. No bare unwraps remain. Consider `lazy_static!` or `OnceLock` for compile-once patterns to avoid re-compilation on every file visit.
