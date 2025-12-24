# Apartment Game Design Document

## Core Fantasy
You are not a tycoon. You are a custodian of a failing place. You are tasked with fixing a half‑dead apartment block with flickering lights, stained carpets, and one stubborn tenant who refuses to leave. You fix one thing at a time. Slowly, the building remembers what it was meant to be.

People don’t just move in because numbers went up. They move in because the place starts to mean something.

## Core Loop
### Inspect
Walk the building floor by floor. Units, hallways, stairwells, utilities, shared spaces. Everything has condition, reputation, and hidden problems.

### Repair & Improve
- **Structural** – plumbing, wiring, insulation, fire safety
- **Cosmetic** – paint, flooring, lighting, fixtures
- **Quality‑of‑life** – soundproofing, storage, ventilation
- **Shared upgrades** – laundry, bike storage, rooftop garden, lobby

## Attract Tenants
Tenants arrive based on:
- Unit quality
- Building reputation
- Neighborhood state
- Rent price
- Design intent

## Balance Money
- **Rent** brings stability
- **Sales** bring capital and consequences
- **Loans** exist, but debt has teeth

## Expand or Specialize
Buy nearby buildings, or double down on making one block exceptional.

## Apartment Design as Gameplay
Each apartment has design slots, not just stats. Examples include:
- Flooring type
- Wall finish
- Lighting quality
- Kitchen level
- Bathroom level
- Noise insulation
- View / balcony / windows

These combine into design profiles such as Minimalist, Cozy, Luxury, Practical, Artistic, Communal, Barebones.

Design affects:
- Who wants to live there
- How long they stay
- How much they tolerate issues

## Demographics (Not Just “Rich / Poor”)
Tenants have values, not just income. Example groups:
- Students
- Artists
- Young professionals
- Families
- Retirees
- Night workers
- Recluses
- Social butterflies

Each demographic cares about different things (noise tolerance, commute access, space vs location, community vs privacy, maintenance responsiveness, price stability).

## Rent vs Sell
### Renting
- Monthly income
- Ongoing maintenance expectations
- Tenant complaints, requests, and reviews
- Stable but fragile

### Selling Units
- Large upfront cash
- Reduces future rental income
- Owners can block upgrades and influence building votes

## Building Systems (Slow‑Burn Depth)
- **Wear & Tear** – improvements degrade over time
- **Hidden Faults** – old wiring, mold, pests
- **Emergencies** – floods, power failures, inspections
- **Regulations** – safety standards evolve
- **Neighbors** – adjacent buildings affect appeal
- **Neglect** – ignoring one system cascades failures

## Multiple Buildings
Later you can:
- Buy adjacent blocks
- Take over condemned properties
- Convert warehouses
- Manage buildings in different neighborhoods, each with its own crime level, transit, services, and cultural drift.

## Player Identity Choices
Over time the game quietly asks:
- Maximize profit or stability?
- Preserve communities or flip them?
- Specialize or diversify?
- Respond fast or let things slide?

There is no morality meter. Only consequences.

## Rust as a Strength
Rust fits this beautifully:
- Deterministic simulation
- Strong data‑driven systems
- Emergent behavior from simple rules
- Clean separation between systems (tenants, buildings, economy)

## Optional Future Directions
- Procedural tenants with short backstories
- Light narrative events via mail, notes, emails
- City‑wide economic shifts
- Co‑op mode managing adjacent buildings
- Modding via data definitions

## MVP Scope
- **World**: One apartment building, 6–8 units, 1 shared hallway, 1 shared utility system. No city map, neighborhoods, or expansion.
- **Core Systems**:
  1. **Apartments** – each unit has five properties: Condition (0‑100), Design Type (Bare, Practical, Cozy), Size (Small / Medium), Noise Level (Low / High), Rent Price. No furniture placement.
  2. **Repairs & Upgrades** – three actions per apartment: Repair (raises condition, costs money), Upgrade Design (Bare → Practical → Cozy, one‑way), Soundproofing (toggles upgrade, reduces noise complaints).
  3. **Shared Space** – Hallway Repair affects whole building appeal.
  4. **Tenants** – three archetypes (Student, Professional, Artist) with cares/hates. Each tenant has Happiness (0‑100), Rent tolerance, Noise tolerance. No backstories or dialogue.
  5. **Tenant Flow** – start with one tenant, vacant units generate applications, player chooses who to accept. Mismatched fits cause complaints and early move‑out.
  6. **Money** – monthly rent income, repair/upgrade costs. No loans, taxes, or selling.
  7. **Time** – discrete turns or slow real‑time ticks. Each tick: rent collected, condition decays slightly.
- **Goal**: Make repairing apartments and choosing tenants feel meaningful within 30–60 minutes of play. If the player finishes thinking “I get this, and I want more buildings”, the MVP worked.
