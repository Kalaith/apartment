Narrative & City Soul Implementation Plan
Overview
This plan brings soul to the game by connecting existing systems (City, Neighborhoods, Relationships, Events) into a cohesive gameplay experience. The focus is on making mid-to-late game engaging when the player has achieved full occupancy.

User Review Required
IMPORTANT

This is a substantial feature set spanning multiple systems. Implementation will be phased to allow testing between phases.

Key Decisions Needed:

Should tenant stories/events be shown as pop-up dialogs, side panel notifications, or mail items?
How visible should relationship changes be? (subtle icons vs explicit notifications)
Priority order for phases - which features are most important to you?
Problem Statement
At Month 6 with full occupancy, the player has nothing meaningful to do except click "End Month". The game needs:

Goals beyond occupancy - relationship building, reputation, expansion
Meaningful choices - respond to tenant requests, navigate conflicts
City exploration - other buildings to acquire, neighborhood dynamics
Emergent stories - tenant relationships create organic drama
Proposed Changes
Phase 1: Tenant Relationships & Hints (Core Experience)
Make existing relationships visible and add context-aware hints.

[MODIFY] 
config.json
Add new config sections for:

Relationship notification thresholds
Context-aware hint messages based on game state
Tenant request probabilities
"hints": {
  "full_occupancy": "Focus on tenant happiness to maximize your 3-year score!",
  "low_condition": "Some apartments need repairs - check the building panel",
  "relationship_formed": "A new friendship formed between {tenant_a} and {tenant_b}!",
  "conflict_brewing": "Tension is rising between {tenant_a} and {tenant_b}..."
}
[NEW] 
hints.json
Data-driven contextual hints that display based on game state:

{
  "conditions": [
    {
      "id": "full_occupancy",
      "trigger": { "vacancy_count": 0 },
      "messages": [
        "All units occupied! Focus on keeping tenants happy.",
        "Your building is full. Watch for relationship opportunities!",
        "Consider upgrading apartments to boost tenant happiness."
      ]
    },
    {
      "id": "relationship_conflict",
      "trigger": { "hostile_relationships": ">0" },
      "messages": [
        "Tenant conflict detected. Intervene or let it resolve naturally.",
        "Unhappy neighbors can drive tenants away..."
      ]
    }
  ]
}
[MODIFY] 
apartment_panel.rs
Display relationship icons on tenant portraits (heart for friendly, lightning for hostile)
Show context-aware hints in the Applications panel when relevant
[MODIFY] 
relationships.rs
Add RelationshipChange enum for tracking new/changed relationships
Return relationship changes from 
tick()
 for notification purposes
Phase 2: Active Tenant Events (Engagement)
Give tenants "voice" through requests and story beats.

[NEW] 
tenant_events.json
Data-driven tenant events by archetype:

{
  "events": [
    {
      "id": "student_party_request",
      "archetype": "Student",
      "trigger": { "happiness": ">60", "tenure_months": ">2" },
      "title": "{tenant.name} wants to host a study group",
      "description": "Your student tenant asks permission for weekly study sessions.",
      "choices": [
        { "label": "Allow it", "effects": ["+5 happiness", "+noise risk"] },
        { "label": "Decline", "effects": ["-3 happiness"] }
      ]
    },
    {
      "id": "elderly_accessibility",
      "archetype": "Elderly",
      "trigger": { "tenure_months": ">6" },
      "title": "{tenant.name} requests accessibility modifications",
      "description": "They'd like a grab bar installed in the bathroom.",
      "choices": [
        { "label": "Install ($50)", "effects": ["-50 money", "+15 happiness", "+apartment condition"] },
        { "label": "Decline", "effects": ["-5 happiness", "-landlord opinion"] }
      ]
    }
  ]
}
[MODIFY] 
stories.rs
Load events from tenant_events.json
Add TenantEventTrigger struct to evaluate conditions
Generate active events during monthly tick
[NEW] 
event_panel.rs
New UI component for displaying tenant events with choices:

Modal overlay when event requires response
Choice buttons with effect previews
Timer for urgent events
Phase 3: City View Integration (Expansion)
Connect the existing city view to meaningful gameplay.

[MODIFY] 
gameplay.rs
Add keyboard shortcut ('C') to toggle city view
Integrate city view into main game loop, not just as separate screen
[NEW] 
neighborhoods.json
Externalize neighborhood definitions:

