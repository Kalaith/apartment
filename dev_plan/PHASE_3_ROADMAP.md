# Phase 3: Scale & Complexity Roadmap

## Overview
With Phase 2's polish, content expansion, and persistence systems in place, Phase 3 transitions the game from a single-building experience to a **multi-property management simulation** with deeper strategic choices and consequence systems.

**Theme**: From landlord to property manager.

## Implementation Status: ✅ CORE COMPLETE

| Feature | Status | Files |
|---------|--------|-------|
| Neighborhood System | ✅ Complete | `src/city/neighborhood.rs` |
| City & Multi-Building | ✅ Complete | `src/city/city.rs` |
| Property Market | ✅ Complete | `src/city/market.rs` |
| Tenant Relationships | ✅ Complete | `src/consequences/relationships.rs` |
| Regulations & Compliance | ✅ Complete | `src/consequences/regulations.rs` |
| Gentrification Tracking | ✅ Complete | `src/consequences/gentrification.rs` |
| Tenant Stories | ✅ Complete | `src/narrative/stories.rs` |
| Narrative Events | ✅ Complete | `src/narrative/events.rs` |
| Mail System | ✅ Complete | `src/narrative/mail.rs` |
| City Map UI | ✅ Complete | `src/ui/city_view.rs` |
| Ownership Models | 📋 Planned | - |
| Advanced Economy | 📋 Planned | - |

---

## Concurrent Tracks

### Track D: Multi-Building Systems (The "Empire")
**Goal**: Allow players to acquire and manage multiple properties across different neighborhoods.

#### D1_NEIGHBORHOOD_SYSTEM
- **Neighborhood Types**: Each with distinct characteristics
  - Downtown (High rent, high turnover, noise complaints)
  - Suburbs (Families, stability, lower margins)
  - Industrial (Students, artists, gentrification pressure)
  - Historic (Elderly, regulations, preservation requirements)
- **Neighborhood Stats**: Crime level, transit access, walkability, schools, services
- **Dynamic Changes**: Neighborhoods evolve based on player actions and city-wide events
- **Reputation Per Area**: Each neighborhood has separate building reputation

