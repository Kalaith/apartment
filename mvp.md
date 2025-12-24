# MVP Scope

## Goal
Make repairing apartments and choosing tenants feel meaningful within **30–60 minutes** of play.

> *If the player finishes thinking “I get this, and I want more buildings”, the MVP worked.*

## World Scope
- 1 apartment building
- 6–8 units total
- 1 shared hallway
- 1 shared utility system
- No city map, neighborhoods, or expansion yet.

## Core Systems (Only These)
### 1. Apartments
Each unit has exactly **5 properties**:
- **Condition** (0–100)
- **Design Type** (Bare, Practical, Cozy)
- **Size** (Small / Medium)
- **Noise Level** (Low / High)
- **Rent Price**

*No furniture placement. No free‑form layouts.*

### 2. Repairs & Upgrades
Only three actions per apartment:
- **Repair** – raises condition, costs money, condition decays slowly over time.
- **Upgrade Design** – Bare → Practical → Cozy (one‑way unless demolished later, which is not in MVP).
- **Soundproofing** – toggle upgrade, reduces noise complaints.

**Shared space:** Hallway Repair (affects whole building appeal).

### 3. Tenants
Exactly **3 tenant archetypes**:

| Tenant | Cares About | Hates |
|--------|-------------|-------|
| Student | Low rent | Bad condition |
| Professional | Condition, quiet | Noise |
| Artist | Cozy design | Sterile spaces |

Each tenant has:
- **Happiness** (0–100)
- **Rent tolerance**
- **Noise tolerance**

*No backstories. No dialogue trees.*

### 4. Tenant Flow
- Start with **1 tenant** already living there.
- Vacant units generate applications.
- Player chooses who to accept.
- Wrong fit leads to **complaints** and **early move‑out**.
- Tenants leave if happiness hits **0**.

### 5. Money
Only two money flows:
- **Monthly rent**
- **Repair / upgrade costs**

*No loans, taxes, or selling units.* Money pressure exists but won’t kill the run instantly.

### 6. Time
Discrete turns or slow real‑time ticks.
Each tick:
- Rent collected
- Condition decays slightly
- Tenant happiness updates

*One in‑game month per tick is fine.*

## Win / End State
- No infinite mode yet.
- Session ends when **all units are filled** and **average happiness** is above a threshold, **or** you go **bankrupt**.
- Show a simple summary: total income, tenant turnover, building reputation.

## UI Needs (Minimal)
- Building overview screen
- Apartment detail panel
- Tenant list
- Simple notifications:
  - “Noise complaint”
  - “Tenant moved out”
  - “Rent paid”

*No character portraits required.*

## What Is Explicitly **NOT** in MVP
- Multiple buildings
- Buying/selling units
- Owner politics
- Events, random disasters, neighborhood stats
- Procedural stories, modding
- Graphics beyond icons and bars

These can be added later as future features.

## Why This MVP Works
- Shows the **design → tenant → consequence** loop.
- Creates small but real decisions.
- Finite scope, easy to rebalance.
- Playable in one evening.
- If this version feels dull, the full game won’t save it. If it works, everything else becomes additive.