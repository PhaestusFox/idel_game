# Voxel Quest - Development Plan (Expanded Draft)

## 1. Project Overview

- Working Title: Voxel Quest
- Genre: Idle / Incremental / Voxel Simulation
- Platform(s): Desktop
- Target Audience: Solo players who enjoy automation, optimization, and simulation sandboxes
- Team Size / Roles: 1 (Design, Engineering, Art Direction, Production)
- Planned Timeline: 6-9 months for a meaningful Early Access-style release
- Current Phase: Prototype

## 2. Vision

### 2.1 High Concept (1-3 sentences)

Voxel Quest is a deterministic, multi-scale exploration and automation game where the player gathers resources in a voxel world and gradually builds production systems that run while idle. Local changes are temporary, but player choices permanently alter world-generation pressures and economic efficiency. Acting at larger scales is faster, but equivalent in simulation value to detailed local play.

### 2.2 Pillars (3-5)

- Pillar 1: Deterministic voxel world (same seed + tile = same layout)
- Pillar 2: Multi-scale interaction (local, regional, global decisions)
- Pillar 3: Temporary edits, permanent systemic consequences
- Pillar 4: Idle growth through automation and logistics
- Pillar 5: Player-driven ecological tradeoffs

### 2.3 Unique Selling Points

- USP 1: Fully deterministic exploration loops for discoverability and replayability
- USP 2: World ecology responds to industrial behavior, not scripted events
- USP 3: Transparent open development (Steam + YouTube devlogs)

## 3. Player Experience

### 3.1 Player Fantasy

The player should feel constant curiosity and momentum: explore, discover, optimize, and scale. Growth should feel earned through understanding systems, not grinding random outcomes.
The player should make meaningful tradeoffs where increased production changes ecological conditions, making some resources easier and others harder to obtain.

- Example: increasing global temperature raises sea levels, improving water access but flooding previously productive land.

### 3.2 Session Types

- Short session (5-10 min): Gather targeted resources, claim one upgrade, queue one automation task
- Medium session (20-40 min): Build/optimize one production chain and unlock one traversal or logistics upgrade
- Long session (60+ min): Rework regional strategy and adjust environmental impact for long-term economy gains

### 3.3 Accessibility Goals

- Input accessibility: Keyboard/mouse baseline; controller support in Phase 2; remappable keybinds
- Visual accessibility: UI scale slider, high-contrast mode, colorblind-safe resource indicators
- Audio accessibility: Separate volume channels, optional subtitle/event text for key alerts

## 4. Core Gameplay Loop

### 4.1 Primary Loop

1. Player does: Explores a tile/region and gathers resources manually
2. Game responds with: Deterministic world generation and ecological state updates
3. Player earns: Resources, discoveries, and production knowledge
4. Player invests into: Tools, automation, logistics, and scale upgrades
5. Loop repeats with increased complexity by: Requiring rarer resources and better regional planning

### 4.2 Secondary Loops

- Progression loop: Unlock technologies for broader traversal and larger production throughput
- Economy loop: Build settlements specialized in extraction, processing, or logistics
- Exploration loop: Find more efficient biomes and optimize route planning
- Meta loop (long-term goals): Adapt to environmental shifts caused by industrial choices

### 4.3 Fail States and Recovery

- What can go wrong for players: Local species collapse, supply bottlenecks, over-specialized economies
- How players recover: Reintroduce species from other regions, use restoration tech, or pivot economy strategy

## 5. Systems Breakdown

### 5.1 World / Level Design

- World structure: Multi-layer voxel simulation with local tiles grouped into regions and global climate layer
- Generation approach: Coordinate-seeded deterministic noise stack (base terrain + biome mask + resource pass)
- Biomes / regions: Forest, wetland, plains, mountain, coast; each with distinct resource distributions
- Traversal: On foot early game, then transport network and strategic fast-travel later

### 5.2 Player Interaction

- Controls: WASD movement, mouse look/selection, interact key, hotbar for tools
- Interaction model: Raycast/select node -> contextual action (gather, inspect, place, automate)
- Feedback cues: Node outline, yield preview, action progress bar, economy delta popups

### 5.3 Resources and Economy

- Resource list: Wood, Fiber, Stone, Water, Biomass, Metal Ore, Energy, Research
- Source/sink map: Raw gathering -> processing stations -> logistics storage -> upgrade/maintenance sinks
- Conversion rules: Recipe-driven transforms with energy + labor costs and environmental side effects
- Automation rules: Buildings require setup cost, upkeep, and throughput caps affected by local conditions
- Offline progression rules: Fixed simulation cap (for example 8h), reduced efficiency when logistics bottlenecked

### 5.4 Progression and Unlocks

- Milestones: Local self-sufficiency -> regional trade network -> global climate management
- Upgrade paths: Exploration tools, extraction efficiency, processing speed, logistics capacity, restoration tech
- Gating rules: New tiers require resource diversity, not only total quantity
- Endgame goals: Sustain high-output economy while keeping ecological stability within thresholds

### 5.5 Difficulty and Balance

- Difficulty curve strategy: Rising complexity from input volume to system coupling and ecological constraints
- Tuning knobs: Yield rates, recipe costs, upkeep costs, transport throughput, climate sensitivity
- Anti-stall mechanics: Emergency contracts, temporary efficiency boosts, and partial refund on deconstruction

## 6. Content Plan

### 6.1 Content at Launch

- Areas/biomes: 3 at first playable milestone (Forest, Wetland, Plains)
- Resource types: 8 core resources + 3 rare catalysts
- Upgrades: 20-30 upgrades across 4 trees
- Events/encounters: Seasonal swings, migration events, resource bloom/decline windows

### 6.2 Post-Launch Content Ideas

