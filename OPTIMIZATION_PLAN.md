# Citadels Team Server - Optimization Plan

## Executive Summary

This document outlines a comprehensive optimization plan for the Citadels Team Server codebase, covering architecture, design, error handling, code quality, performance, and maintainability improvements.

## Current State Analysis

### Statistics
- **Total Files**: 48+ Rust source files
- **Lines of Code**: ~5,200
- **Large Files**: 
  - `src/history.rs`: ~1,200 lines (needs modularization)
  - `src/game.rs`: ~790 lines (needs splitting by responsibility)
- **Error Handling Issues**: 148 `unwrap()` calls across 10 files
- **Logging Issues**: 54 `println!` calls across 8 files
- **TODOs**: 31 TODO comments identified

### Key Issues Identified

1. **Architecture & Structure**
   - Large monolithic files (history.rs, game.rs)
   - Tight coupling between components
   - Missing service layer abstraction
   - Inconsistent module organization

2. **Error Handling**
   - Excessive use of `unwrap()` (148 instances)
   - No proper error types or error propagation
   - Missing error recovery mechanisms

3. **Code Quality**
   - Magic numbers (66 cards, 8 buildings) not extracted as constants
   - Code duplication in history.rs
   - Inconsistent naming conventions
   - Missing documentation

4. **Performance**
   - Inefficient iteration patterns (using indices instead of iterators)
   - Unnecessary cloning in observation updates
   - Missing connection lifecycle management for WebSocket

5. **Maintainability**
   - Mixed languages (Chinese/English) in code
   - TODOs scattered throughout codebase
   - Missing unit tests
   - Incomplete error handling

---

## Phase 1: Architecture & Design Improvements

### 1.1 Module Restructuring

#### Split `src/history.rs` (~1,200 lines)
**Target**: Break into focused modules

**New Structure**:
```
src/history/
├── mod.rs                 # History struct and core implementation
├── events.rs              # Event type definitions (HistoryReqEvent, HistoryRespEvent)
├── constants.rs           # Event name constants
├── builder.rs             # Event builder methods
└── sink.rs                # Abstract sink trait for different backends
```

**Benefits**:
- Better organization and maintainability
- Easier to add new event types
- Allows for pluggable sinks (WebSocket, file, database, etc.)

**Estimated Effort**: 4-6 hours

#### Split `src/game.rs` (~790 lines)
**Target**: Separate game phases into distinct modules

**New Structure**:
```
src/game/
├── mod.rs                 # Game struct and main run loop
├── initialization.rs      # init_gold, init_card, init_obs
├── round.rs               # run_round, choose_role
├── role_execution.rs      # execute_player_turn, role-specific logic
├── building.rs            # Building phase logic
├── victory.rs             # Victory condition checking
└── stats.rs               # RoundStats and game statistics
```

**Benefits**:
- Clear separation of concerns
- Easier to test individual phases
- Better code navigation

**Estimated Effort**: 6-8 hours

### 1.2 Extract Game Constants

**Create**: `src/constants.rs`

```rust
pub const TOTAL_CARDS: usize = 66;
pub const WINNING_BUILDING_COUNT: usize = 8;
pub const INITIAL_GOLD: u32 = 2;
pub const INITIAL_CARD_CHOICES: usize = 2;
pub const MIN_PLAYERS: usize = 4;
pub const MAX_PLAYERS: usize = 6;
pub const MAX_ROLES: usize = 8;
// ... etc
```

**Replace**: All magic numbers throughout codebase

**Estimated Effort**: 2-3 hours

### 1.3 Introduce Service Layer

**Create**: `src/services/` directory

- `game_service.rs`: Orchestrates game flow
- `player_service.rs`: Player state management
- `history_service.rs`: History recording abstraction
- `agent_service.rs`: Agent coordination

**Benefits**:
- Decouple business logic from implementation details
- Easier to mock for testing
- Better dependency injection

**Estimated Effort**: 8-10 hours

### 1.4 Repository Pattern for History

**Create**: `src/history/sink.rs`

```rust
pub trait HistorySink: Send + Sync {
    async fn record_event(&mut self, event: &HistoryReqEvent) -> Result<()>;
    async fn wait_for_ready(&mut self) -> Result<()>;
}
```

