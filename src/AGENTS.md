# SRC — CRATE ROOT

## OVERVIEW
Binary crate root. No lib.rs. `main.rs` is thin dispatcher; business logic lives in `commands/` and `scanner/`.

## MODULE MAP

| File | Lines | Purpose |
|------|-------|---------|
| `main.rs` | 197 | CLI dispatch — parse → load → match → call handler |
| `cli.rs` | 202 | clap derive: `Cli`, `Commands` enum, `QueryCommands` (15 variants), `NavCommands` (5), `AnalyzeCommands` (3) |
| `model.rs` | 156 | Data types: `FrontendMap`, `Component`, `Route`, `ApiCall`, `Store`, `Framework`, `ProjectInfo` — all serde, `Display` on enums |
| `store.rs` | 35 | `save()`/`load()` FrontendMap to `.frontendmap/map.json` |
| `commands/` | 772 | Handler functions split by domain: `query.rs` (431), `nav.rs` (195), `analyze.rs` (143) |
| `scanner/` | ~2000 | Facade submodule (6 files) — see `scanner/AGENTS.md` |

## ENTRY POINTS

```
main() → Cli::parse() → match Commands {
    Index   → scanner::scan() → store::save()
    Query   → store::load(".") → match QueryCommands → query::fn(...)
    Nav     → store::load(".") → match NavCommands   → nav::fn(...)
    Analyze → store::load(".") → match AnalyzeCommands → analyze::fn(...)
}
```

Handlers in `commands/` are `pub fn`, called from `main()` via `commands::{query, nav, analyze}`.

## CONVENTIONS

- **Separated commands** — query/nav/analyze each in own `commands/*.rs` module
- **Free functions** — handlers are `pub fn query_xxx(map: &FrontendMap, ...)`, not methods
- **`main.rs` is dispatcher only** — no business logic remains here
- **`model.rs` has Display impl** — `ComponentKind`, `StoreKind`, `Framework` all implement `Display`
- **All `.unwrap()` replaced with `.expect()`** — no bare unwraps in production code
- **Edition 2024** — requires Rust ≥1.85

## COMMANDS

Navigate by `Commands` variant → find handler in `commands/<group>.rs`:

| CLI Path | Module | Handler |
|----------|--------|---------|
| `frontendmap index` | main.rs | `scanner::scan()` → `store::save()` |
| `frontendmap query summary` | main.rs | inline in `main()` |
| `frontendmap query components` | query.rs | `query::query_entries(...)` |
| `frontendmap query similar` | query.rs | `query::query_similar(...)` |
| `frontendmap query deps` | query.rs | `query::query_deps(...)` |
| `frontendmap query impact` | query.rs | `query::query_impact(...)` |
| `frontendmap query flow` | query.rs | `query::query_flow(...)` |
| `frontendmap query scope` | query.rs | `query::query_scope(...)` |
| `frontendmap query path` | query.rs | `query::query_path(...)` |
| `frontendmap query export` | query.rs | `query::query_export(...)` |
| `frontendmap nav guide` | nav.rs | `nav::nav_guide(...)` |
| `frontendmap nav quality` | nav.rs | `nav::nav_quality(...)` |
| `frontendmap nav health` | nav.rs | `nav::nav_health(...)` |
| `frontendmap nav report` | nav.rs | `nav::nav_report(...)` |
| `frontendmap nav map` | nav.rs | `nav::nav_map(...)` |
| `frontendmap analyze deps` | analyze.rs | `analyze::analyze_deps(...)` |
| `frontendmap analyze fanout` | analyze.rs | `analyze::analyze_fanout(...)` |
| `frontendmap analyze tests` | analyze.rs | `analyze::analyze_tests(...)` |

**To find any command**: grep for `QueryCommands::` or `NavCommands::` or `AnalyzeCommands::` in `main.rs`, then follow to the handler function in `commands/`.
