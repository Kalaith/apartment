# Task 07: Visual Polish & Juice
## Priority: 🟡 HIGH (Quality of Life)
## Dependencies: Task 06 (UI)
## Estimated Effort: 3-4 hours

## Objective
Transform the utilitarian developer UI into a responsive, satisfying game interface. Add feedback loops that make actions feel meaningful.

## deliverables

### 1. Visual Feedback System
- **Floating Text**: create a temporary entity system for visual markers that float up and fade out.
  - `+$500` (Green) when rent collected.
  - `-$50` (Red) when repairing.
  - `+Happiness` (Hearts) when requests filled.
- **Selection Highlights**: specific, high-contrast borders for selected apartments.

### 2. Animation System
- **Tweening Helper**: simple linear/ease-out interpolation utility.
- **Panel Transitions**: Side panels should slide in from off-screen rather than popping.
- **Button States**: Visual depress/scale effect on click.

### 3. Tenant Visualization
- **Distinct Colors**: Each archetype (Student, Professional, Artist) needs a distinct color palette in the UI.
- **Happiness Icons**: Replace the simple bar with dynamic icons (😭 ☹️ 😐 🙂 😃).

### 4. Layout Improvements
- **Grid Spacing**: Adjust building view to handle varying window sizes better.
- **Font Hierarchy**: clearer distinction between headers, labels, and values.

## Implementation Notes
- Keep immediate mode paradigm; animations will need state tracking (e.g., `panel_offset`).
- `FloatingText` struct will need to be added to `GameplayState` and updated every frame.
