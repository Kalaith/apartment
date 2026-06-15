# Apartment Manager

Apartment Manager is a cozy building-management game about caring for a neglected apartment block and the people who live there.

You are not a property tycoon. You are the custodian of a tired building with flickering lights, worn carpets, difficult choices, and tenants who need the place to work.

## Gameplay

- Inspect apartments and decide what to repair or upgrade.
- Review tenant applications and handle tenant needs.
- Adjust rent without destroying happiness or occupancy.
- Improve shared facilities and building condition.
- Manage money, time, requests, and long-term reputation.

## Goal

Survive 36 months while keeping the building financially stable and livable. Completing one building unlocks a harder property with new pressure.

## Controls

- Mouse: select apartments and use buttons.
- Space: end turn or advance time.
- Esc: pause menu.

## Current Scope

Playable building progression with multiple properties, tenant systems, repairs, upgrades, missions, and month-by-month management.
# Practical Future Improvements

- Add month-step regression tests for rent changes, occupancy, tenant happiness, maintenance debt, and property unlock progression.
- Separate building economics and tenant simulation from UI panels so city, hallway, tenant, and ownership views render derived state only.
- Add validation for tenant applications and apartment assignments to prevent duplicate tenants, impossible vacancies, or stale lease records.
- Create scenario fixtures for each property tier to make balancing repair costs, upgrades, and reputation less manual.

