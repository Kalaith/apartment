# Second Story — Review Remediation Plan

This plan turns the August 2026 game review into an ordered implementation
program. A milestone is complete only when its acceptance checks pass and its
changes have been committed separately using the RustGames commit convention.

## Outcome

Second Story should finish this program as a coherent 36-month building-care
game in which:

- every tenant, application, relationship, and transaction belongs to an
  unambiguous building and unit;
- every visible feature is reachable, communicates its consequences, and uses
  the same underlying rules as the simulation and balance harness;
- investing in tenants and the building is measurably better than minimal
  upkeep, while neglect remains a risky but legible strategy;
- the interface presents the building, decisions, finances, and narrative in a
  responsive hierarchy instead of exposing disconnected panels;
- Windows and WebGL builds pass the normal publisher after every milestone.

## Milestone 1 — Authoritative addresses and safe transactions

Make `building_id + apartment_id` the identity of a unit everywhere. Existing
saves must migrate to building zero, while new tenants and applications must
record the active building. Monthly processing must touch only the active
building's tenants and applications; inactive properties continue through the
explicit portfolio simulation.

Condo conversion and buyback must be atomic. A conversion must clear or move
its tenant deliberately, make the unit unavailable to leasing, and record any
displacement. A buyback must check funds before ownership changes and return a
clean, leasable vacancy. Sold units must not count as vacancies, occupied
rentals, or full-occupancy targets.

Input and tutorial state are simulation invariants too: pause and blocking
modals must prevent month advancement and click-through, tutorial hints must
fire once per qualifying month, and the final tutorial milestone must actually
complete.

Acceptance checks:

- A tenant in Building 0 / Unit 0 never pays the rent of Building 1 / Unit 0.
- Applications cannot be accepted into another building, an occupied unit, or
  a sold unit; stale applications are rejected without losing valid state.
- Unaffordable condo buyback changes nothing; successful buyback charges once
  and produces a vacant rental.
- Selling an occupied unit removes both sides of the lease and records the
  displacement; sold units are excluded from leasing and occupancy metrics.
- Space does not advance time while paused or while a blocking event is open.
- Tutorial acquisition respects the inherited tenant and reaches inactive,
  completed state without per-frame hint spam.

## Milestone 2 — One contract for every shipped feature

Give the player reachable routes to Building, Tenants/Applications, Finances,
City/Portfolio, Inbox, and Tasks. Mail can be opened and marked read; dialogues
can be answered; missions are visible with progress and rewards; City and
Market can be entered and exited without undocumented keys.

Consolidate upgrade definitions and costs around `assets/upgrades.json`.
Remove the parallel cheap design-upgrade path. Every offered upgrade must have
a tested effect: laundry changes building appeal, lighting and kitchens change
unit quality, soundproofing changes effective noise, and every staff role has a
documented monthly benefit or is removed from the offered roster. Marketing,
open houses, utilities, and insurance must be controllable if they remain in
the simulation.

Rent changes must use one action that updates the apartment, records the active
building's gentrification history, and feeds tenant negotiation. Event text
must describe the effect actually applied. Mission building targets must be
honored instead of silently evaluating whichever building is selected.

Acceptance checks:

- Every `UiAction` variant is produced by a visible control or removed.
- Every visible control either changes authoritative state or reports why it
  cannot; no offered upgrade is cosmetic by accident.
- Inbox unread counts can reach zero, dialogues cannot accumulate without a
  response route, and mission progress is visible before a reward fires.
- Rent changes update the correct building history and can trigger the stated
  gentrification/negotiation consequences.
- Marketing and operating-policy costs and benefits are shown before choice.

## Milestone 3 — Representative balance and progression

Rebuild the harness around the same new-game constructor, difficulty modifiers,
upgrade actions, vetting costs, mission rewards, and terminal behavior used by
the game. Run every campaign template across many seeds and add strategies for
careful investment, minimal upkeep, outright neglect, aggressive rents, condo
sales, and portfolio expansion.

Report cash, score, net income, occupancy, happiness, condition, applications,
tenant departures, investment payback, and terminal outcomes. Stop a run when
the game ends. Tune only after this representative report exists.

Balance targets:

- The Investor strategy beats Greedy by at least 10% in mean final score on
  each tier and is not behind in both score and cash.
- Normal building investments repay their cost or create equivalent score and
  retention value within 12–18 months of a 36-month campaign.
- Easy supports recovery from mistakes but does not routinely end near twenty
  times starting cash; Medium and Hard create materially different failure and
  recovery profiles.
- Slumlord play may occasionally survive, but has clearly worse score,
  condition, retention, and bankruptcy risk.
- Condo sales are situational liquidity decisions rather than an immediate
  score/cash dominant strategy.
- Vetting and rejection trade vacancy time against expected rent loss and
  damage; neither accepting everyone nor rejecting every risk is automatic.

## Milestone 4 — Responsive management UI

Replace the empty 60/40 canvas split with a responsive management shell:

- a compact top bar for building, month, funds, occupancy, and monthly net;
- primary navigation for Building, Tenants, Finances, City, Inbox, and Tasks,
  with pending-decision badges;
- a scalable building cutaway as the main workspace;
- an always-useful inspector showing building summary when nothing is selected
  and contextual unit/tenant actions when it is;
- a collapsible activity drawer instead of a permanently empty 100 px footer;
- target-anchored tutorial coaching instead of a toast covering activity;
- one theme and component language across the title menu and gameplay.

Use at least 16 px body text, 13 px captions, 40–44 px primary controls, clear
focus/hover/disabled states, labelled icons, and measured text truncation.
Panels become drawers or stacked views at narrow widths rather than shrinking
below readable sizes.

Acceptance checks:

- Verified captures at 1280×720, 1024×768, and 800×600 have no overlap,
  clipping, unreachable controls, or unexplained empty regions.
- The six-unit and ten-unit campaign buildings remain readable and clickable at
  every verification size.
- A fresh player can find applications, repair a unit, inspect finances, read
  mail, answer an event, and end a month without relying on hidden shortcuts.
- Keyboard/mouse interaction respects modal focus and Escape consistently.

## Milestone 5 — Completion audit

Run a requirement-by-requirement audit against this document. Add tests for any
acceptance check not already proven, remove obsolete TODO entries and dead
compatibility paths, and refresh verified menu/gameplay captures plus the root
catalog thumbnail if the title presentation changes.

Required final validation:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test balance_report -- --ignored --nocapture
.\publish.ps1
```

The final handoff must report the current balance table, capture sizes checked,
test totals, and publisher result. Passing compilation alone is not evidence
that this plan is complete.
