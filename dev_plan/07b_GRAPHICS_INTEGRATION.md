# Task 07b: Graphics Integration Strategy
## Priority: 🟡 HIGH (Follow-up to Prompts)
## Dependencies: Task 07 (Visual Polish)
## Estimated Effort: 3-4 hours

## Objective
Implement the asset loading capability and update UI components to render the generated sprites instead of geometric primitives.

## Deliverables

### 1. Asset Management (`src/assets.rs`)
Create a central resource manager to handle texture loading and retrieval.

```rust
pub struct AssetManager {
    // Textures
    pub tenant_portraits: HashMap<TenantArchetype, Texture2D>,
    pub ui_icons: HashMap<String, Texture2D>,
    pub designs: HashMap<DesignType, Texture2D>,
    pub building_elements: HashMap<String, Texture2D>,
    
    // Status
    pub loaded: bool,
}

impl AssetManager {
    pub async fn load_all() -> Self {
        // Load all textures from assets/ directory
    }
}
```

### 2. UI Component Updates

**Building View (`src/ui/building_view.rs`):**
- Replace `draw_rectangle` for apartments with `draw_texture`.
- Layer the "design" texture (background) -> furniture/details -> tenant sprite (foreground).
- Use `building_exterior.png` for the main frame instead of `draw_rectangle_lines`.

**Apartment Panel (`src/ui/apartment_panel.rs`):**
- Show the `tenant_portrait` instead of just text name.
- Use `happiness_icon` textures instead of colored circles.
- Add `icon_*.png` to buttons (Repair, Upgrade, etc.).

**City View (`src/ui/city_view.rs`):**
- Use `neighborhood_backgrounds` for the background of panels/cards.
- Use `icon_market` and `icon_key` for listings.

**Header (`src/ui/header.rs`):**
- Add `icon_money`, `icon_calendar`, etc. next to values.

### 3. Texture Atlas Strategy
Since Macroquad handles separate textures well, we can stick to individual files for development ease, but consider packing them if performance drops (unlikely for this 2D sim).

### 4. Fallback Mode
Ensure the UI still renders (with colored rectangles) if assets are missing, to allow development to continue even if images aren't generated yet.

## New Dependencies
- `macroquad::texture::Texture2D`

## Integration Steps
1. Create `src/assets.rs` and add `AssetManager` to `GameplayState`.
2. Asynchronously load assets during game init (add loading screen?).
3. Refactor `draw_apartment_unit` to accept `&AssetManager`.
4. Refactor `draw_apartment_panel` to show portraits.
5. Update `common.rs` button helpers to support icons.
