use crate::board::Board;
use std::collections::HashMap;

/// オープニングブック
/// 各局面（盤面のシリアライズ文字列）に対して推奨される次の手を保持
pub struct OpeningBook {
    book: HashMap<String, Vec<String>>,
}

impl OpeningBook {
    /// オープニングブックを初期化
    ///
    /// 各オープニングを実際に盤面に適用して、serialize() した値をキーとする
    pub fn new() -> Self {
        let mut book = HashMap::new();

        // オープニングのライン（各ラインは手順の配列）を定義
        let opening_lines: Vec<Vec<&str>> = vec![
            // ==================== Italian Game ====================
            vec!["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "c3", "Nf6", "d4"],
            vec!["e4", "e5", "Nf3", "Nc6", "Bc4", "Nf6", "d3"],
            // ==================== Ruy Lopez ====================
            vec!["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "Nf6", "O-O", "Be7"],
            // ==================== Petrov's Defense ====================
            vec!["e4", "e5", "Nf3", "Nf6", "Nxe5", "d6", "Nf3"],
            // ==================== Sicilian Defense ====================
            vec!["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6"],
            vec!["e4", "c5", "Nf3", "Nc6", "d4", "cxd4", "Nxd4"],
            // ==================== French Defense ====================
            vec!["e4", "e6", "d4", "d5", "Nc3", "Nf6"],
            vec!["e4", "e6", "d4", "d5", "e5", "c5"],
            // ==================== Caro-Kann Defense ====================
            vec!["e4", "c6", "d4", "d5", "Nc3"],
            // ==================== Queen's Gambit ====================
            vec!["d4", "d5", "c4", "e6", "Nc3", "Nf6"],
            vec!["d4", "d5", "c4", "c6", "Nf3"],
            vec!["d4", "d5", "c4", "dxc4", "Nf3"],
            // ==================== King's Indian Defense ====================
            vec!["d4", "Nf6", "c4", "g6", "Nc3", "Bg7", "e4"],
            // ==================== English Opening ====================
            vec!["c4", "e5", "Nc3"],
            vec!["c4", "Nf6", "Nc3"],
            vec!["c4", "c5", "Nf3"],
        ];

        // 各ラインを展開してブックに登録
        for line in opening_lines {
            // 各ラインの n 手目まで適用して、n+1 手目を推奨手として登録
            for n in 0..line.len() {
                let mut board = Board::new();

                // n 手目まで適用
                for i in 0..n {
                    if board.parse_and_play_token(line[i]).is_err() {
                        eprintln!("Warning: Failed to parse opening move: {}", line[i]);
                        break;
                    }
                }

                // 盤面をシリアライズしてキーとする
                let key = board.serialize();
                let recommended = line[n];

                // 既存のエントリに追加、または新規作成
                book.entry(key)
                    .or_insert_with(Vec::new)
                    .push(recommended.to_string());
            }
        }

        OpeningBook { book }
    }

    /// 現在の盤面に対する推奨手を取得
    ///
    /// # Arguments
    /// * `board` - 現在の盤面状態
    ///
    /// # Returns
    /// オープニングブックに登録されている推奨手（複数候補がある場合は最初の手を返す）
    pub fn lookup(&self, board: &Board) -> Option<String> {
        let key = board.serialize();
        self.book
            .get(&key)
            .and_then(|candidates| candidates.first().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opening_book_basic() {
        let book = OpeningBook::new();

        // 初手
        let board = Board::new();
        let result = book.lookup(&board);
        assert!(result.is_some());
        let recommended = result.unwrap();
        assert!(recommended == "e4" || recommended == "d4" || recommended == "c4");

        // イタリアンゲーム
        let mut board = Board::new();
        for m in &["e4", "e5", "Nf3", "Nc6"] {
            board.parse_and_play_token(m).unwrap();
        }
        let result = book.lookup(&board);
        assert!(result.is_some());
        let recommended = result.unwrap();
        assert!(recommended == "Bc4" || recommended == "Bb5");

        // シチリアン
        let mut board = Board::new();
        for m in &["e4", "c5"] {
            board.parse_and_play_token(m).unwrap();
        }
        assert_eq!(book.lookup(&board), Some("Nf3".to_string()));
    }

    #[test]
    fn test_opening_book_not_found() {
        let book = OpeningBook::new();

        // オープニングブックに無い手順
        let mut board = Board::new();
        for m in &["a4", "h5"] {
            board.parse_and_play_token(m).unwrap();
        }
        assert_eq!(book.lookup(&board), None);
    }
}
