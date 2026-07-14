# Commercial Release Roadmap — Prototype → Shippable Game

*Compiled 2026-07-14 from a full-codebase audit (~18,700 LOC across 78 Rust files + 12 JSON content files), the sim-harness balance report, playtest feedback (`feedback.md`), and the dev_plan roadmaps.*

## Where the project actually stands

**Strong foundation.** The core loop (tenants, happiness, economy, upgrades, relationships) is genuinely functional, config-driven, and tested (~55 tests, clean CI with fmt/clippy/test/dual-target builds). The `theme.rs`/`widgets.rs` design system landed, the tenant-dilemma emergent stories work, error handling in production paths is clean (zero unwrap/panic outside test code), and the sim harness gives empirical balance measurement most indies never build.

**The gap to commercial is not "more engine" — it's four things:**
1. **Dead wiring** — several fully-modeled systems produce zero runtime effect (regulations, reputation, tenant life events, half the narrative effects).
2. **Content volume** — 5 archetypes / 3 buildings / 7 relationship events / 5 missions / 2 dialogues repeat within a single 36-month run.
3. **Presentation** — zero audio, minimal juice, no tooltips/settings/confirmations, missing-glyph emoji, dead i18n scaffolding.
4. **Release engineering** — no save versioning, achievements broken in shipped builds, no LICENSE/icon/installer/store integration.

---

## 0. Scope decision — DECIDED 2026-07-14

- [x] **Commercial shape:** $5 premium indie, **already live on itch.io**; Steam release later, only once the game is a full product (i.e. after Milestone 4 below). Implications: itch is the live update channel now (devlogs/patches keep the page alive); the Steam launch is the one-shot marketing moment, so everything in §10/§12 must be done before it; content targets in §4 can sit at the lower end of the ranges for a $5 title, but repetition within a single run is still disqualifying. Being *currently sold* also means the asset-rights audit in §12 is due now, not Steam-gated.
- [x] **Fantasy:** cozy-with-teeth — scarcity, hard choices, and emergent tenant stories. Every design task below serves that identity; anything that doesn't (e.g. finishing HOA/condo boards as a tycoon-ish layer) defaults to cut.
- [x] **Name: "Second Story"** — decided 2026-07-14. Collision check: no game of that exact name found; search noise from Square Enix's *Star Ocean: The Second Story R* is cross-genre — mitigate with a distinguishing subtitle on store pages (e.g. "Second Story — a landlord's second chance"). ("Tenants & Timber" was rejected: reads derivative next to the established landlord sim *The Tenants*, and "Timber" implies a materials mechanic the game doesn't have.)
  - [x] Window title (`main.rs`), web page title/h1 (`index.html`), README, CLAUDE.md updated 2026-07-14.
  - [ ] Remaining rename tasks: title-logo texture (`assets/textures` `title_logo` still says the old name), itch page rename + slug (deferred — page not yet marketed), `Cargo.toml` description metadata (§10), capture-harness env prefix stays `APARTMENT`, and **keep** the internal save id `apartment_manager` (renaming it orphans existing saves; revisit only alongside §9 migration work).

---

## 1. Ship-blocking defects (broken today, in every shipped build)

*Status: code fixes DONE 2026-07-14 (clippy `-D warnings`, `cargo fmt --check`, 57 tests, and a wasm release build all clean). Two items are non-code and remain open: web-save verification (needs a deployed web run) and index.html (owned separately — the loading-screen fix already landed there).*

