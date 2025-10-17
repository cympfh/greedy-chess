# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a chess AI implementation in Rust that uses iterative deepening with min-max search. The program reads chess moves from stdin in various notations (SAN, UCI/LAN) and outputs the next best move within a specified time limit.

## Architecture

### Core Components

- **Board Representation** (`src/board.rs`): 64-square array with piece tracking, castling rights, en passant, and move counters
- **Move Parsing** (`src/board.rs`): Handles multiple chess notations:
  - SAN (Standard Algebraic Notation): e.g., "Nf3", "O-O", "exd5"
  - UCI/LAN format: e.g., "e2e4", "e7e8Q"
  - Castling: "O-O" (kingside), "O-O-O" (queenside)
- **Move Generation** (`src/board.rs`): Complete legal move generation for all piece types
- **Game State Management** (`src/board.rs`): Move application, castling rights, en passant tracking
- **Search Algorithm** (`src/board.rs`): Iterative deepening with alpha-beta pruning and parallel search support
- **Evaluation Function** (`src/evaluate.rs`): Two evaluation strategies (default: advanced with piece-square tables, classic: simple material count)
- **Caching System** (`src/cache.rs`): SHA256-based caching of board evaluations keyed by board state, timeout, thread count, and evaluator type
- **Opening Book** (`src/opening.rs`): Hardcoded common opening moves

### Key Data Structures

- `Board`: Main game state with 64-square representation
- `Move`: Encodes from/to squares, promotion, capture flags, special moves
- `Piece`: Kind (Pawn/Knight/Bishop/Rook/Queen/King) + Color (White/Black)

### Coordinate System

- Files: 0-7 (a-h), Ranks: 0-7 (1st-8th rank)
- Square indexing: `rank * 8 + file` (e.g., e1 = idx(4,0) = 4)

## Development Commands

### Building and Running
```bash
cargo build          # Compile the project
cargo run            # Run with stdin input (see README.md example)
cargo check          # Quick compile check
```

### Testing
```bash
cargo test           # Run tests (currently no tests exist)
```

### Input Format
The program expects chess moves as whitespace-separated tokens via stdin:
```bash
echo "e4 e5 Nf3 Nc6" | cargo run
echo "e4 e5 Nf3 Nc6" | cargo run -- --timeout 10  # 10 second search
```

## Algorithm Details

- **Search**: Iterative deepening with alpha-beta pruning
  - Starts at depth 1 and gradually increases depth
  - Returns best move found before timeout
  - Typical depth reached: 4-6 (with 5 second timeout)
  - Parallel search support with configurable thread count
- **Evaluation**: Two evaluation strategies selectable via `--evaluate` option:
  - **default** (advanced): Position evaluation with piece-square tables (centipawn scale)
    - **Material**: Pawn: 100, Knight: 320, Bishop: 330, Rook: 500, Queen: 900, King: 20000
    - **Piece-Square Tables**: Position-based bonuses/penalties for all piece types
      - Central knights/bishops: +15-20 bonus
      - Castled king (middlegame): +20-30 bonus
      - Active king (endgame): +30-40 bonus
    - **Strategic bonuses**: Bishop pair: +50, Castling rights: +15
  - **classic** (simple): Material-only evaluation for comparison
    - Pawn: 1, Knight: 3, Bishop: 3, Rook: 5, Queen: 9, King: 999
    - No position evaluation
- **Optimizations**:
  - Alpha-beta pruning: ~3-4x speedup over full minimax
  - Parallel search: Linear speedup with thread count
  - Caching: SHA256-based result caching keyed by (board_state, timeout, threads, evaluator)
  - Opening book: Hardcoded responses for common opening positions
- **Command-line options**:
  - `--timeout <seconds>` (default: 5): Search time limit
  - `--threads <n>` (default: serial): Parallel search with n threads
  - `--evaluate <type>` (default: default): Evaluation function (default or classic)
  - `--print-only`: Display board without calculating next move

## Key Implementation Details

- **Complete legal move generation**: Includes check/checkmate detection, castling validation, en passant
- **Modular architecture**: Separated into `board.rs`, `cache.rs`, `evaluate.rs`, `opening.rs`, and `main.rs`
- **Dependencies**:
  - Core: rayon (parallel search), serde (serialization), sha2 (caching)
  - CLI: clap (argument parsing)
- **Japanese comments** throughout the codebase

## Recent Changes (2025-10-17)

### Phase 1: Infrastructure
- Removed `--depth` option in favor of timeout-based iterative deepening
- Simplified function names (removed `_with_timeout` suffixes)
- Unified caching strategy around timeout values
- Cleaned up unused depth-based methods

### Phase 2: Performance Improvements
- ✅ **Parallel Search**: Added `-n/--threads` option for parallel move evaluation
- ✅ **Alpha-Beta Pruning**: Replaced minimax with alpha-beta pruning (~3-4x speedup)
- ✅ **Advanced Evaluation**: Created `evaluate.rs` module with piece-square tables
  - Position-aware evaluation for all piece types
  - Endgame vs middlegame king tables
  - Bishop pair and castling rights bonuses
- ✅ **Cache Enhancement**: Updated cache keys to include thread count

### Phase 3: Evaluation Function Selection
- ✅ **Dual Evaluation System**: Added `-e/--evaluate` option to switch between evaluation strategies
  - `default`: Advanced evaluation with piece-square tables (original implementation)
  - `classic`: Simple material-only evaluation (for comparison and testing)
- ✅ **Cache Key Update**: Updated cache system to include evaluator type in keys
- ✅ **Documentation**: Updated README.md and CLAUDE.md with new option