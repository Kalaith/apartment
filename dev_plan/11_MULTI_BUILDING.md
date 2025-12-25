# Task 11: Multi-Building System

## Priority: 🔴 CRITICAL (Core Expansion)
## Dependencies: Task 10 (Neighborhood)
## Estimated Effort: 5-6 hours

## Objective
Enable players to acquire and manage multiple buildings across different neighborhoods, with a city-wide view and property market.

## Implementation Status: ✅ COMPLETE

### Files Created
- `src/city/city.rs` - City struct managing all buildings
- `src/city/market.rs` - Property market and listings

### Features Implemented

1. **City Layer** (`City` struct)
   - Top-level game world containing neighborhoods
   - Manages all buildings in the game
   - Tracks active building selection
   - Global economic factors (economy health, interest rate, inflation)
   - Monthly tick updates all city systems

2. **Property Market** (`PropertyMarket` struct)
   - Dynamic property listings appear periodically
   - Listings include name, condition, price, existing tenants
   - Prices vary by neighborhood, condition, and market demand
   - Listings age and prices may drop over time

3. **Building Acquisition**
   - `PropertyListing` defines available buildings
   - `BuildingCondition` enum (Condemned, Poor, Fair, Good, Excellent)
   - Multiple financing options:
     - Cash purchase
     - Bank mortgage (down payment + monthly payments)
     - Investor partner (investment for profit share)

4. **Portfolio Management**
   - `active_building_index` for current view
   - `switch_building()` to change active building
   - `buildings_with_info()` for portfolio display
   - `total_property_value()` for net worth calculation

### Key Methods

**City:**
- `City::new()` - Create with default neighborhoods
- `City::with_starter_building()` - Initialize with one building
- `add_building()` - Add purchased building to neighborhood
- `active_building()` / `active_building_mut()` - Current building
- `tick()` - Monthly city updates

**Market:**
- `PropertyMarket::refresh_listings()` - Generate new listings
- `PropertyListing::generate()` - Create random listing
- `PropertyListing::to_building()` - Convert to Building struct
- `FinancingOption::upfront_cost()` - Calculate initial payment
- `FinancingOption::monthly_payment()` - Calculate ongoing costs

### UI Components
- `src/ui/city_view.rs` - City map and portfolio panel
- `draw_city_map()` - Neighborhood grid
- `draw_portfolio_panel()` - Building list
- `draw_market_panel()` - Property listings

## Integration Points
- `GameplayState` needs to use `City` instead of single `Building`
- Tenants need building ID association
- Economy needs portfolio-level calculations
- Save system needs city serialization

## Future Enhancements
- Building loans with monthly payments
- Property insurance
- Building sales and disposal
- Management delegation options