- [x] **Achievements are silently empty in every shipped build.** ~~`narrative/achievements.rs:107-119` reads `assets/achievements.json` from disk with no `include_str!` fallback and no wasm branch.~~ **Fixed:** `load_achievements_config()` now uses the standard wasm-`include_str!` / native-disk-with-embedded-fallback pattern (matching `data/config.rs`), so achievements load in every shipped build. Removed the now-unused `std::fs` import.
- [x] **`AllTenantsLeft` loss condition is logically broken.** ~~`simulation/win_condition.rs:39-45` checks *current* occupancy to decide whether the building was *ever* lived in — always false when `tenants.is_empty()`, so the guard is inverted.~~ **Fixed:** added a `has_ever_had_tenant` flag to `GameplayState` (`#[serde(default)]`, latched in `end_turn` and in the sim harness), threaded through `advance_tick` → `GameTick::process` → `check_win_condition`, which now keys the loss off it. Added 3 regression tests (never-occupied → no loss; occupied-then-empty → loss; empty-before-check-tick → tolerated).
- [ ] **Web saves are unverified and plausibly broken.** *(Still open — needs a deployed-web run, not a code change.)* WASM persistence depends on custom `storage_*_extern` JS symbols (`macroquad-toolkit/src/wasm_storage.rs`) that must be provided by the external `mq_js_bundle.js`; the build masks missing symbols with `--allow-undefined`. Verify end-to-end that save/continue works in the deployed web build; add a smoke test to the release checklist.
- [x] **Web loading screen hides itself before the game loads.** ~~`index.html`~~ **Already fixed in `index.html`** (owned separately — it now waits for `window.wasm_exports` before hiding the overlay, and also syncs canvas size via a `ResizeObserver`). Left untouched per instruction.
- [x] **Menu "Quit" calls `std::process::exit(0)`** (`state/menu.rs`) — dead/incorrect on WASM. **Fixed:** the Quit button (hit-test + draw in `menu.rs`, and the "Quit Game" button in the pause menu `gameplay_views.rs`) is now `#[cfg(not(target_arch = "wasm32"))]` — hidden entirely on web (a browser tab has nothing to exit), fully functional on native. "Quit to Menu" remains on both.
- [x] **Committed `player_progress.json` contains a data bug** — ~~`completed_buildings` includes an empty-string id `""`.~~ **Fixed:** `mark_completed`/`unlock_building` now reject empty ids at the source, `load_player_progress` sanitizes any legacy blank entries on read (new `PlayerProgress::sanitize`), and the stale repo-root `player_progress.json` (git-tracked) + `savegame.json` (untracked) artifacts were removed — real saves live in `%LOCALAPPDATA%`.
- [x] **Apartment panel scroll has no max clamp** (`ui/apartment_panel.rs`) — **Not a defect on inspection:** a max clamp already exists — `draw_upgrades` computes `max_scroll` from content height and returns `final_scroll = current_scroll.min(max_scroll)`, which is stored as the panel offset. The stored value is always clamped to `[0, max]`; the only imperfection is a one-frame lag (the current frame draws with the pre-clamp value, up to one 30px wheel-notch of transient over-scroll), which is imperceptible. No change made.

---

## 2. Wire or cut: systems that exist but do nothing

This is the highest-leverage design work in the project — features that "look done in the type system" but are inert at runtime. For each: either wire it into gameplay or delete it (per the no-dead-code standard). Wiring is recommended for the first four; they're exactly the "teeth" the balance needs.

