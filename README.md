# Citadels Team Server

A team-based Citadels game server implemented in Rust with support for AI agents, WebSocket communication, and game history recording.

## Overview

This project implements a multiplayer version of Citadels (碉堡游戏/七大奇迹) with team gameplay mechanics. The game supports 4 or 6 players divided into two teams (楚/Han and 汉/Chu). Players compete to build the most valuable district of buildings before their opponents.

## Features

- 🎮 **Full game implementation**: Complete Citadels mechanics with role selection, building construction, and special abilities
- 🤖 **AI Agent Support**: Multiple agent types (Random, V2, Noop, WS Proxy)
- 🌐 **WebSocket Integration**: Real-time communication for remote agents
- 📊 **Game History**: Detailed event logging and replay capability
- 📈 **Batch Simulation**: Run multiple games for win rate analysis
- 🎯 **Team-based gameplay**: Tension mode with 楚 vs 汉 factions

## Project Structure

```
citadels_team_server/
├── src/
│   ├── bin/              # Binary executables
│   │   ├── main.rs        # Main game server with WS agents
│   │   ├── sim.rs         # Batch simulation runner
│   │   ├── history.rs     # History recording service
│   │   └── ws_agent.rs    # WebSocket agent client
│   ├── domain/            # Game domain models
│   ├── fa_agents/         # First Action agents
│   ├── fyi_agents/        # FYI (For Your Information) agents
│   ├── obs/               # Observation models
│   ├── game.rs             # Main game engine
│   ├── player.rs           # Player implementation
│   ├── deck.rs             # Card deck management
│   ├── history.rs          # Game history tracking
│   └── ws_dispatcher.rs    # WebSocket message dispatcher
├── doc/
│   ├── rule.md            # Detailed game rules
│   └── naming.md          # Naming conventions
├── config.toml            # Server configuration
└── Cargo.toml             # Rust dependencies
```

## Prerequisites

- **Rust**: Version 1.70+ with Rust 2024 edition
- **Cargo**: Rust package manager
- **Tokio**: Runtime for async operations (included in dependencies)

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd citadels_team_server

# Build the project
cargo build --release

# Or install binaries
cargo install --path .
```

## Configuration

Create or edit `config.toml` in the project root:

```toml
host = "127.0.0.1"
port = 7001
history_uuid = "abb0f3fd-f725-4abd-bda6-829b5683b8bb"
ws_agent_uuid = "917c7861-185d-496c-82a1-51692a294a2e"
```

- `host`: WebSocket server host
- `port`: WebSocket server port
- `history_uuid`: UUID for history recording endpoint
- `ws_agent_uuid`: UUID for WebSocket agent endpoint

## Usage

### 1. Running the Main Server

Start the main game server:

```bash
cargo run --bin main
```

This starts:

- WebSocket dispatcher on configured host/port
- Game execution with AI agents
- Logging to `logs/main.log`

### 2. Batch Simulation

Run multiple games for statistical analysis:

```bash
cargo run --bin sim
```

This runs 1000 concurrent games and displays win rates for both teams.

### 3. Game History Service

Record and replay game events:

```bash
cargo run --bin history
```

Connects to the WebSocket server and logs all game events.

### 4. WebSocket Agent Client

Connect an agent to the server:

```bash
cargo run --bin ws_agent
```

This connects a client agent (e.g., V2FAAgent) to the server via WebSocket.

## Agent Types

### First Action (FA) Agents

- **RandomFAAgent**: Makes random valid choices
- **V2FAAgent**: Heuristic-based decision making
- **NoopFAAgent**: No-operation fallback agent
- **WsProxyFAAgent**: Proxies requests to WebSocket clients

### FYI Agents

- **NoopFYIAgent**: Receives game updates but doesn't influence gameplay
- Used for spectators or data collection

## Game Rules

See `doc/rule.md` for complete game rules. Summary:

- **Players**: 4 or 6 players (must be even)
- **Teams**: Two factions (楚/Han vs 汉/Chu)
- **Goal**: First team to build 8 buildings triggers endgame; highest total score wins
- **Rounds**: Role selection → Role execution → Building phase

### Core Mechanics

1. **Initial Setup**: Each player receives 2 gold and chooses 1 of 2 starting cards
2. **Role Selection**: Players secretly choose roles from available pool
3. **Role Execution**: Roles execute in number order (刺客, 小偷, 换牌师, etc.)
4. **Building**: Players use gold and permits to construct buildings
5. **Victory**: First to 8 buildings ends round; team with highest score wins

### Special Buildings

- **天文台** (Observatory): Draw 3, choose 1
- **图书馆** (Library): Draw 2, keep both
- **铁匠铺** (Smithy): Pay 2 gold to draw 3 cards
- **实验室** (Laboratory): Sell a card for 1 gold
- **墓地** (Graveyard): Buy destroyed buildings for 1 gold
- **城墙** (City Wall): Buildings cost +1 gold to destroy
- **要塞** (Fortress): Cannot be destroyed

## Logging

Logs are written to `logs/` directory:

- `main.log`: Main game server logs
- `sim.log`: Simulation run logs
- `history.log`: Game history events
- `ws_agent.log`: WebSocket agent communication

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Project Statistics

- **Source Files**: 48 Rust files
- **Lines of Code**: ~5,200
- **Dependencies**: Tokio, Serde, WebSocket, Tracing

## Key Components

### Game Engine (`src/game.rs`)

Core game logic including:
- Round management
- Role selection and execution
- Building construction
- Special ability handling
- Victory condition checking

### Players (`src/player.rs`)

Player state management:

- Gold and permits tracking
- Buildings collection
- Role assignment
- Scoring calculation

### History (`src/history.rs`)

Event recording system:

- Game events (1,176 lines)
- Request/response tracking
- JSON serialization
- WebSocket broadcasting

### WebSocket Dispatcher (`src/ws_dispatcher.rs`)

Real-time communication:

- Multiple endpoint support
- Client connection management
- Message broadcasting
- Connection state tracking

## Architecture

### Async Model

Built on Tokio async runtime:

- Non-blocking I/O
- Concurrent game simulations
- Channel-based message passing
- Spawned tasks for parallel execution

### Type Safety

- PlayerIndex: Type-safe player indexing
- PlayerOffset: Relative player positioning
- RoleSet: Bit-set for role tracking
- Role-based game mechanics

### Traits

- `AbstractFAAgent`: First action decision making
- `AbstractFYIAgent`: Information collection
- Polymorphic agent swapping

## Performance

### Simulation Speed

Run 1000 games in batch mode:

```bash
cargo run --bin sim
# Example output:
# num_games: 1000
# Time taken: 45.2s
# win rate 楚: 0.523
# win rate 汉: 0.477
```

### Memory Usage

- Efficient Vec<> usage
- Arc<RwLock<>> for shared state
- Minimal cloning with reference passing

## Troubleshooting

### Common Issues

1. **Port already in use**: Change port in `config.toml`
2. **Missing config.toml**: Create from example above
3. **Log directory**: Ensure `logs/` directory exists

### Debug Mode

```bash
RUST_LOG=debug cargo run --bin main
```

## Contributing

See TODO items in codebase for potential improvements:

- [ ] Add FinishGame event to history
- [ ] Implement proper error handling (144 unwrap() calls)
- [ ] Add comprehensive tests
- [ ] Reduce History.rs duplication
- [ ] Optimize observation updates

## License

[Add your license here]

## Authors

[Add author information]

## Related Projects

- Original Citadels board game
- Similar implementations in other languages
- Game AI research applications