{
  "neighborhoods": [
    {
      "id": 0,
      "name": "Central District",
      "type": "Downtown",
      "description": "High-rise living with premium rents but demanding tenants.",
      "base_stats": {
        "crime": 40, "transit": 95, "walkability": 90,
        "schools": 50, "services": 95, "rent_demand": 1.2
      },
      "tenant_preferences": ["Professional", "Student"],
      "events": ["new_business", "transit_disruption", "festival"]
    }
  ]
}
[MODIFY] 
neighborhood.rs
Load neighborhood data from JSON
Add neighborhood-specific events that affect all buildings in area
[MODIFY] 
city_view.rs
Add "Enter Building" button to manage selected property
Show neighborhood events/news in city view
Display reputation progress toward unlocking new purchase opportunities
Phase 4: Relationship Stories (Soul)
Deep narrative connections between tenants.

[NEW] 
relationship_events.json
Story beats triggered by relationship milestones:

{
  "events": [
    {
      "id": "friendship_deepens",
      "trigger": { "relationship": "Friendly", "strength": ">50" },
      "title": "{tenant_a.name} and {tenant_b.name} have become close friends",
      "description": "They've started having dinner together regularly.",
      "effects": ["+cohesion bonus"]
    },
    {
      "id": "conflict_escalates",
      "trigger": { "relationship": "Hostile", "strength": ">30" },
      "title": "The conflict between {tenant_a.name} and {tenant_b.name} is getting worse",
      "choices": [
        { "label": "Mediate (-$25)", "effects": ["-25 money", "-20 hostility"] },
        { "label": "Let them sort it out", "effects": ["50% chance: resolve, 50%: escalate"] }
      ]
    }
  ]
}
[MODIFY] 
relationships.rs
Track relationship history/strength over time
Generate narrative events at relationship milestones
Add player intervention options for conflicts
Phase 5: The 3-Year Score (Endgame)
Make the ending meaningful with career summary.

[NEW] 
career_summary.rs
End-game screen showing:

Score breakdown (happiness, income, relationships, reputation)
Memorable moments (first tenant, biggest conflict resolved, longest-staying tenant)
Achievements unlocked
"Play Again" with different starting conditions
[NEW] 
achievements.json
Data-driven achievements:

{
  "achievements": [
    { "id": "full_house", "name": "Full House", "desc": "Fill every apartment", "trigger": "vacancy_count == 0" },
    { "id": "matchmaker", "name": "Matchmaker", "desc": "Cultivate 5 friendly relationships", "trigger": "friendly_count >= 5" },
    { "id": "peacekeeper", "name": "Peacekeeper", "desc": "Resolve 3 tenant conflicts", "trigger": "conflicts_resolved >= 3" },
    { "id": "old_timer", "name": "Old Timer", "desc": "Keep a tenant for 24 months", "trigger": "max_tenure >= 24" }
  ]
}
File Summary
Phase	New Files	Modified Files
1	hints.json	
config.json
, apartment_panel.rs, 
relationships.rs
2	tenant_events.json, event_panel.rs	stories.rs
3	neighborhoods.json	
neighborhood.rs
, 
city_view.rs
, 
gameplay.rs
4	relationship_events.json	
relationships.rs
5	career_summary.rs, achievements.json	
win_condition.rs
Verification Plan
Phase-by-Phase Testing
Since the game is a Rust/Macroquad project, verification will be:

Compilation Check (after each phase)

User runs: cargo check
Expected: No errors or warnings
Manual Playtesting (each phase)

Start new game
Play to Month 6+ with full occupancy
Verify new features are visible and functional:
Phase 1: Relationship icons visible, hints appear
Phase 2: Tenant events trigger, choices work
Phase 3: City view accessible via 'C' key
Phase 4: Relationship stories appear at milestones
Phase 5: Career summary shows at Month 36
Data Validation

Verify JSON files load without errors
Check console for any "Failed to parse" messages
Existing Tests
The codebase has unit tests in various modules (cargo test), but the narrative features are primarily UI-driven and require manual testing.

Implementation Order
I recommend implementing in this order based on impact:

Phase 1 (Quick win) - Visible relationships + hints give immediate feedback
Phase 2 (Engagement) - Tenant events create choices
Phase 5 (Closure) - Career summary gives meaning to 3-year goal
Phase 3 (Expansion) - City view for advanced players
Phase 4 (Polish) - Deep relationship stories
Questions for You
Which phases are highest priority for you?
Do you want tenant events as pop-up modals or integrated into the side panel?
Should I start with Phase 1 (relationship visibility + hints) after approval?