**Implementations**:
- `WebSocketHistorySink`: Current WebSocket implementation
- `FileHistorySink`: File-based logging
- `DatabaseHistorySink`: Future database integration
- `MemoryHistorySink`: For testing

**Benefits**:
- Pluggable backends
- Better testability
- Separation of concerns

**Estimated Effort**: 4-6 hours

---

## Phase 2: Error Handling Improvements

### 2.1 Create Error Types

**Create**: `src/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("Invalid player index: {0}")]
    InvalidPlayerIndex(usize),
    
    #[error("Deck exhausted")]
    DeckExhausted,
    
    #[error("Invalid game state: {0}")]
    InvalidGameState(String),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    // ... etc
}
```

**Benefits**:
- Type-safe error handling
- Better error messages
- Proper error propagation

**Estimated Effort**: 3-4 hours

### 2.2 Replace unwrap() Calls

**Strategy**:
1. Identify all `unwrap()` calls (148 instances)
2. Categorize by context:
   - **Critical**: Must handle with `?` or proper error handling
   - **Assertions**: Replace with `expect()` with descriptive messages
   - **Option handling**: Use `unwrap_or()`, `unwrap_or_else()`, or proper error handling
3. Replace systematically by module

**Priority Files**:
1. `src/game.rs` (6 unwrap calls)
2. `src/history.rs` (97 unwrap calls)
3. `src/ws_dispatcher.rs` (9 unwrap calls)
4. `src/bin/main.rs`, `src/bin/ws_agent.rs`, etc.

**Estimated Effort**: 12-16 hours

### 2.3 Add Error Recovery

- WebSocket reconnection logic
- Graceful degradation when agents fail
- Better error messages for debugging

**Estimated Effort**: 4-6 hours

---

## Phase 3: Logging Consistency

### 3.1 Replace println! with tracing

**Target**: 54 `println!` calls across 8 files

**Strategy**:
- Replace `println!` with appropriate `tracing::info!`, `tracing::debug!`, etc.
- Use structured logging with fields
- Maintain log levels appropriately

**Files to Update**:
- `src/bin/main.rs` (7 instances)
- `src/bin/sim.rs` (4 instances)
- `src/bin/ws_agent.rs` (26 instances)
- `src/bin/history.rs` (7 instances)
- `src/ws_dispatcher.rs` (3 instances)
- `src/game.rs` (5 instances)
- `src/history.rs` (1 instance)

**Estimated Effort**: 3-4 hours

### 3.2 Standardize Logging Format

- Use consistent field names
- Add tracing spans for request tracking
- Improve log messages (English)

**Estimated Effort**: 2-3 hours

---

## Phase 4: Code Quality Improvements

### 4.1 Fix TODOs

**Priority TODOs**:
1. ✅ Extract constants (66 cards, 8 buildings)
2. ✅ Replace unwrap() calls
3. ✅ Replace println! with tracing
4. ✅ WebSocket join handles
5. ✅ History sink abstraction
6. ✅ Extract magic numbers
7. Improve naming (Chinese method names → English)

**Estimated Effort**: 8-10 hours

### 4.2 Reduce Code Duplication

**Areas**:
- History event serialization patterns
- Observation update logic
- Agent request/response handling

**Estimated Effort**: 4-6 hours

### 4.3 Improve Naming

**Strategy**:
- Create naming conventions document
- Gradually migrate Chinese names to English
- Use descriptive names

**Estimated Effort**: 4-6 hours

### 4.4 Add Documentation

**Add**:
- Module-level documentation
- Function-level documentation
- Inline comments for complex logic
- Architecture decision records (ADRs)

**Estimated Effort**: 6-8 hours

---

## Phase 5: Performance Optimizations

### 5.1 Optimize Iteration Patterns

**Issue**: Using indices instead of direct iteration

**Example** (from game.rs):
```rust
// Current
for i in (0..self.players.len()).map(PlayerIndex::from_usize) {
    // ...
}

// Optimized
for (idx, player) in self.players.iter().enumerate() {
    let i = PlayerIndex::from_usize(idx);
    // ...
}
```

**Estimated Effort**: 3-4 hours

### 5.2 Reduce Cloning

**Areas**:
- Observation updates
- History event serialization
- Agent communication

**Strategy**:
- Use references where possible
- Use `Arc` for shared immutable data
- Cache computed values

