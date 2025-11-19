# Citadels Team Server - Architecture & Design Plan

## Current Architecture Issues

### 1. Monolithic Structure
- **Problem**: Core logic concentrated in two large files (`game.rs` ~790 lines, `history.rs` ~1,200 lines)
- **Impact**: Hard to maintain, test, and extend
- **Root Cause**: Missing separation of concerns and modular boundaries

### 2. Tight Coupling
- **Problem**: Direct dependencies between components
- **Impact**: Difficult to test in isolation, changes ripple across modules
- **Root Cause**: No abstraction layers or dependency injection

### 3. Mixed Responsibilities
- **Problem**: Game logic, state management, I/O, and serialization all mixed
- **Impact**: Violates single responsibility principle
- **Root Cause**: Missing service layer and clear boundaries

### 4. Error Handling Anti-patterns
- **Problem**: 148 `unwrap()` calls, no error types
- **Impact**: Crashes on unexpected conditions, poor error messages
- **Root Cause**: Missing error domain model

---

## Proposed Architecture

### High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   main.rs    │  │    sim.rs     │  │  ws_agent.rs │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                      Service Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ GameService  │  │ PlayerService│  │HistoryService│     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐  ┌──────────────┐                       │
│  │ AgentService │  │RoundService  │                       │
│  └──────────────┘  └──────────────┘                       │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                      Domain Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │     Game      │  │    Player    │  │     Deck     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │     Round     │  │    Role      │  │  Observation │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  WsDispatcher│  │ HistorySink  │  │   Agents     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                        │                                    │
│                  ┌──────┴──────┐                           │
│                  │             │                           │
│            ┌─────▼───┐   ┌────▼────┐                       │
│            │WebSocket│   │  File   │  [Future: Database]  │
│            │  Sink   │   │  Sink   │                       │
│            └─────────┘   └─────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Design Principles

### 1. Layered Architecture
- **Application Layer**: Entry points (binaries)
- **Service Layer**: Orchestration and business workflow
- **Domain Layer**: Core game logic and entities
- **Infrastructure Layer**: External dependencies (WebSocket, I/O)

### 2. Dependency Inversion
- High-level modules depend on abstractions (traits)
- Low-level modules implement traits
- Enables testing and flexibility

### 3. Single Responsibility
- Each module has one clear purpose
- Game phases split into separate modules
- History recording separated from game logic

### 4. Repository Pattern
- Abstract data access (HistorySink trait)
- Multiple implementations (WebSocket, File, Database)
- Testable with in-memory implementations

---

## Module Structure

### Proposed Directory Structure

```
src/
├── lib.rs                    # Public API exports
│
├── application/              # Application layer
│   ├── mod.rs
│   └── commands/             # Command handlers
│       ├── mod.rs
│       ├── run_game.rs
│       └── simulate.rs
│
├── services/                 # Service layer
│   ├── mod.rs
│   ├── game_service.rs       # Game orchestration
│   ├── player_service.rs     # Player operations
│   ├── round_service.rs       # Round management
│   ├── role_service.rs        # Role execution
│   ├── history_service.rs     # History abstraction
│   └── agent_service.rs       # Agent coordination
│
├── domain/                   # Domain layer (already exists)
│   ├── mod.rs                # Existing domain types
│   ├── game/
│   │   ├── mod.rs            # Game struct
│   │   ├── state.rs          # Game state
│   │   ├── phases.rs         # Game phases enum
│   │   └── victory.rs        # Victory conditions
│   ├── round/
│   │   ├── mod.rs            # Round struct
│   │   ├── role_selection.rs # Role selection logic
│   │   └── execution.rs      # Round execution
│   └── player/
│       ├── mod.rs            # Player struct
│       └── operations.rs     # Player actions
│
├── history/                  # History module (split from history.rs)
│   ├── mod.rs                # History trait/struct
│   ├── events.rs             # Event type definitions
│   ├── builder.rs            # Event building helpers
│   ├── sink.rs               # Sink trait
│   └── sinks/
│       ├── mod.rs
│       ├── websocket_sink.rs # WebSocket implementation
│       ├── file_sink.rs      # File logging
│       └── memory_sink.rs    # Testing
│
├── agents/                    # Agent infrastructure
│   ├── mod.rs
│   ├── traits.rs             # AbstractFAAgent, AbstractFYIAgent
│   ├── registry.rs           # Agent registration
│   └── implementations/      # Existing agent implementations
│       ├── random_agent.rs
│       ├── v2_agent.rs
│       └── ws_proxy_agent.rs
│
├── infrastructure/           # Infrastructure layer
│   ├── mod.rs
│   ├── websocket/
│   │   ├── mod.rs
│   │   ├── dispatcher.rs     # WsDispatcher
│   │   └── connection.rs      # Connection management
│   └── config/
│       ├── mod.rs
│       └── loader.rs          # Config loading
│
├── error.rs                   # Error types
├── constants.rs               # Game constants
└── util.rs                    # Utilities
```

---

## Component Design

### 1. Game Service (Orchestrator)

**Purpose**: Coordinate game flow, delegate to specialized services

**Responsibilities**:
- Initialize game state
- Coordinate round execution
- Manage game lifecycle
- Handle game completion

