/// 盤面評価モジュール
///
/// Piece-Square Tablesと追加ボーナスを使用して局面を評価する
use crate::board::{Board, Color, Kind, Piece};
use std::sync::atomic::{AtomicU8, Ordering};

/// 評価関数の種類を保持するグローバル変数
static EVALUATOR_TYPE: AtomicU8 = AtomicU8::new(0); // 0 = Advanced, 1 = Classic

/// 評価関数の種類を設定する
pub fn set_evaluator_type(evaluator_type: crate::EvaluatorType) {
    match evaluator_type {
        crate::EvaluatorType::Advanced => EVALUATOR_TYPE.store(0, Ordering::Relaxed),
        crate::EvaluatorType::Classic => EVALUATOR_TYPE.store(1, Ordering::Relaxed),
    }
}

/// 駒の基本価値（センチポーン単位: 1ポーン=100）
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20000;

/// Piece-Square Tables: ポーン
/// 白視点での評価値（黒の場合は上下反転）
const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Piece-Square Tables: ナイト
const KNIGHT_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

/// Piece-Square Tables: ビショップ
const BISHOP_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

/// Piece-Square Tables: ルーク
const ROOK_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];

/// Piece-Square Tables: クイーン
const QUEEN_TABLE: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

/// Piece-Square Tables: キング（中盤）
/// キャスリングを推奨する配置
const KING_MIDDLEGAME_TABLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

/// Piece-Square Tables: キング（終盤）
/// キングを中央に活性化させる
const KING_ENDGAME_TABLE: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
];

/// ボーナス: ビショップペア
const BISHOP_PAIR_BONUS: i32 = 50;

/// ボーナス: キャスリング権
const CASTLING_RIGHTS_BONUS: i32 = 15;

/// クラシック評価関数: 駒の価値の合計のみで評価
///
/// 以前の単純な評価関数。位置評価を行わず、駒の価値のみで評価する。
///
/// # 引数
/// * `board` - 評価する盤面
///
/// # 戻り値
/// 評価値（白から見て正の値が有利、負の値が不利）
pub fn evaluate_classic(board: &Board) -> i32 {
    let mut score = 0;

    for i in 0..64 {
        if let Some(piece) = board.piece_at(i) {
            let value = match piece.kind {
                Kind::Pawn => 1,
                Kind::Knight => 3,
                Kind::Bishop => 3,
                Kind::Rook => 5,
                Kind::Queen => 9,
                Kind::King => 999,
            };

            match piece.color {
                Color::White => score += value,
                Color::Black => score -= value,
            }
        }
    }

    score
}

/// 盤面を評価する（デフォルト: Piece-Square Tables使用）
///
/// グローバル設定に基づいて適切な評価関数を使用する
///
/// # 引数
/// * `board` - 評価する盤面
///
/// # 戻り値
/// 評価値（白から見て正の値が有利、負の値が不利）
pub fn evaluate(board: &Board) -> i32 {
    match EVALUATOR_TYPE.load(Ordering::Relaxed) {
        1 => evaluate_classic(board),
        _ => evaluate_advanced(board),
    }
}

/// 高度な評価関数: Piece-Square Tables使用
///
/// # 引数
/// * `board` - 評価する盤面
///
/// # 戻り値
/// 評価値（白から見て正の値が有利、負の値が不利）
fn evaluate_advanced(board: &Board) -> i32 {
    let mut score = 0;

    // 駒の数をカウント（終盤判定用）
    let mut piece_count = 0;
    let mut white_bishops = 0;
    let mut black_bishops = 0;

    // 各マスの駒を評価
    for i in 0..64 {
        if let Some(piece) = board.piece_at(i) {
            piece_count += 1;

            // 駒の基本価値
            let material_value = get_piece_value(piece.kind);

            // 位置評価
            let positional_value = get_positional_value(piece, i, piece_count <= 14);

            let total_value = material_value + positional_value;

            match piece.color {
                Color::White => {
                    score += total_value;
                    if piece.kind == Kind::Bishop {
                        white_bishops += 1;
                    }
                }
                Color::Black => {
                    score -= total_value;
                    if piece.kind == Kind::Bishop {
                        black_bishops += 1;
                    }
                }
            }
        }
    }

    // ビショップペアボーナス
    if white_bishops >= 2 {
        score += BISHOP_PAIR_BONUS;
    }
    if black_bishops >= 2 {
        score -= BISHOP_PAIR_BONUS;
    }

    // キャスリング権ボーナス
    if board.castle_wk() || board.castle_wq() {
        score += CASTLING_RIGHTS_BONUS;
    }
    if board.castle_bk() || board.castle_bq() {
        score -= CASTLING_RIGHTS_BONUS;
    }

    score
}

/// 駒の基本価値を取得する
fn get_piece_value(kind: Kind) -> i32 {
    match kind {
        Kind::Pawn => PAWN_VALUE,
        Kind::Knight => KNIGHT_VALUE,
        Kind::Bishop => BISHOP_VALUE,
        Kind::Rook => ROOK_VALUE,
        Kind::Queen => QUEEN_VALUE,
        Kind::King => KING_VALUE,
    }
}

/// 駒の位置評価を取得する
///
/// # 引数
/// * `piece` - 評価する駒
/// * `square` - 盤面上の位置（0-63）
/// * `is_endgame` - 終盤かどうか（駒数16個以下）
fn get_positional_value(piece: Piece, square: usize, is_endgame: bool) -> i32 {
    // 黒の駒の場合は盤面を上下反転
    let index = if piece.color == Color::White {
        square
    } else {
        square ^ 56 // 上下反転: rank を反転
    };

    match piece.kind {
        Kind::Pawn => PAWN_TABLE[index],
        Kind::Knight => KNIGHT_TABLE[index],
        Kind::Bishop => BISHOP_TABLE[index],
        Kind::Rook => ROOK_TABLE[index],
        Kind::Queen => QUEEN_TABLE[index],
        Kind::King => {
            if is_endgame {
                KING_ENDGAME_TABLE[index]
            } else {
                KING_MIDDLEGAME_TABLE[index]
            }
        }
    }
}
