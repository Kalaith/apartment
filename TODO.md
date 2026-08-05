# TODO — Second Story

The implementation order and acceptance checks for the August 2026 review are
tracked in [`docs/REMEDIATION_PLAN.md`](docs/REMEDIATION_PLAN.md). Keep this file
as the short problem list; use the plan as the completion authority.

## Balance

- Investing has to beat neglect. The August 2026 run had Greedy finish at
  ~$97.2k and Investor at ~$97.1k across 60 seeds, despite the harness giving
  Investor unrealistically cheap upgrades. Rebuild the harness before tuning.
- Money is too easy to earn overall, and there is no situation that makes selling an apartment attractive.
- Give rejecting an applicant real teeth — a risky tenant carries a rent premium and can damage property, but the downside still rarely bites hard enough to make vetting a decision.

## UI

- The interface is dated and elements overlap. This is the single most important milestone from the last playtest and nothing since has addressed it.
- Separate building economics and tenant simulation from the UI panels so the city, hallway, tenant and ownership views render derived state only.

## Simulation integrity

- Validate tenant applications and apartment assignments so duplicate tenants, impossible vacancies and stale lease records cannot occur.
- Extend the harness with scenario fixtures for each property tier — it currently only exercises the starting building, so balancing repair costs, upgrades and reputation on the later properties is still manual.