- [x] **Regulations / ComplianceSystem is entirely dead.** ~~no code path ever schedules an inspection or creates a violation.~~ **WIRED (2026-07-14).** Added a data-driven `RegulationsConfig` (`assets/config.json` → `regulations` block: pass threshold, random-inspection chance, fine multiplier, fix-deadline months, reputation deltas). New `ComplianceSystem::run_inspection` grades each regulation against the building's inspection score (min of average unit condition and hallway condition): below the threshold → citation, fine, `pending_fixes` deadline, and compliance-reputation loss; clean → reputation gain. `RegulationType::inspection_interval`/`name` extracted; `add_violation` de-test-gated. Orchestrated each month in `gameplay_turn::run_due_inspections` — runs scheduled (timer-driven) and random spot-check inspections on the active building, bills accrued fines (including missed-deadline escalations from `ComplianceSystem::tick`) as a required `InspectionFine` expense (can push toward bankruptcy — real teeth), and moves the neighborhood's visible reputation. 5 new tests. This directly attacks "too easy to make money": neglect now carries scheduled, escalating cost.
- [x] **Half the narrative effect types are silently dropped.** ~~`state/gameplay_effects.rs:63` `_ => {}` catch-all discards 6 of 12 variants.~~ **WIRED (2026-07-14).** Implemented all six handlers and **removed the catch-all** so any future variant is now a compile error, not a silent drop: `NeighborhoodReputation`/`RentDemand` mutate the matching neighborhood (clamped), `EconomyChange` moves `city.economy_health` (0.5–1.5), `BuildingHappiness` shifts the active building's tenants, `PropertyValue` scales the building's rent ceiling (`rent_multiplier`) as the value proxy, and `TriggerInspection` drives a complaint inspection through the item-2.1 engine (`execute_inspection` + immediate `bill_outstanding_fines`, both refactored out of the monthly path for reuse). City/neighborhood news events now have real consequences instead of being pure flavor. 5 new tests.
- [x] **Reputation is displayed and scored but cannot be moved by the player.** ~~`choice.reputation_change` never applied; `MissionReward::Reputation` shows floating text only.~~ **WIRED (2026-07-14).** Reputation is now a real, player-movable currency with sources, a write path, and a consequence: **Sources** — `NarrativeEventSystem::process_choice` now returns a `ChoiceOutcome { effect, reputation_change, neighborhood_id }`, and `gameplay_actions` applies the reputation change on every event resolution; `MissionReward::Reputation` now actually moves reputation (was floating-text only); and failed/clean inspections move it (item 2.1). **Write path** — new `apply_reputation_change(delta, neighborhood_id)` mutates the targeted (or active) neighborhood, clamped 0–100, with floating-text feedback. **Consequence** — applicant *volume* now scales with the active neighborhood's reputation via a data-driven `applications.reputation_influence` (neutral 50 → ×1.0, 0 → ×0.5, 100 → ×1.5), threaded through `advance_tick` → `generate_applications`. Reputation now feeds a live gameplay loop (cultivate it → more applicants → better tenant selection) instead of being a static scored stat. 4 new tests.
- [x] **Tenant life events (the emergent-story engine) have no generator.** ~~10 `LifeChangeType`s modeled but nothing generates them and `StoryImpact::LifeChange` is a no-op.~~ **WIRED (2026-07-14).** `LifeChangeType::impact()` maps each of the 10 life changes to concrete consequences (happiness / rent-tolerance / move-out-risk) composed from a **data-driven `LifeEventsConfig`** (`assets/config.json` → `life_events`: monthly chance + reusable magnitudes), and `LifeChangeType::eligible_for(archetype)` gates which are plausible per archetype. `apply_story_impact` now **expands the former `LifeChange` no-op** into those effects. New `state/gameplay_life_events.rs::generate_tenant_life_events` runs each month in `end_turn`: rolls the per-tenant chance, applies the impact, records a `StoryEvent` on the tenant's story, and surfaces the emergent beat via event log + floating text ("Sarah lost their job."). This is the engine behind the playtest's most-praised hook — job losses, new babies, and retirements now create the income/space pressure that forces real keep-or-evict dilemmas. 4 new tests.
- [x] **Missions: 3 of 6 goal types are never evaluated; `UnlockBuilding` reward is a stub.** ~~`MaintainHappiness`/`PerfectCollection`/`FullRepair` fell to `_ => {}`; `MissionReward::UnlockBuilding` did nothing.~~ **WIRED (2026-07-14).** `update_missions` now evaluates all six goals and the catch-all is removed: `MaintainHappiness` accrues consecutive months at/above the happiness threshold (resets on a bad month), `PerfectCollection` accrues months with no missed-rent event (reads `last_tick_result`, which `end_turn` now records *before* mission evaluation), and `FullRepair` completes when every unit and the hallway are at/above the repair bar. `MissionReward::UnlockBuilding(order)` now unlocks the matching building template in persistent player progress via new `unlock_building_by_order`. 2 new tests. **Deferred (UI, not dead logic):** missions still auto-accept — turning acceptance into a player choice needs a mission-select panel, tracked for the §6 UI milestone (making it manual now, with no UI, would just prevent missions from ever starting).
- [x] **Dialogue system: 1 of 3 types live.** ~~Only `FaceToFaceRequest` generated; `ConflictMediation`/`RentNegotiation` dead.~~ **WIRED (2026-07-14).** `generate_dialogues` now also produces both: `ConflictMediation` is sourced from a real **hostile relationship** in the tenant network (initiator complains about a housed neighbor; choices mediate / take-a-side / stay-out via the existing `RelationshipChange`/`HappinessChange` effects), and `RentNegotiation` fires when the building charges above baseline (`rent_multiplier > 1.1`) and a price-sensitive tenant (Elderly/Family/Student) is unhappy — offering a one-time rent break or holding firm. The network is threaded into the generator; all five `DialogueEffect` variants were already handled by the dispatcher. 1 new test.
- [ ] **Ownership/HOA voting is placeholder** — "satisfaction > 50 = YES" (`building/ownership.rs:81,94`) and the UI literally renders "Management options not yet implemented for this ownership type" (`ui/ownership_panel.rs:285`). Either finish condo/HOA as a real late-game mechanic or cut the surface area for 1.0.
- [x] **Tenant council has no mechanical effect** — ~~`should_form_council` fires floating text only.~~ **WIRED (2026-07-14).** A newly-formed council now takes a one-time collective action: it bargains the building's `rent_multiplier` down (data-driven `gentrification.council_rent_rollback`) and solidarity lifts tenant morale (`council_solidarity_happiness`). Latched via a new `council_formed` state flag so the effect applies once and can re-form if the landlord backslides. Pushing rent too hard on an unhappy building now has a concrete cost.
- [x] **Developer counter-offer is a stub** — ~~`effect: None`.~~ **WIRED (2026-07-14).** "Counter Offer" is now a gamble decided at event generation: ~60% the developer sweetens the deal ~25% (Money + SellBuilding), ~40% they walk and the sale is off. No more inert button.
- [x] **Dead animation:** ~~`panel_tween.current` never read.~~ **WIRED (2026-07-14).** The detail panel (apartment/hallway) now slides in from the right as the selection tween eases to 1.0 (`offset_x = (1 - panel_tween.current) * 60`).
- [~] **Dead config to reconcile:** (partial — 2026-07-14)
  - [x] `happiness.design_style_modifiers` now covers all five tiers — **Luxury (+15) and Opulent (+20) added** (config.json + `config_defaults.rs`); those upgrades finally grant the happiness they should.
  - [ ] `text_strings.json` never loaded / out of sync — deferred to §8 (localization/string-table work).
  - [ ] `ThemeConfig`/`LayoutConfig` loaded but never read — deferred (thread `Theme::from_config`, per the UI notes).
  - [ ] `config.json` `economy.design_upgrade_costs` / `soundproofing_cost` / `kitchen_renovation_cost` are stale duplicates of `upgrades.json` — left in place per "wire, don't delete" instruction; revisit when the config surface is cleaned up.
  - Two divergent final-score formulas exist (`win_condition.rs:64` vs `career_summary.rs:31`) — unify.

