# Code Review: apartment

Date: 2026-06-07
Project Path: `D:\WebHatchery\RustGames\apartment`

## Review Scope

- Read updated project instructions from `AGENTS.md`, `../AGENTS.md`, and `CODE_STANDARDS.md`.
- Scanned Rust source for file sizes, TODO/FIXME/panic-style markers, module-root layout, long functions, logging in render/update paths, and selected gameplay state/action paths.
- Did not run `cargo` commands.
- Did not run tests or `publish.ps1`.

## Findings

### [High] Expired narrative events apply the wrong consequence, or no consequence at all

**Files:**
- `src/state/gameplay_turn.rs:148`
- `src/narrative/events.rs:192`
- `src/narrative/events.rs:428`

`end_turn()` auto-resolves expired response events with `process_choice(event_id, 0)`. That selects the first player-facing choice rather than the event's documented `default_effect`. Separately, `NarrativeEventSystem::handle_expired_events()` only marks events read/processed and never returns or applies `default_effect`.

Impact: ignoring an event can silently choose option 0, which may be a positive or destructive player choice, and some default consequences are never applied. Event deadlines therefore do not model the intended "no response" behavior.

Recommended fix: add an expiration path such as `expire_event(event_id) -> Option<NarrativeEffect>` that removes the pending event and returns `default_effect`, then apply that effect from gameplay state. Keep `process_choice` only for explicit player choices.

### [High] Monthly mail uses the previous month's rent income

**File:** `src/state/gameplay_turn.rs:91`

After `advance_tick()` returns the current `result`, mail generation reads income from `self.last_tick_result`, but `self.last_tick_result = Some(result)` is not assigned until `src/state/gameplay_turn.rs:237`.

Impact: month-end mail is off by one month. The first month reports zero rent income, and later mail reports stale income while expenses are pulled from the current tick.

Recommended fix: use `result.rent_collected` directly when generating mail for `self.current_tick`, or assign `last_tick_result` before mail generation if no other current-result ownership issue exists.

### [Medium] Dialogue money effects bypass transactions and financial totals

**File:** `src/state/gameplay_actions.rs:495`

`DialogueEffect::MoneyChange` mutates `self.funds.balance` directly for positive amounts and calls `self.funds.spend()` for negative amounts. Neither path records a typed `Transaction`, and the positive path does not update `total_income`.

Impact: the ledger, monthly reports, total income/expense stats, and any future accounting-dependent systems can drift from the actual balance after dialogue choices.

Recommended fix: route dialogue money through `Transaction::income` and `Transaction::expense`, using an appropriate `TransactionType` and `self.current_tick`.

### [Medium] Application panel emits debug logs from the render path

**File:** `src/ui/application_panel.rs:41`

When a filtered application list is empty but there are applications elsewhere, `draw_application_panel()` prints debug output every frame, including one line per application.

Impact: this can spam the native/browser console and degrade WebGL debugging/performance. It also violates the updated UI standard that UI components should remain pure rendering/action emitters.

Recommended fix: remove these `println!` calls, or gate one-shot diagnostics behind an explicit debug flag outside the draw path.

### [Medium] Many functions exceed the updated 100-line maximum

The updated `CODE_STANDARDS.md` sets an absolute max of 100 lines per function. Current examples above that limit include:

- `src/state/gameplay_actions.rs:16` - `process_action`, about 541 lines
- `src/ui/ownership_panel.rs:6` - `draw_ownership_panel`, about 325 lines
- `src/ui/hallway_panel.rs:7` - `draw_hallway_panel`, about 249 lines
- `src/ui/building_view.rs:201` - `draw_apartment_unit_sized`, about 237 lines
- `src/state/gameplay_turn.rs:12` - `end_turn`, about 227 lines
- `src/ui/career_summary.rs:5` - `draw_career_summary`, about 225 lines
- `src/state/menu.rs:92` - `draw`, about 224 lines
- `src/state/gameplay.rs:280` - `update`, about 209 lines
- `src/economy/costs.rs:65` - `process_upgrade`, about 195 lines
- `src/ui/application_panel.rs:7` - `draw_application_panel`, about 182 lines

Impact: these functions are difficult to review, test, and safely modify. Several combine rendering, layout, input, and state transition decisions.

Recommended fix: split by responsibility. For UI, extract section renderers and small action helpers. For state/action code, split action groups into methods or services by domain: leasing, ownership, dialogue, city, narrative, and money.

### [Medium] Two files exceed the updated 600-line soft limit

No Rust file is currently over the 800-line hard limit. Two files exceed the new 600-line soft limit:

- `src/ui/city_view.rs` - 635 lines
- `src/data/config.rs` - 622 lines

Impact: not a hard failure, but the updated standards treat these as restructure candidates before they grow further.

Recommended fix: split `city_view.rs` into map, market, portfolio/listing card modules. Split `config.rs` into domain-specific config structs or move large defaults closer to their domain modules while preserving JSON compatibility.

### [Medium] Module roots still use `mod.rs`, contrary to the updated standard

The updated standards prefer Rust named module source filenames and state: do not create new `mod.rs` files; when restructuring existing modules, prefer migrating `foo/mod.rs` to `foo.rs`.

Current `mod.rs` files:

- `src/building/mod.rs`
- `src/city/mod.rs`
- `src/consequences/mod.rs`
- `src/data/mod.rs`
- `src/economy/mod.rs`
- `src/narrative/mod.rs`
- `src/save/mod.rs`
- `src/simulation/mod.rs`
- `src/state/mod.rs`
- `src/tenant/mod.rs`
- `src/ui/mod.rs`
- `src/util/mod.rs`

Impact: this is an updated-standards compliance gap. It is not a runtime bug, but future restructuring should avoid adding more `mod.rs` files and should migrate existing roots opportunistically.

Recommended fix: migrate one module root at a time, e.g. `src/ui/mod.rs` to `src/ui.rs`, keeping child modules in `src/ui/` and updating no call sites beyond module declarations if done carefully.

## Scan Notes

- Files at or above 800 lines: none found.
- TODO/FIXME markers: none found under `src/`.
- `unwrap(`, `expect(`, `panic!`, `todo!`, `unimplemented!`: none found under `src/`.
- `std::fs` appears only in non-WASM branches or desktop-oriented fallback loaders in the reviewed scan; no immediate WebGL-only file-loading violation was identified from the search results.

## Suggested Fix Order

1. Fix narrative event expiration/default-effect handling.
2. Fix mail income to use the current tick result.
3. Route dialogue money changes through transactions.
4. Remove application-panel debug logging.
5. Continue splitting functions over 100 lines, starting with `process_action()` and UI draw functions.
6. Plan soft-limit file splits and `mod.rs` migration as separate mechanical refactors.
