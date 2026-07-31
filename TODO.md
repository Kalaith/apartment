# TODO — Second Story

## Balance

- Investing has to beat neglect. The last full harness run had the Greedy strategy (accept everyone, minimal upkeep) finish 36 months at ~$95.5k against the Investor's ~$96.1k, so repairs, upgrades and staff bought almost nothing. Re-run `cargo test balance_report -- --ignored --nocapture` after any economy change and check the two strategies have separated.
- Money is too easy to earn overall, and there is no situation that makes selling an apartment attractive.
- Give rejecting an applicant real teeth — a risky tenant carries a rent premium and can damage property, but the downside still rarely bites hard enough to make vetting a decision.

## UI

- The interface is dated and elements overlap. This is the single most important milestone from the last playtest and nothing since has addressed it.
- Separate building economics and tenant simulation from the UI panels so the city, hallway, tenant and ownership views render derived state only.

## Simulation integrity

- Validate tenant applications and apartment assignments so duplicate tenants, impossible vacancies and stale lease records cannot occur.
- Extend the harness with scenario fixtures for each property tier — it currently only exercises the starting building, so balancing repair costs, upgrades and reputation on the later properties is still manual.