---

## 3. Game design & balance (the "too easy" problem)

The measured state (sim harness, 60 seeds × 3 strategies): **zero bankruptcies in 180 runs**, Greedy ends at ~20× starting funds with 0 upgrades needed, and even never-repairing Slumlord finishes comfortably. The playtest corroborates: no reason to reject tenants, staff impact invisible, no reason to sell. The 2026-07-03 tuning pass gave neglect real teeth (Slumlord now finishes last) — but the game is still unloseable.

- [ ] **Make failure reachable.** Target: a naive strategy should face genuine bankruptcy risk on Medium. Levers already in config: `base_monthly_cost_per_unit`, `property_tax_annual_increase`, critical-failure costs, plus the unwired regulation fines (§2). Re-verify every change with `cargo test balance_report -- --ignored --nocapture` — that harness already caught neglect being optimal once.
- [ ] **Make tenant selection matter.** Vetting works mechanically but nothing forces trade-offs — applicant pools should be scarce enough (reputation-gated, §2) that rejecting someone has a vacancy cost, and bad tenants should be tempting (higher rent offers from risky applicants).
- [ ] **Make staff visibly worth it.** The mechanics now exist (janitor decay offset, manager auto-approvals, security happiness) but the playtest found them unnoticeable — add explicit UI attribution ("Janitor saved $420 in decay this month" in the monthly report) so the value is legible, and tune salaries so hiring is a real decision.
- [ ] **Give selling a purpose.** "No reason to sell" — selling should fund the next acquisition in a portfolio strategy, or be forced by events (developer offers, regulation pressure). Ties to making the city/portfolio layer a real game (§4).
- [ ] **Difficulty settings.** 3 tiers currently differ only in unit count/condition/rent ceilings. Differentiate rules: event frequency, regulation strictness, market volatility, starting debt. Add an explicit difficulty selector separate from building choice.
- [ ] **Per-run seeded RNG.** All randomness flows through the toolkit's global RNG with no seed stored in `GameplayState` — store a run seed for reproducible runs (bug reports, daily-challenge potential, save-scumming policy).
- [ ] **Pacing curve.** 36 months at current tuning = full occupancy by month ~6 then 30 months of autopilot. Design mid-game escalation (year-2 regulation wave, gentrification pressure, aging-building failures) so the last 18 months aren't a victory lap.

