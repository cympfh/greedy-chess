use serde_json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// キャッシュ管理
pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    /// キャッシュを初期化する
    pub fn new() -> Self {
        Cache {
            cache_dir: PathBuf::from("/tmp/chess.cache"),
        }
    }

    /// 盤面状態と深度からキャッシュキー（ハッシュ値）を生成する
    fn generate_key(&self, board_state: &str, depth: u32) -> String {
        let mut hasher = Sha256::new();
        hasher.update(board_state.as_bytes());
        hasher.update(depth.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// キャッシュファイルのパスを取得する
    fn get_path(&self, cache_key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", cache_key))
    }

    /// キャッシュから最善手を読み込む
    ///
    /// # 引数
    /// * `board_state` - 正規化された盤面状態の文字列
    /// * `depth` - 探索深度
    pub fn read(&self, board_state: &str, depth: u32) -> Option<String> {
        let key = self.generate_key(board_state, depth);
        let path = self.get_path(&key);

        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let result: serde_json::Value = serde_json::from_str(&content).ok()?;
        result.get("best_move")?.as_str().map(|s| s.to_string())
    }

    /// キャッシュに最善手を書き込む
    ///
    /// # 引数
    /// * `board_state` - 正規化された盤面状態の文字列
    /// * `depth` - 探索深度
    /// * `best_move` - 最善手（SAN形式）
    pub fn write(&self, board_state: &str, depth: u32, best_move: &str) -> std::io::Result<()> {
        fs::create_dir_all(&self.cache_dir)?;

        let key = self.generate_key(board_state, depth);
        let path = self.get_path(&key);

        let result = serde_json::json!({
            "best_move": best_move,
        });

        let content = serde_json::to_string_pretty(&result)?;
        fs::write(path, content)
    }
}