**Interface**:
```rust
pub struct GameService {
    round_service: RoundService,
    player_service: PlayerService,
    history_service: HistoryService,
    agent_service: AgentService,
}

impl GameService {
    pub async fn initialize_game(&mut self, config: GameConfig) -> Result<GameId>;
    pub async fn run_game(&mut self, game_id: GameId) -> Result<GameResult>;
}
```

### 2. Round Service

**Purpose**: Manage individual round logic

**Responsibilities**:
- Role selection phase
- Role execution phase
- Round statistics tracking

**Interface**:
```rust
pub struct RoundService {
    role_selection: RoleSelectionService,
    role_execution: RoleExecutionService,
}

impl RoundService {
    pub async fn execute_round(&mut self, round: u32, game: &mut Game) -> Result<RoundStats>;
}
```

### 3. History Service (Abstraction)

**Purpose**: Decouple history recording from implementation

**Responsibilities**:
- Provide history recording interface
- Route events to configured sinks
- Manage history lifecycle

**Interface**:
```rust
pub trait HistorySink: Send + Sync {
    async fn record_event(&mut self, event: HistoryEvent) -> Result<()>;
    async fn wait_for_ready(&mut self) -> Result<()>;
}

pub struct HistoryService {
    sinks: Vec<Box<dyn HistorySink>>,
}

impl HistoryService {
    pub async fn record(&mut self, event: HistoryEvent) -> Result<()>;
}
```

### 4. Agent Service

**Purpose**: Coordinate agent interactions

**Responsibilities**:
- Manage agent lifecycle
- Route requests to agents
- Handle agent failures
- Coordinate parallel agent calls

**Interface**:
```rust
pub struct AgentService {
    agents: PlayerIndexedVec<Box<dyn AbstractFAAgent>>,
    fallback_agent: Box<dyn AbstractFAAgent>,
}

impl AgentService {
    pub async fn choose_role(&mut self, player: PlayerIndex, obs: &Obs, choices: RoleSet) -> Result<Role>;
    pub async fn parallel_choose_init_card(&mut self, choices: Vec<(PlayerIndex, Card, Card)>) -> Result<Vec<(PlayerIndex, Card)>>;
}
```

---

## Design Patterns

### 1. Strategy Pattern (Agents)
- **Current**: Trait-based agents (`AbstractFAAgent`)
- **Improvement**: Registry pattern for dynamic agent selection

### 2. Repository Pattern (History)
- **Current**: Direct WebSocket coupling
- **Improvement**: `HistorySink` trait with multiple implementations

### 3. Service Layer Pattern
- **Current**: Direct domain object manipulation
- **Improvement**: Services orchestrate domain operations

### 4. Factory Pattern (Game Creation)
- **Current**: Manual game construction in binaries
- **Improvement**: `GameFactory` for consistent game setup

### 5. Observer Pattern (FYI Agents)
- **Current**: Direct calls to FYI agents
- **Improvement**: Event-driven notifications

---

## Key Design Decisions

### 1. Error Handling Strategy
- **Custom Error Types**: Domain-specific error enum
- **Result Propagation**: Use `?` operator consistently
- **Error Context**: Rich error messages with context

### 2. Async Strategy
- **Current**: Tokio async runtime
- **Maintain**: Keep async model
- **Improve**: Better error handling in async contexts

### 3. State Management
- **Current**: Mutable Game struct
- **Consider**: Event sourcing (future enhancement)
- **Now**: Keep mutable state, improve encapsulation

### 4. Testing Strategy
- **Unit Tests**: Service layer with mock dependencies
- **Integration Tests**: Full game simulation
- **Property Tests**: Game invariants

---

## Migration Path

### Phase 1: Foundation (Week 1)
1. Create error types (`src/error.rs`)
2. Extract constants (`src/constants.rs`)
3. Create HistorySink trait (new module structure)

### Phase 2: Split & Refactor (Week 2)
1. Split `history.rs` into modules
2. Split `game.rs` into modules
3. Extract services from game logic

### Phase 3: Service Layer (Week 3)
1. Implement GameService
2. Implement RoundService
3. Migrate binaries to use services

### Phase 4: Polish (Week 4)
1. Replace all unwrap() calls
2. Replace println! with tracing
3. Add tests
4. Documentation

---

## Benefits

### Maintainability
- Clear module boundaries
- Easier to locate code
- Reduced cognitive load

### Testability
- Mockable dependencies
- Isolated unit tests
- Integration test support

### Extensibility
- Easy to add new agent types
- Pluggable history backends
- New game modes possible

### Reliability
- Proper error handling
- Better error messages
- Graceful failure modes

---

## Success Metrics

1. **Module Size**: All modules < 300 lines
2. **Coupling**: Clear dependency hierarchy
3. **Testability**: 60%+ test coverage
4. **Error Handling**: Zero unwrap() in production code
5. **Documentation**: All public APIs documented

---

## Future Enhancements (Out of Scope)

1. **Event Sourcing**: Full game event replay
2. **Database Backend**: Persistent game storage
3. **Distributed Agents**: Remote agent execution
4. **GraphQL API**: Modern API layer
5. **Metrics Dashboard**: Real-time game statistics