---

## 4. Content expansion (everything repeats within one run today)

Current volume vs. what a commercial run-based sim needs:

| Content | Now | 1.0 target (rough) |
|---|---|---|
| Tenant archetypes | 5 | 12–15, with per-archetype arcs |
| Relationship events | 7 | 30–40 |
| Tenant request types | ~6 | 15–20 |
| Narrative/news events | ~13 (hardcoded, mostly flavor) | 50+, data-driven, all with real effects |
| Dialogue bodies | 2 | 30+ across all 3 dialogue types |
| Missions | 5 | 20–30 across a campaign |
| Buildings / campaign | 3 | 6–10 with distinct identities |
| Neighborhoods | 4 | 6–8 with mechanical personality |
| Achievements | 6 | 25–40 |

- [ ] **Move hardcoded content to JSON.** Narrative event templates (`events.rs`), dialogue bodies (`dialogue.rs`), and missions (`missions.rs`) are hardcoded Rust — migrate to `assets/*.json` per the project's own data-driven hard rule before scaling content, so writing content doesn't require recompiling.
- [ ] **Campaign arc.** 3 buildings = the whole game. Design a campaign with distinct building identities (rent-controlled walk-up, aging luxury tower, converted warehouse…) where each teaches/tests a different system, plus mission-driven unlocks (the stubbed `UnlockBuilding` reward).
- [ ] **Endgame mode.** After the campaign: endless/sandbox mode with the multi-building city layer as the driver (portfolio play is mostly built but under-used), or a scored challenge mode leveraging the seeded RNG.
- [ ] **Name pools** (~10 first names per archetype) will visibly repeat — expand, and add portrait variety (5 tenant portraits currently).

---

## 5. Audio (currently: literally none)

The game is completely silent — no music, no SFX, no audio assets, zero `macroquad::audio` calls — while the toolkit ships a complete, unused `SoundManager` (`macroquad-toolkit/src/audio.rs`: sfx/music volumes, asset-pack loading, mute-on-hidden).