- Update 1: Mountains + ore extraction chain + terrain hazard system
- Update 2: River logistics and hydro infrastructure
- Update 3: Advanced ecology tools and global policy layer

## 7. Technical Plan

### 7.1 Engine and Stack

- Engine/framework: Bevy
- Language(s): Rust
- Key dependencies: `bevy`, `serde`, `ron` (or `bincode` later), deterministic hashing crate, profiling tools

### 7.2 Architecture

- High-level modules: `world`, `generation`, `simulation`, `economy`, `ui`, `save`, `debug`
- Data ownership model: ECS components for runtime state, resources for global systems/config
- Save/load architecture: Versioned schema with migration path and deterministic seed persistence
- Config/data-driven design approach: External data files for rates, recipes, and biome distributions

### 7.3 Performance Targets

- Target FPS: 60 (playable floor 30 on minimum spec)
- Frame-time budget: 16.6ms target, with simulation capped to fixed timestep
- Max entities/chunks/nodes: Define prototype budgets and scale after profiling
- Min spec hardware: Mid-range CPU from last 5-7 years, modest dedicated/integrated GPU

### 7.4 Determinism and Reproducibility (if applicable)

- Seed strategy: Global world seed + hashed coordinates per layer and generation pass
- Deterministic rules: No frame-time-dependent logic in economy progression; fixed-step simulation only
- Validation tests: Snapshot tests for tile generation parity and offline progression consistency

## 8. UI/UX Plan

- HUD requirements: Resource bar, production rates, bottleneck indicators, local ecology status
- Menu flow: Main menu -> continue/new world -> in-game pause/settings -> save/quit
- Onboarding/tutorial: 5-step guided first session (gather, craft, automate, expand, stabilize)
- Clarity goals: Every major number should expose source and sink context in one click
- Style direction: Clean technical UI with natural-material accents and clear scale indicators

## 9. Audio Plan

- Music direction: Ambient, layered tracks that intensify with industrial scale
- SFX categories: Gathering, machinery, environmental ambiance, UI feedback
- Reactive audio triggers: Alerts for bottlenecks, ecosystem stress, and milestone unlocks

## 10. Production Roadmap

### 10.1 Milestones

1. Prototype Complete (date): 2026-05-15
2. Vertical Slice Complete (date): 2026-07-31
3. Content Complete (date): 2026-10-15
4. Beta (date): 2026-11-15
5. Release Candidate (date): 2026-12-15
6. Launch (date): 2027-01-15

### 10.2 Sprint Template

- Sprint goal: One clear capability milestone (for example deterministic tile generation + gather loop)
- Deliverables: Playable build, updated balancing config, test pass summary, short changelog
- Risks: Scope expansion, perf regression, deterministic mismatch bugs
- Dependencies: Asset placeholders, recipe data, test harness updates
- Demo checklist: Fresh seed test, revisit parity test, save/load sanity, 20-minute loop playtest

## 11. QA and Testing

- Unit tests scope: Hashing utilities, recipe calculations, economy tick math
- Integration tests scope: Gather -> process -> automate loops and progression unlock rules
- Performance tests: Chunk generation timing, mesh build timing, simulation tick cost under load
- Save/load test matrix: New save, old version migration, corrupted data handling, long-offline catch-up
- Regression checklist: Determinism parity, no negative-resource loops, no progression deadlocks

## 12. Analytics and Telemetry

- Key events to track: First automation built, first bottleneck resolved, first biome expansion, first extinction event
- KPI targets: Time-to-first-automation under 20 min, 60+ min median session for engaged players
- Retention goals: D1/D7 trend tracking in external playtests
- Economy health metrics: Inflation index, idle efficiency ratio, resource scarcity heatmap

## 13. Risks and Mitigation

| Risk | Impact | Likelihood | Mitigation | Owner |
|---|---|---|---|---|
| Scope creep from content ideas | High | High | Lock each sprint to one systems milestone | Solo dev |
| Determinism bugs across refactors | High | Medium | Add generation snapshot tests in CI | Solo dev |
| Simulation performance degradation | High | Medium | Profile each milestone before adding new systems | Solo dev |
| Balance dead zones (player stalls) | Medium | High | Add anti-stall contracts and rebalance weekly | Solo dev |
| Burnout from solo scope | High | Medium | Keep scope budget and ship small playable increments | Solo dev |

## 14. Open Questions

- How granular should local voxel editing be before it hurts simulation performance?
- Should offline progress model exact simulation replay or an approximate model with caps?
- What is the minimum fun loop that proves the concept in under 15 minutes?

## 15. Decision Log

Use this table to track major decisions and why they were made.

| Date | Decision | Reason | Owner |
|---|---|---|---|
| 2026-03-09 | Desktop-first development | Reduces tech complexity and speeds iteration | Solo dev |
| 2026-03-09 | Deterministic generation required | Core game promise and replay consistency | Solo dev |
| 2026-03-09 | Temporary local edits, persistent systemic effects | Supports both sandbox feel and long-term progression | Solo dev |

## 16. Definition of Done (Per Milestone)

- Gameplay criteria: Core loop is fun and comprehensible in a 15-minute fresh play session
- Technical criteria: Determinism checks pass and no blocking save/load errors
- Performance criteria: Meets target frame budget on minimum spec test machine
- QA criteria: No critical progression blockers or crash-on-load issues
- Documentation criteria: Milestone changelog and updated plan decisions recorded

## 17. Immediate Next Actions

1. Build deterministic tile generation prototype with one biome and one gatherable node type.
2. Implement gather -> inventory -> one conversion recipe -> one automation building.
3. Add save/load with seed + economy state and test round-trip.
4. Create first 20-minute onboarding path and run one full self-playtest.
