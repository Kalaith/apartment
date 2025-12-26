# Phase 4A: Economic Realism

## Goal
Transition from a "money printing" simulation to a tight financial management game where every dollar counts.

## Features

### 1. Ongoing Operating Costs
Implement a recurring expense system that hits the ledger every month.
- **Property Taxes**: 
  - Calculated as `%` of property value.
  - Varies by neighborhood (Downtown = high tax, Suburbs = low tax).
- **Utility Management**:
  - Base costs for Water, Electricity, Heating.
  - Option to "Include Utilities in Rent" (higher rent but you pay costs) vs "Tenant Pays Utilities" (lower rent, but risk of apartment decay if they don't pay).
- **Insurance**:
  - Required for mortgages or high-value buildings.
  - Cost reduced by: Fire alarms, security, good condition.

### 2. Staffing System
Hire employees to manage the portfolio.
- **Janitor**: Automatically repairs small condition drops (e.g., < 5 points) for a monthly salary.
- **Security**: Reduces crime/noise tension in the building.
- **Property Manager**: Filters mail and gives warnings on tenant unhappiness automatically.

### 3. Critical Failures (The "Money Pit")
Random high-impact events that require immediate capital.
- **Boiler Failure**: All apartments lose heating, massive happiness drop until fixed ($$$).
- **Structural Issues**: Hallway condition plummets, building risk increases.

## Technical Tasks
- [ ] Refactor `PlayerFunds` to handle periodic line items.
- [ ] Update `Building` struct to track "Service Level" (Staffing).
- [ ] Add `TaxProcessor` to the monthly tick.
- [ ] Create `CriticalEvent` variants in `src/simulation/events.rs`.