- [ ] **Wire the toolkit `SoundManager`** into `Game`/`AssetManager` startup.
- [ ] **SFX pass (~20–30 sounds):** button click/hover, cash in/out, rent collected, repair/hammer, upgrade complete, tenant move-in/out, door knock (dialogue), event modal open, mail arrival, warning/negative sting, achievement, turn-advance whoosh, win/lose stings.
- [ ] **Music:** menu theme + 2–3 gameplay loops (calm/tense variants keyed to funds or crisis state fits the cozy identity), win/lose cues. Budget for licensed/commissioned tracks — see licensing in §12.
- [ ] **Volume controls** (master/music/SFX) in the settings menu (§8) with persisted values; mute-when-tab-hidden on web (toolkit supports it).

---

## 6. UI/UX & presentation polish

The theme/widget system is a real foundation; these are the gaps on top of it.

- [ ] **Tooltips — highest-value single UI feature.** No tooltip renderer exists at all, yet `text_strings.json` already authors tooltip copy for condition/design/noise/etc. A management sim without hover-explanations of its numbers is not shippable. Add a toolkit-level hover-tooltip widget (candidate for `macroquad-toolkit` per the reach-for-toolkit-first rule).
- [ ] **Confirmation dialogs for destructive actions** — sell building, evict, reject applicant, quit-to-menu with unsaved turn. Currently everything fires instantly and there is no undo anywhere.
- [ ] **Scrolling everywhere it's needed** — only the apartment panel scrolls (unclamped, §1); mail, applications, event log, and city lists will truncate as content grows. Add scrollbars (visible affordance, not just wheel).
- [ ] **Keyboard support** — the entire game is two keys (Space, Esc). Add number/arrow-key panel navigation, Enter-confirm on modals, and hotkeys for common actions. (Full controller support: probably post-1.0 for a mouse-first sim, but keyboard is table stakes.)
- [ ] **Fix the missing-glyph emoji (~39 sites, 8 files)** — 🎉💸🔒✓⚠★ etc. render as tofu boxes because neither Rajdhani nor macroquad's default font has those glyphs (visible on the title screen's "🔒 LOCKED"). Replace with the icon textures already shipped in `assets/textures/` or ASCII/drawn glyphs.
- [ ] **Fix font inconsistency** — code paths that build raw `TextParams` with `..Default::default()` (e.g. `ui/city_view.rs:13`, `ownership_panel.rs`, parts of `state/gameplay*.rs`) fall back to macroquad's built-in font instead of Rajdhani, so the UI mixes two typefaces. Route everything through the themed text helpers.
- [ ] **Juice pass** (dev_plan 07_VISUAL_POLISH was never finished): panel slide-in (revive or replace the dead `panel_tween`), button press/hover feedback, money-count animations, happiness-face icons (textures exist, unused), state-transition fades (transitions are instant enum swaps), light particles for celebrations, subtle screen shake for disasters. FloatingText (28 call sites) is the only feedback effect today.
- [ ] **Use the art you have.** 59 textures load but only 8 files render any — most panels are rect-and-text. Portraits in tenant lists/dialogues, event images in the modal (9 exist), neighborhood art in city view, design-tier imagery in the apartment panel.
- [ ] **Responsive layout.** Fixed 1280×720 design with hardcoded pixel font sizes and fixed 280px menu cards that overflow narrow browsers. The toolkit's `set_ui_text_scale`/`set_min_ui_font_size` are unused — wire them, and verify common desktop sizes + typical browser embeds. Touch input for web/mobile: decide explicitly in scope (§0) rather than by default.
- [ ] **Restyle the stragglers** — title screen and pause menu are hand-rolled with `from_rgba` colors and duplicated hit-testing instead of `theme.rs`/`widgets.rs`; career summary mixes legacy `colors::` constants.

