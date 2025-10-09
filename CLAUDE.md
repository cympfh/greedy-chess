# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a chess AI implementation in Rust that uses a simple greedy algorithm with min-max search. The program reads chess moves from stdin in various notations (SAN, UCI/LAN) and outputs the next best move.

## Architecture

### Core Components

- **Board Representation** (`src/main.rs:25-35`): 64-square array with piece tracking, castling rights, en passant, and move counters
- **Move Parsing** (`src/main.rs:217-481`): Handles multiple chess notations:
  - SAN (Standard Algebraic Notation): e.g., "Nf3", "O-O", "exd5"
  - UCI/LAN format: e.g., "e2e4", "e7e8Q"
  - Castling: "O-O" (kingside), "O-O-O" (queenside)
- **Move Generation** (`src/main.rs:380-481`): Piece-specific movement rules with basic path validation
- **Game State Management** (`src/main.rs:136-215`): Move application, castling rights, en passant tracking

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
```

## Algorithm Details

- **Search**: Min-max algorithm (depth configurable via command line)
- **Evaluation**: Simple material counting:
  - Pawn: 1, Knight: 3, Bishop: 3, Rook: 5, Queen: 9, King: 999
- **Move Selection**: Greedy approach choosing highest-value moves

## Code Quality Notes

- The codebase has some unused imports and dead code (warnings on build)
- Move validation is "naive" - focuses on basic piece movement rules
- No comprehensive legal move validation (check/checkmate detection)
- Japanese comments throughout the codebase