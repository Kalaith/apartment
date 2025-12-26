# Apartment Manager

A cozy apartment building management game built in Rust with [Macroquad](https://macroquad.rs/).

## About

You're not a property tycoon — you're the custodian of a half-dead apartment building with flickering lights, stained carpets, and stubborn tenants. Repair one thing at a time. Watch the building slowly remember what it was meant to be.

## Features

- **Apartment Management** - Repair and upgrade individual units, adjust rent prices, install soundproofing
- **Tenant System** - Accept applications from diverse archetypes (Students, Professionals, Artists, Families, Retirees), each with unique preferences
- **Building Upgrades** - Improve hallways, install laundry facilities, upgrade shared spaces
- **City Expansion** - Purchase additional buildings across different neighborhoods
- **Ownership Options** - Keep units as rentals or sell them as condos
- **Narrative Elements** - Receive mail, handle tenant requests, watch tenant stories unfold

## Controls

| Key | Action |
|-----|--------|
| **Space** | End turn / Advance time |
| **Tab** | Toggle between Building and City views |
| **ESC** | Pause menu (settings, save, quit) |
| **Mouse** | Select apartments, click buttons |

## Building & Running

```bash
# Build and run
cargo run

# Build release version
cargo run --release
```

## Tech Stack

- **Rust** - Performance and safety
- **Macroquad** - 2D graphics and input
- **Serde** - Save/load game state
- **Rand** - Procedural generation

## Project Structure

```
src/
├── building/     # Apartments, upgrades, ownership
├── city/         # Neighborhoods, market, multi-building
├── consequences/ # Gentrification, compliance, relationships
├── economy/      # Funds, transactions, ledger
├── narrative/    # Events, mail, tenant stories
├── simulation/   # Game tick, win conditions
├── state/        # Game state management
├── tenant/       # Tenants, applications, archetypes
└── ui/           # All UI components
```

## License

MIT
