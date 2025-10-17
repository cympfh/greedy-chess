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
- **Search Algorithm** (`src/board.rs`): Iterative deepening with time-bounded min-max search
- **Caching System** (`src/cache.rs`): SHA256-based caching of board evaluations keyed by board state and timeout
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

- **Search**: Iterative deepening min-max algorithm with timeout
  - Starts at depth 1 and gradually increases depth
  - Returns best move found before timeout
  - Typical depth reached: 5-7 (with 5 second timeout)
- **Evaluation**: Simple material counting:
  - Pawn: 1, Knight: 3, Bishop: 3, Rook: 5, Queen: 9, King: 999
- **Optimizations**:
  - Caching: SHA256-based result caching keyed by (board_state, timeout)
  - Opening book: Hardcoded responses for common opening positions
- **Command-line options**:
  - `--timeout <seconds>` (default: 5): Search time limit
  - `--print-only`: Display board without calculating next move

## Key Implementation Details

- **Complete legal move generation**: Includes check/checkmate detection, castling validation, en passant
- **Modular architecture**: Separated into `board.rs`, `cache.rs`, `opening.rs`, and `main.rs`
- **No external dependencies** for core chess logic (only serde for serialization)
- **Japanese comments** throughout the codebase

## Recent Changes (2025-10-17)

- Removed `--depth` option in favor of timeout-based iterative deepening
- Simplified function names (removed `_with_timeout` suffixes)
- Unified caching strategy around timeout values
- Cleaned up unused depth-based methods