---

## 7. Onboarding

A real tutorial exists (3 milestones, mentor/rival NPCs, `hints.json` context system) but is thin for cold commercial players.

- [ ] **Interactive step gating** — spotlight/point-at-the-button affordances instead of passive floating-text hints; block-or-guide on the first repair, first application, first rent change, first turn-end.
- [ ] **Extend milestone coverage** to one guided touch of each core system: rent adjustment, vetting, upgrade purchase, staff hire, event choice, mail, city view.
- [ ] **Skippable & replayable** tutorial toggle; controls reference in-game (currently only in `index.html`).
- [ ] **Legibility of systems** — expose the *why* behind numbers (happiness factor breakdown exists in code; surface it fully with tooltips §6, and monthly-report attribution §3).

---

## 8. Settings, accessibility, localization

- [ ] **Settings screen** (currently: a fullscreen toggle in pause is the only setting): audio volumes, fullscreen/resolution, UI text scale, autosave cadence, reduced motion, colorblind-safe palette toggle. Persist to a settings file via the toolkit persistence layer (works on both targets).
- [ ] **Decide on localization, then make the string layer real.** `text_strings.json` is authored but **never loaded — zero references in src/**; all ~163 draw-text call sites use hardcoded Rust strings. Even for English-only 1.0, routing user-facing text through one loaded string table is the prerequisite for any future localization and fixes the current drift (JSON knows 3 design tiers, game has 5). If localizing: font coverage beyond Latin, layout tolerance for longer strings.
- [ ] **Accessibility minimums:** scalable text (toolkit hook exists), colorblind-safe status colors (happiness/condition meters are color-coded), no information conveyed by color alone, reduced-motion option once the juice pass (§6) lands.

---

## 9. Save system & persistence robustness

Native saves are already commercial-grade in *location and atomicity* (`%LOCALAPPDATA%\apartment_manager\`, temp-file+rename). The gap is durability across updates — the #1 post-launch risk.

- [ ] **Save versioning + migration.** `GameplayState` has no version field; the toolkit's migration API (`load_from_slot_with_migration`, `peek_version_value`) exists and is unused; only 3 of ~20 core state fields have `#[serde(default)]`. **Any added non-optional field in any core struct breaks every existing save.** Add a version field now (before launch), use the toolkit migration path, and default-annotate aggressively.
- [ ] **Fail loudly, not silently.** A failed load currently `eprintln!`s and leaves the Continue button rendered-but-dead (`state/menu.rs:67-71`) — silent data loss from the player's perspective. Show an error, keep a backup copy of the last-good save, offer "start fresh."
- [ ] **Multiple save slots** — toolkit `save_to_slot`/`get_save_slots` API exists unused; one slot means one run per player per machine.
- [ ] **Save-system tests** — currently 1 round-trip test. Add: old-schema fixture loads (freeze a pre-release save as a fixture), corrupted-file handling, version-migration paths.
- [ ] **Steam Cloud** (if Steam, §10): the AppData location makes this a config-only step, but plan for it.

---

## 10. Platform, distribution & storefront

Currently absent, all needed for any paid release:

- [ ] **Project metadata:** `Cargo.toml` is `version 0.1.0` with no description/license/authors; adopt real semver + a CHANGELOG; surface the version in-game (menu corner) and in bug reports.
- [ ] **Windows packaging:** exe icon + version resource (`winres`/`embed-resource` via `build.rs`), proper window title (§0), signed installer or at minimum a clean zip layout; test on a machine without dev tooling.
- [ ] **LICENSE file + asset licensing audit** — see §12; blocking for both Steam and itch.
- [ ] **Steam integration** (if chosen): Steamworks SDK (achievements sync — after fixing §1, the internal system is ready), cloud saves, rich presence optional; store page assets (capsules, screenshots, trailer), demo build (the WebGL version is a natural free demo).
- [ ] **Crash reporting & analytics (native):** the web build has a bug-report widget (Project Roost); native has nothing. At minimum: panic-hook that writes a crash log with version + seed and points users to it. Opt-in telemetry only, disclosed.
- [ ] **Web distribution hardening:** `mq_js_bundle.js` lives outside the repo (`../shared-assets/runtime/`) — vendor the exact storage-enabled bundle into the project so web deploys are reproducible; verify localStorage saves (§1); loading screen fix (§1). The itch.io page already exists — keep it updated per-milestone (devlogs are itch's discovery mechanism) and decide whether the itch build stays the paid full game or becomes the free demo once Steam launches.

---

## 11. QA, performance & release engineering

- [ ] **Playtesting program.** One playtest review exists (`feedback.md`). Commercial tuning needs recurring external playtests each milestone; keep using the sim harness for economy regressions and add its report to CI as a tracked artifact.
- [ ] **Grow the test surface where it's thin:** save/migration (§9), win/loss conditions (would have caught the §1 bug), mission goal evaluation, a full-36-month integration test asserting the report/event pipeline; the README's own wishlist (month-step regression tests for rent/occupancy/happiness; application/assignment validation against duplicate tenants or stale leases) is still open.
- [ ] **Web smoke test in CI** — wasm is built but never executed; a headless browser boot-and-save check would catch the §1-class web breakage.
- [ ] **Visual regression** — the screenshot capture harness exists (2 static captures in `docs/verification/`); automate scene captures per PR and diff them.
- [ ] **Performance instrumentation** — nothing measures frame cost today (fine at current scale, but add an FPS/frame-time debug overlay before the juice pass and city-scale content growth; watch the `Building` clones in `sync_building()`/`save_building_to_city()` if buildings get bigger).
- [ ] **Release checklist doc** — build both targets, run harness, load previous-version save fixture, web save check, fresh-machine install check.
- [ ] **Doc hygiene:** dev_plan docs reference stale `h:/WebHatchery` paths; `random_events.rs:24-27` carries a stale "MVP, for now" comment for behavior that's now implemented.

---

## 12. Legal & business

- [ ] **AI-generated art disclosure & rights.** All 59 textures trace to AI image prompts (`graphics_prompts.json`, `graphics_batch.json`). **The game is already being sold at $5 on itch.io, so commercial-use rights need verifying now**, not at Steam submission. Steam additionally requires AI-content disclosure at submission and holds you responsible for rights; verify the generator's terms permit commercial use, and decide whether key art (store capsule, title) should be commissioned human work for both legal comfort and marketing quality.
- [ ] **Font license:** Rajdhani is OFL (fine to embed) — record it in a THIRD-PARTY-LICENSES file along with all Rust crate licenses (macroquad is MIT/Apache; generate the list with `cargo-about`/`cargo-license`).
- [ ] **Music/SFX licensing** for everything added in §5 — keep receipts/licenses in-repo.
- [ ] **Company/tax basics** for selling (store payee setup), privacy policy if any telemetry/bug-reporting is in the native build.

---

## Suggested sequencing

**Milestone 1 — "Honest prototype" (fix what's false):** §1 ship-blockers, §2 wire-or-cut decisions executed (regulations, reputation, narrative effects, life-event generator; cut what won't ship), unified score formula. *The game now does what its code claims.*

**Milestone 2 — "Feels like a game":** §5 audio, §6 tooltips + confirmations + juice + font/emoji fixes, §7 tutorial deepening, §3 balance pass so failure is real (harness-verified). *A stranger can be handed the game cold.*

**Milestone 3 — "Worth money":** §4 content expansion to targets (data-driven first), campaign arc + endgame mode, §8 settings/string table. *Content outlasts a single run.*

**Milestone 4 — "Shippable product":** §9 save versioning + slots, §10 packaging/store/platform work, §11 QA hardening + external playtests, §12 legal. *Launch.*