#### D2_BUILDING_ACQUISITION
- **Property Market**: Available buildings appear on a city map
- **Building Conditions**: 
  - Condemned properties (cheap, massive repairs needed)
  - Occupied buildings (tenants already in place, can't evict easily)
  - Conversions (warehouses, old hotels, commercial spaces)
- **Financing Options**:
  - Bank loans (interest rates, credit rating)
  - Investors (take a % of profits, can be bought out)
  - Mortgages (must maintain payments or face foreclosure)

#### D3_PORTFOLIO_MANAGEMENT
- **Dashboard View**: See all buildings at a glance
- **Building Comparison**: Performance metrics across portfolio
- **Cross-Building Actions**: Transfer funds, share resources
- **Specialization Bonuses**: Benefits for focusing on one neighborhood or building type

---

### Track E: Consequence Systems (The "Weight")
**Goal**: Make player decisions feel meaningful with lasting ramifications.

#### E1_TENANT_RELATIONSHIPS
- **Tenant History**: Long-term tenants provide stability but have more influence
- **Community Dynamics**: 
  - Tenants form relationships (positive or negative)
  - "Problem tenants" vs "Model tenants"
  - Tenant councils (can organize, demand improvements)
- **Eviction System**:
  - Legal process (time and money)
  - Reputation hit
  - Potential harassment lawsuits

#### E2_GENTRIFICATION_MECHANICS
- **Displacement Tracking**: Original tenants priced out when neighborhood improves
- **Cultural Shifts**: Artist communities attract professionals, changing area character
- **Moral Choices**: 
  - Renovate and raise rents (profit vs displacement)
  - Grandfather in long-term tenants (loyalty vs revenue)
  - Sell to developers (immediate payout vs community loss)
- **No Morality Meter**: Just consequences shown through tenant stories and neighborhood changes

#### E3_REGULATION_AND_COMPLIANCE
- **Building Codes**: Evolving safety standards require costly updates
- **Rent Control**: Some neighborhoods have rent caps
- **Inspections**: Random or complaint-triggered
- **Fines and Violations**: Escalating penalties for non-compliance
- **Legal Troubles**: Lawsuits from tenant complaints or injuries

---

### Track F: Strategic Depth (The "Game")
**Goal**: Create meaningful player choices with multiple viable strategies.

#### F1_OWNERSHIP_MODELS
- **Rent vs Sell Units**: (Expands MVP concept)
  - Condo conversion mechanics
  - Owner-occupied units block upgrades
  - Condo board voting system
  - Mixed ownership management
- **Co-op Buildings**: Tenants collectively own, different management style
- **Section 8 / Social Housing**: Guaranteed income, strict requirements, reputation benefits

#### F2_BUILDING_SPECIALIZATION
- **Design Identity**: Maintain coherent building theme for bonuses
  - Luxury apartments (high margins, demanding tenants)
  - Artist lofts (cheap, creative community, grants available)
  - Family housing (stable, long leases, needs space)
  - Student dormitory (seasonal, high turnover, bulk contracts)
- **Amenity Systems**:
  - Shared spaces (gym, rooftop, laundry)
  - Services (doorman, maintenance staff, security)
  - Community programs (tenant events, board, newsletters)

#### F3_ADVANCED_ECONOMY
- **Operating Costs**: Expanding beyond simple repairs
  - Staff wages (maintenance, security, management)
  - Utilities (who pays what)
  - Insurance (rates vary by building condition and neighborhood)
  - Property taxes (can increase dramatically)
- **Revenue Streams**:
  - Commercial ground floor units (shops, cafes)
  - Parking space rentals
  - Laundry room income
  - Billboard/phone tower leases
- **Economic Cycles**: Market booms and recessions affect rent demand

---

### Track G: Narrative Depth (The "Soul")
**Goal**: Add light narrative elements without becoming story-heavy.

#### G1_PROCEDURAL_TENANT_STORIES
- **Background Generation**: Tenants have simple procedural backstories
  - Job type, family status, hobbies
  - Reasons for moving (career change, breakup, new school)
- **Life Events**: Tenants experience changes
  - Job loss (can't pay rent)
  - New baby (need bigger space)
  - Relationship changes (couples split, roommates needed)
- **Requests and Favors**: Personal communication beyond complaints
  - "Can I have a cat?" (breaks no-pets rule but tenant loves you)
  - "My mother needs to move in temporarily"
  - "I'm starting a small business from home"

#### G2_EVENT_NARRATIVES
- **Contextual Events**: Not just random mechanical effects
  - "Pipe burst on Floor 3" → specific tenant consequences
  - "Power outage during storm" → elderly tenant hospitalized
  - "Developer wants to buy building" → choice with consequences
- **Newspaper/Mail System**: 
  - Articles about neighborhood changes
  - Letters from tenants
  - City notices and regulations
  - Bills and financial updates

#### G3_BUILDING_IDENTITY
- **Building Names**: Player can name buildings (and they gain reputations)
- **History Tracking**: Records of significant events
  - First renovation
  - Longest-term tenant
  - Memorable disasters or triumphs
- **End-Game Summary**: Expanded results screen showing building legacies

---

## Task Files Structure

New task files created for Phase 3:

| File | Description | Track |
|------|-------------|-------|
| `dev_plan/10_NEIGHBORHOOD_SYSTEM.md` | City map, area stats, dynamics | D |
| `dev_plan/11_MULTI_BUILDING.md` | Acquisition, financing, portfolio view | D |
| `dev_plan/12_CONSEQUENCE_SYSTEMS.md` | Relationships, gentrification, compliance | E |
| `dev_plan/13_OWNERSHIP_MODELS.md` | Rent/sell, co-ops, specialization | F |
| `dev_plan/14_ADVANCED_ECONOMY.md` | Operating costs, revenue streams, cycles | F |
| `dev_plan/15_NARRATIVE_ELEMENTS.md` | Procedural stories, events, identity | G |

---

## Implementation Priority

### High Priority (Core Phase 3)
1. **D1_NEIGHBORHOOD_SYSTEM** - Foundation for everything else
2. **D2_BUILDING_ACQUISITION** - Core expansion mechanic
3. **F3_ADVANCED_ECONOMY** - Needed for complexity scaling
4. **E3_REGULATION_AND_COMPLIANCE** - Adds meaningful constraints

### Medium Priority (Depth)
5. **E1_TENANT_RELATIONSHIPS** - Emotional engagement
6. **F2_BUILDING_SPECIALIZATION** - Strategic variety
7. **G2_EVENT_NARRATIVES** - Contextual storytelling

### Lower Priority (Polish)
8. **D3_PORTFOLIO_MANAGEMENT** - Nice UI improvements
9. **G1_PROCEDURAL_TENANT_STORIES** - Flavor and immersion
10. **F1_OWNERSHIP_MODELS** - Advanced strategy options

---

## Technical Considerations

### Data Structures
- **City/Neighborhood Layer**: New top-level structure containing multiple buildings
- **Economic Model**: Needs refactoring to handle portfolio-level calculations
- **Save System**: Expand to handle multiple buildings and complex relationships
- **Event System**: Needs to target specific buildings or city-wide effects

### Performance
- Multiple buildings tick simultaneously - optimize simulation loops
- UI must handle switching between building views efficiently
- Consider LOD (level of detail) for buildings not currently viewed

### UI/UX Challenges
- **City Map Interface**: New top-level view showing all properties
- **Building Switcher**: Quick navigation between properties
- **Comparative Analytics**: Charts and graphs for portfolio performance
- **Decision Modals**: Complex choices need clear presentation

---

## Suggested Next Steps

**Start with Track D (Multi-Building) as the foundation:**
1. Implement neighborhood system and city map
2. Add building acquisition mechanics
3. Expand economy to support portfolio operations

**Then add Track E (Consequences) for depth:**
4. Implement tenant relationship system
5. Add gentrification mechanics
6. Build out regulation and compliance

**Finally Track F (Strategy) and Track G (Narrative) for polish and replayability.**

---

## Success Metrics

Phase 3 is complete when:
- [ ] Player can own and manage 3+ buildings simultaneously
- [ ] Each neighborhood feels distinct and affects gameplay differently
- [ ] Player faces meaningful strategic dilemmas (profit vs stability)
- [ ] Building reputation and neighborhood changes create emergent stories
- [ ] Economic systems create natural pressure and decision points
- [ ] Session length extends to 2-3 hours with multiple playstyles viable

**If Phase 3 succeeds**, the game transitions from a focused simulation to a strategic sandbox with emotional resonance.
