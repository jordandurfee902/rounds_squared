# SETS - Project Standards

## Project Resources
- **Latest Bevy Documentation:** [https://docs.rs/bevy/latest/bevy/](https://docs.rs/bevy/latest/bevy/)
- **Note:** Always refer to the latest documentation when implementing new Bevy features.

## Project Overview
SETS is a blazingly fast clone of the game "Rounds" by Landfall Studios. It is built using the **Bevy Game Engine (v0.18)** and features a **custom physics engine** built from scratch in Rust.

The game involves players fighting in a 2D arena with simple geometric shapes. The loser of each round selects a card to upgrade their character, following the original game's core loop.

## Architecture & File Structure
To ensure easy navigation for AI agents and maintainable code, the project follows a highly modular structure.

### Directory Layout
- `src/main.rs`: **Minimal entry point.** Only for `App` initialization and plugin registration.
- `src/physics.rs`: Core physics components and systems (integration, collision detection, resolution).
- `src/player/`: Player-specific logic (controller, health, state).
- `src/combat/`: Projectiles, weapon systems, and card effects.
- `src/map/`: Static level geometry and level generation.
- `src/ui/`: Menus, HUD, and card selection interface.
- `src/graphics/`: Post-processing (Bloom), particles, and shaders.

## Development Rules

### 1. File Size Management
- **Target:** Keep files under **200 lines**.
- **Hard Limit:** Files should never exceed **400 lines**.
- **Rationale:** Larger files increase the likelihood of AI context errors and make debugging more difficult. If a file grows too large, split it into sub-modules (e.g., `src/player/movement.rs`, `src/player/input.rs`).

### 2. Lean `main.rs`
- `main.rs` must remain as clean as possible.
- It should strictly be used for:
    - Module declarations (`mod ...;`).
    - Adding `DefaultPlugins` and feature-specific `Plugins`.
    - Resource initialization (e.g., `ClearColor`).
- No gameplay logic, system implementations, or complex component definitions should reside here.

### 3. Component-Based Design
- Prefer small, reusable components.
- Group related systems into Bevy `Plugins`.
- Use `SystemSets` to manage execution order, especially for the custom physics engine.

### 4. Custom Physics
- Do NOT use external physics crates (like Rapier) unless explicitly requested.
- All collision and integration logic must be implemented in `src/physics.rs` or its sub-modules.
