# Phase 4C: Interaction & Relationships

## Goal
Make tenants feel like people with needs and opinions, creating emotional stakes in management decisions.

## Features

### 1. The Dialogue System (Lite)
Tenants now "knock on your door" for more than just mail.
- **Face-to-Face Requests**: High-priority issues (e.g., "My heater is out and it's winter") trigger a UI pop-up.
- **Conflict Mediation**: Tenant A complains about Tenant B's music. You choose:
  - Warn Tenant B (Happiness loss for B, gain for A).
  - Install soundproofing (Costly, both happy).
  - Ignore (Happiness loss for A, potential move-out).

### 2. Rent Negotiation
Instead of a global slider, rent changes are a conversation.
- **The Pitch**: "The market is up" vs "I fixed the roof."
- **Tenant Pushback**: Long-term tenants point to their loyalty to keep rent low.

### 3. Community Value
- **Social Cohesion**: Happy tenants who "fit" together (e.g., all Artists) generate a bonus that reduces maintenance needs or boosts reputation.
- **Tenant Council**: If too many tenants are unhappy, they form a council to demand collective changes.

## Technical Tasks
- [ ] Create a `Dialogue` state machine in `src/narrative`.
- [ ] Add `SocialTension` tracking between apartments.
- [ ] Implement `Negotiation` mini-game logic.
- [ ] Expand `TenantRelationship` with "Landlord Opinion" stat.
