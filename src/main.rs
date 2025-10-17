mod board;
mod cache;
mod opening;

use board::Board;
use cache::Cache;
use opening::OpeningBook;
use clap::Parser;
use std::io::{self, Read};

/// コマンドライン引数
#[derive(Parser, Debug)]
#[command(author, version, about = "チェスAI - 標準入力から棋譜を読み込み、次の最善手を出力する", long_about = None)]
struct Args {
    /// 探索深度（大きいほど強いが遅い）
    #[arg(short, long, default_value_t = 3)]
    depth: u32,

    /// 盤面を表示するだけで最善手を計算しない
    #[arg(short, long)]
    print_only: bool,
}

/// メイン関数
///
/// 標準入力から棋譜を読み込み、AIが次の最善手を計算して出力する
/// コマンドライン引数で探索深度を指定可能（デフォルト3）
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let depth = args.depth;

    // 標準入力から棋譜（空白区切りの手）を読み、順次適用
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let tokens: Vec<String> = buf.split_whitespace().map(|s| s.to_string()).collect();

    let mut board = Board::new();

    for (ply, tok) in tokens.iter().enumerate() {
        if tok.ends_with('.') {
            continue;
        } // "12." など無視
        if tok.starts_with('{') && tok.ends_with('}') {
            continue;
        } // コメント { ... } 簡易無視
        if tok.starts_with(';') {
            continue;
        } // セミコロ解説行を無視

        if let Err(e) = board.parse_and_play_token(tok) {
            eprintln!("Failed at ply {} on token '{}': {}", ply + 1, tok, e);
            return Err(e.into());
        }
    }

    // print_only モードなら盤面を表示して終了
    if args.print_only {
        board.print_as_comment();
        return Ok(());
    }

    // オープニングブックを初期化
    let opening_book = OpeningBook::new();

    // オープニングブックから手を検索（現在の盤面を渡す）
    let san = if let Some(opening_move) = opening_book.lookup(&board) {
        eprintln!("; Using opening book");
        opening_move
    } else {
        // キャッシュを初期化
        let cache = Cache::new();

        // 盤面をシリアライズしてキャッシュキーを生成
        let board_state = board.serialize();

        // キャッシュから結果を読み込む
        if let Some(cached_move) = cache.read(&board_state, depth) {
            eprintln!("; Using cached result");
            cached_move
        } else {
            // AIが次の一手を考える
            if let Some(best_move) = board.find_best_move(depth) {
                let san = board.move_to_san(best_move);
                // キャッシュに保存
                if let Err(e) = cache.write(&board_state, depth, &san) {
                    eprintln!("; Warning: Failed to write cache: {}", e);
                }
                san
            } else {
                eprintln!("No legal moves available");
                return Err("No legal moves".into());
            }
        }
    };

    // 最善手を出力
    println!("{}", san);

    // 最善手を適用して盤面を表示
    if let Ok(()) = board.parse_and_play_token(&san) {
        println!(";");
        board.print_as_comment();
    }

    Ok(())
}