**Estimated Effort**: 4-6 hours

### 5.3 WebSocket Connection Management

**Improvements**:
- Proper join handle tracking
- Connection pooling
- Graceful shutdown
- Resource cleanup

**Estimated Effort**: 4-6 hours

---

## Phase 6: Testing & Validation

### 6.1 Add Unit Tests

**Coverage Target**: 60%+ for core logic

**Priority Areas**:
- Game state transitions
- Player operations
- Role execution logic
- Card deck management
- Victory condition checking

**Estimated Effort**: 12-16 hours

### 6.2 Add Integration Tests

**Areas**:
- Full game simulation
- WebSocket communication
- Agent integration
- History recording

**Estimated Effort**: 8-10 hours

### 6.3 Add Property-Based Tests

**Using**: `proptest` crate

**Focus**:
- Game invariants (e.g., total cards always = 66)
- State transitions
- Agent decision making

**Estimated Effort**: 6-8 hours

---

## Phase 7: Additional Improvements

### 7.1 Configuration Management

**Improvements**:
- Validate configuration on load
- Environment variable support
- Default configuration values
- Configuration hot-reloading (future)

**Estimated Effort**: 3-4 hours

### 7.2 Metrics & Observability

**Add**:
- Game duration metrics
- Win rate tracking
- Agent performance metrics
- WebSocket connection metrics

**Estimated Effort**: 6-8 hours

### 7.3 Code Formatting & Linting

**Ensure**:
- Consistent formatting (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
- Pre-commit hooks (already has `.pre-commit-config.yaml`)

**Estimated Effort**: 2-3 hours

---

## Implementation Priority

### High Priority (Do First)
1. ✅ Extract constants (2-3 hours)
2. ✅ Replace critical unwrap() calls (8-10 hours)
3. ✅ Replace println! with tracing (3-4 hours)
4. ✅ Split history.rs into modules (4-6 hours)

### Medium Priority (Do Next)
5. ✅ Split game.rs into modules (6-8 hours)
6. ✅ Create error types (3-4 hours)
7. ✅ Replace remaining unwrap() calls (4-6 hours)
8. ✅ Fix WebSocket join handles (2-3 hours)

### Low Priority (Nice to Have)
9. ✅ Service layer introduction (8-10 hours)
10. ✅ Add unit tests (12-16 hours)
11. ✅ Performance optimizations (8-12 hours)
12. ✅ Documentation improvements (6-8 hours)

---

## Estimated Total Effort

- **High Priority**: 17-23 hours
- **Medium Priority**: 15-21 hours
- **Low Priority**: 34-46 hours
- **Total**: 66-90 hours (~2-3 weeks for one developer)

---

## Success Criteria

1. ✅ No `unwrap()` calls in production code (only in tests or with proper error handling)
2. ✅ All `println!` replaced with structured logging
3. ✅ Large files split into focused modules
4. ✅ All magic numbers extracted as constants
5. ✅ Proper error types and error propagation
6. ✅ At least 60% test coverage for core logic
7. ✅ Zero clippy warnings
8. ✅ Consistent code formatting
9. ✅ Clear module boundaries and separation of concerns

---

## Migration Strategy

### Incremental Approach

1. **Phase 1-2**: Architecture + Error handling (no breaking changes)
2. **Phase 3**: Logging (backward compatible)
3. **Phase 4**: Code quality (gradual improvements)
4. **Phase 5-6**: Performance + Testing (additive)

### Backward Compatibility

- Maintain public API compatibility
- Gradual migration of internal APIs
- Feature flags for new implementations (if needed)

---

## Tools & Dependencies

### New Dependencies Needed
```toml
[dependencies]
thiserror = "1.0"           # Error types
proptest = "1.0"            # Property-based testing
```

### Existing Tools
- `cargo fmt`: Code formatting
- `cargo clippy`: Linting
- `cargo test`: Testing
- `tracing`: Already in use for logging

---

## Notes

- This plan is comprehensive but flexible
- Prioritize based on project needs
- Some improvements can be done in parallel
- Consider code review and pair programming for complex refactoring
- Maintain git history with meaningful commit messages

---

## Next Steps

1. Review and approve this plan
2. Create GitHub issues for each phase
3. Start with high-priority items
4. Regular progress reviews
5. Update plan based on learnings

