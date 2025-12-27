# Apartment Manager

A cozy apartment building management game built in Rust with [Macroquad](https://macroquad.rs/).

## About

You're not a property tycoon — you're the custodian of a half-dead apartment building with flickering lights, stained carpets, and stubborn tenants. Repair one thing at a time. Watch the building slowly remember what it was meant to be.

## Features

- **Progressive Building Unlock** - Start with a starter building and unlock harder properties by completing each challenge
- **Three Unique Buildings**:
  - **Sunset Apartments** (Easy) - 6-unit starter building with a helpful tenant
  - **The Meridian** (Medium) - 8-unit downtown tower with demanding tenants
  - **Blackwood Manor** (Hard) - 10-unit historic pyramid estate in poor condition
- **Apartment Management** - Repair and upgrade individual units, adjust rent, install soundproofing
- **Tenant System** - Accept applications from diverse archetypes (Students, Professionals, Artists, Families, Elderly)
- **Building Upgrades** - Improve hallways, install laundry facilities, upgrade designs
- **Narrative Elements** - Receive mail, handle tenant requests, complete missions

## Controls

| Key | Action |
|-----|--------|
| **Space** | End turn / Advance time |
| **ESC** | Pause menu (save, quit) |
| **Mouse** | Select apartments, click buttons |

## How to Win

Survive 3 years (36 months) managing your building. Keep tenants happy, maintain good condition, and stay financially afloat. Complete a building to unlock the next one!

## Building & Running

```bash
# Build and run (development)
cargo run

# Build release version
cargo run --release

# Build for WebGL
cargo build --release --target wasm32-unknown-unknown
```

## Publishing

Use the included publish script to create distributable packages:

```powershell
# Build Windows + WebGL packages
.\publish.ps1

# Windows only
.\publish.ps1 -WindowsOnly

# WebGL only  
.\publish.ps1 -WebGLOnly
```

Output packages are created in `dist/`:
- `apartment_manager_windows.zip` - Windows executable
- `apartment_manager_webgl.zip` - Browser-playable WebGL version

## Tech Stack

- **Rust** - Performance and safety
- **Macroquad** - 2D graphics and input
- **Serde** - Save/load game state
- **Rand** - Procedural generation

## Project Structure

```
src/
├── building/     # Apartments, upgrades, ownership
├── city/         # Neighborhoods, market
├── consequences/ # Gentrification, compliance, relationships
├── economy/      # Funds, transactions, ledger
├── data/         # Config, templates
├── narrative/    # Events, mail, tenant stories, missions
├── save/         # Game saves, player progress
├── simulation/   # Game tick, win conditions
├── state/        # Game state management
├── tenant/       # Tenants, applications, archetypes
└── ui/           # All UI components
assets/
├── building_templates.json  # Building definitions
├── config.json              # Game balance settings
└── *.png                    # Textures and sprites
```

## License

MIT
