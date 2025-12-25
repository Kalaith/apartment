# Task 15: Narrative Elements

## Priority: 🟢 MEDIUM (Polish)
## Dependencies: Task 03 (Tenant), Task 12 (Consequences)
## Estimated Effort: 4-5 hours

## Objective
Add light narrative elements to make the game feel alive without becoming story-heavy. Procedural tenant stories, contextual events, and a mailbox system provide personality and immersion.

## Implementation Status: ✅ COMPLETE

### Files Created
- `src/narrative/mod.rs` - Module exports
- `src/narrative/stories.rs` - Tenant backstory generator
- `src/narrative/events.rs` - Narrative event system
- `src/narrative/mail.rs` - Mailbox and correspondence

---

## Features Implemented

### 1. Tenant Stories (`stories.rs`)

**TenantStory struct:**
- Job title
- Hometown
- Move reason
- Hobbies
- Personality traits
- Family status (partner, children)
- Story events history
- Pending requests

**BackgroundGenerator:**
- Archetype-appropriate job titles
- Randomized hometowns
- Move reasons matching archetype
- Hobby pools per archetype
- Personality traits

**Life Events (`LifeChangeType`):**
- New job (better/worse)
- Job loss
- Got married/partnered
- Separation
- New baby
- Child left home
- Health issues
- Retirement
- Started/finished school

**Tenant Requests (`TenantRequest`):**
- Pet requests (cat, dog, hamster)
- Temporary guests (family, caregivers)
- Home business (consulting, art studio)
- Modifications (lighting, paint)
- Sublease

**Integration:**
```rust
let story = TenantStory::generate(tenant.id, &tenant.archetype);
println!("{}", story.summary());
// "University Student from the suburbs. Needed to be closer to campus. Enjoys gaming, reading."
```

---

### 2. Narrative Events (`events.rs`)

**NarrativeEventType:**
- Neighborhood news
- City-wide events
- Tenant story beats
- Building milestones
- Character encounters
- External offers (developers, investors)
- Seasonal events

**NarrativeEvent:**
- Headline and description
- Optional choices
- Default effect if no choice
- Read/unread tracking
- Response deadlines

**NarrativeChoice:**
- Label and description
- Effect on game
- Reputation change

**NarrativeEffect:**
- None (flavor only)
- Money gained/lost
- Neighborhood reputation
- Building happiness
- Tenant happiness
- Economy changes
- Rent demand
- Trigger inspection
- Property value change
- Multiple combined effects

**Event Generation:**
- Neighborhood news (new businesses, crime, festivals)
- City events (market changes, transit, taxes)
- Seasonal events (spring cleaning, heat waves, back to school)
- Developer offers to buy buildings

---

### 3. Mail System (`mail.rs`)

**MailType:**
- Tenant letters
- City notices
- Financial statements
- Advertisements
- News clippings
- Personal correspondence
- Official documents

**MailItem:**
- Sender, subject, body
- Month received
- Read/unread status
- Optional action
- Requires attention flag

**MailAction:**
- Pay fine (with deadline)
- Respond to tenant
- Schedule inspection
- Accept/reject offer
- Acknowledge

**Mailbox:**
- `receive()` - Add new mail
- `unread_count()` - Quick count
- `needs_attention()` - Urgent items
- `unread_mail()` - Sorted by priority
- `cleanup()` - Remove old read mail

**Automatic Mail:**
- Monthly financial statements
- Random tenant letters based on happiness/conditions
- News clippings

---

## Sample Mail Generation

```rust
// Tenant letter based on conditions
if apartment.condition < 40 {
    // Maintenance request letter
} else if tenant.happiness > 80 {
    // Thank you note
} else if tenant.happiness < 40 {
    // Concerns letter
} else {
    // Friendly check-in
}
```

---

## Event Examples

**Neighborhood News:**
> "New Business Opens"
> "A new café has opened in the neighborhood, adding to local charm."
> Effect: +5 neighborhood reputation

**Developer Offer:**
> "Developer Makes Offer"
> "A developer has expressed interest in purchasing Sunset Apartments for $320,000."
> Choices:
> - Accept Offer (sell building, -20 reputation)
> - Counter Offer (negotiate)
> - Decline (+5 reputation)

**Seasonal:**
> "Summer Heat Wave Warning"
> "Hot weather expected. Tenants are asking about AC."

---

## Design Philosophy

1. **Light Touch**: Events add flavor without taking over gameplay
2. **Player Agency**: Choices matter but don't gate progress
3. **Consequence Connection**: Events link to mechanical systems
4. **Character**: Tenants feel like people, not just stats
5. **Immersion**: World events make the city feel alive

---

## Testing
```rust
cargo test stories -- --nocapture
cargo test events -- --nocapture
cargo test mail -- --nocapture
```
