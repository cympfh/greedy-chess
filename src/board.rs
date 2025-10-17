use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Kind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Piece {
    kind: Kind,
    color: Color,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move {
    from: usize, // 0..63
    to: usize,   // 0..63
    promo: Option<Kind>,
    is_capture: bool,
    is_en_passant: bool,
    is_castle_kingside: bool,
    is_castle_queenside: bool,
}

#[derive(Clone)]
pub struct Board {
    sq: [Option<Piece>; 64],
    side: Color,
    castle_wk: bool,
    castle_wq: bool,
    castle_bk: bool,
    castle_bq: bool,
    ep_square: Option<usize>, // アンパッサン可能な取り先
    halfmove_clock: u32,
    fullmove_number: u32,
}

/// ファイルとランクから盤面インデックス（0..63）を計算する
///
/// # 引数
/// * `file` - ファイル番号 (0..7, a..h に対応)
/// * `rank` - ランク番号 (0..7, 1段目..8段目に対応)
fn idx(file: usize, rank: usize) -> usize {
    rank * 8 + file
}

/// 盤面インデックスからファイル番号を取得する
///
/// # 引数
/// * `i` - 盤面インデックス (0..63)
fn file_of(i: usize) -> usize {
    i % 8
}
/// 盤面インデックスからランク番号を取得する
///
/// # 引数
/// * `i` - 盤面インデックス (0..63)
fn rank_of(i: usize) -> usize {
    i / 8
}

/// ファイルとランクが盤面の範囲内かチェックする
///
/// # 引数
/// * `file` - ファイル番号 (符号付き整数)
/// * `rank` - ランク番号 (符号付き整数)
fn in_bounds(file: isize, rank: isize) -> bool {
    (0..8).contains(&file) && (0..8).contains(&rank)
}
/// 符号付きファイルとランクから盤面インデックスを計算する
///
/// # 引数
/// * `file` - ファイル番号 (符号付き整数)
/// * `rank` - ランク番号 (符号付き整数)
fn to_idx(file: isize, rank: isize) -> usize {
    (rank as usize) * 8 + (file as usize)
}

impl Board {
    /// チェスの初期配置で盤面を作成する
    pub fn new() -> Self {
        use Color::*;
        use Kind::*;
        let mut sq = [None; 64];

        // 白
        let back_white = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for f in 0..8 {
            sq[idx(f, 0)] = Some(Piece {
                kind: back_white[f],
                color: White,
            });
            sq[idx(f, 1)] = Some(Piece {
                kind: Pawn,
                color: White,
            });
        }
        // 黒
        let back_black = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for f in 0..8 {
            sq[idx(f, 7)] = Some(Piece {
                kind: back_black[f],
                color: Black,
            });
            sq[idx(f, 6)] = Some(Piece {
                kind: Pawn,
                color: Black,
            });
        }

        Board {
            sq,
            side: White,
            castle_wk: true,
            castle_wq: true,
            castle_bk: true,
            castle_bq: true,
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// 指定された位置の駒を取得する
    ///
    /// # 引数
    /// * `i` - 盤面インデックス (0..63)
    fn piece_at(&self, i: usize) -> Option<Piece> {
        self.sq[i]
    }
    /// 指定された位置に駒を配置する
    ///
    /// # 引数
    /// * `i` - 盤面インデックス (0..63)
    /// * `p` - 配置する駒（Noneの場合は駒を取り除く）
    fn set_piece(&mut self, i: usize, p: Option<Piece>) {
        self.sq[i] = p;
    }

    /// 盤面をコメント形式で標準出力に表示する
    pub fn print_as_comment(&self) {
        println!(";");
        for r in (0..8).rev() {
            print!("; {} ", r + 1);
            for f in 0..8 {
                let i = idx(f, r);
                match self.sq[i] {
                    None => print!(". "),
                    Some(p) => {
                        let c = match (p.kind, p.color) {
                            (Kind::Pawn, Color::White) => 'P',
                            (Kind::Knight, Color::White) => 'N',
                            (Kind::Bishop, Color::White) => 'B',
                            (Kind::Rook, Color::White) => 'R',
                            (Kind::Queen, Color::White) => 'Q',
                            (Kind::King, Color::White) => 'K',
                            (Kind::Pawn, Color::Black) => 'p',
                            (Kind::Knight, Color::Black) => 'n',
                            (Kind::Bishop, Color::Black) => 'b',
                            (Kind::Rook, Color::Black) => 'r',
                            (Kind::Queen, Color::Black) => 'q',
                            (Kind::King, Color::Black) => 'k',
                        };
                        print!("{} ", c);
                    }
                }
            }
            println!();
        }
        println!(";   a b c d e f g h");
        println!("; Side to move: {:?}", self.side);
        println!(
            "; Castling: {}{}{}{}",
            if self.castle_wk { 'K' } else { '-' },
            if self.castle_wq { 'Q' } else { '-' },
            if self.castle_bk { 'k' } else { '-' },
            if self.castle_bq { 'q' } else { '-' }
        );
        if let Some(ep) = self.ep_square {
            let f = (file_of(ep) as u8 + b'a') as char;
            let r = (rank_of(ep) + 1).to_string();
            println!("; En passant: {}{}", f, r);
        } else {
            println!("; En passant: -");
        }
        println!(";");
    }

    /// 手番の色を反転する
    ///
    /// # 引数
    /// * `c` - 現在の色
    fn other(c: Color) -> Color {
        if let Color::White = c {
            Color::Black
        } else {
            Color::White
        }
    }

    /// 指し手を盤面に適用する
    ///
    /// キャスリング、アンパッサン、昇格などの特殊な手も処理し、
    /// キャスリング権やアンパッサン権、手数カウントを更新する
    ///
    /// # 引数
    /// * `m` - 適用する指し手
    fn make_move(&mut self, m: Move) {
        // 基本適用（最低限）
        let mut moved = self.piece_at(m.from).expect("No piece on from");
        // アンパッサン
        if m.is_en_passant {
            self.set_piece(m.to, Some(moved));
            self.set_piece(m.from, None);
            // 取られるポーン
            let to_rank = rank_of(m.to) as isize;
            let dir = if moved.color == Color::White { -1 } else { 1 };
            let cap_sq = to_idx(file_of(m.to) as isize, to_rank + dir);
            self.set_piece(cap_sq, None);
        } else if m.is_castle_kingside || m.is_castle_queenside {
            // キャスリング
            let (k_from, k_to, r_from, r_to) = if moved.color == Color::White {
                if m.is_castle_kingside {
                    (idx(4, 0), idx(6, 0), idx(7, 0), idx(5, 0))
                } else {
                    (idx(4, 0), idx(2, 0), idx(0, 0), idx(3, 0))
                }
            } else {
                if m.is_castle_kingside {
                    (idx(4, 7), idx(6, 7), idx(7, 7), idx(5, 7))
                } else {
                    (idx(4, 7), idx(2, 7), idx(0, 7), idx(3, 7))
                }
            };
            // king
            let king = self.piece_at(k_from).unwrap();
            self.set_piece(k_from, None);
            self.set_piece(k_to, Some(king));
            // rook
            let rook = self.piece_at(r_from).unwrap();
            self.set_piece(r_from, None);
            self.set_piece(r_to, Some(rook));
        } else {
            // 通常
            self.set_piece(m.to, Some(moved));
            self.set_piece(m.from, None);
        }

        // 昇格
        if let Some(pk) = m.promo {
            moved.kind = pk;
            self.set_piece(m.to, Some(moved));
        }

        // キャスリング権の更新（キング/ルークが動いたら消す）
        match moved.color {
            Color::White => {
                // 白キング・白ルークの移動/捕獲で権利を消す
                if m.from == idx(4, 0) {
                    self.castle_wk = false;
                    self.castle_wq = false;
                }
                if m.from == idx(0, 0) || m.to == idx(0, 0) {
                    self.castle_wq = false;
                }
                if m.from == idx(7, 0) || m.to == idx(7, 0) {
                    self.castle_wk = false;
                }
                // 黒ルークが取られたら黒権利調整
                if m.to == idx(0, 7) {
                    self.castle_bq = false;
                }
                if m.to == idx(7, 7) {
                    self.castle_bk = false;
                }
            }
            Color::Black => {
                if m.from == idx(4, 7) {
                    self.castle_bk = false;
                    self.castle_bq = false;
                }
                if m.from == idx(0, 7) || m.to == idx(0, 7) {
                    self.castle_bq = false;
                }
                if m.from == idx(7, 7) || m.to == idx(7, 7) {
                    self.castle_bk = false;
                }
                if m.to == idx(0, 0) {
                    self.castle_wq = false;
                }
                if m.to == idx(7, 0) {
                    self.castle_wk = false;
                }
            }
        }

        // EP 権
        self.ep_square = None;
        if moved.kind == Kind::Pawn {
            let r_from = rank_of(m.from) as isize;
            let r_to = rank_of(m.to) as isize;
            if (r_from - r_to).abs() == 2 {
                // 通過マス
                let mid = to_idx(file_of(m.from) as isize, (r_from + r_to) / 2);
                self.ep_square = Some(mid);
            }
        }

        // 手数カウント（50手ルール用の半手）: ここでは参考値として動かすだけ
        if moved.kind == Kind::Pawn || m.is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.side == Color::Black {
            self.fullmove_number += 1;
        }
        self.side = Board::other(self.side);
    }

    // ============ ここから指し手解釈（UCI/LAN 先、SAN 簡易後） ============

    /// トークン（指し手の文字列）を解析して盤面に適用する
    ///
    /// SAN形式、UCI/LAN形式、キャスリング記号などに対応
    ///
    /// # 引数
    /// * `token` - 指し手を表す文字列
    pub fn parse_and_play_token(&mut self, token: &str) -> Result<(), String> {
        let t = token.trim();
        if t.is_empty() {
            return Ok(());
        }

        // 結果記号や注釈はスキップ
        if t == "1-0" || t == "0-1" || t == "1/2-1/2" || t == "*" {
            return Ok(());
        }
        if t.ends_with('.') {
            return Ok(());
        } // move number like "12."

        // キャスリング（SAN）
        if t == "O-O" || t == "0-0" {
            let mv = self.build_castle(true)?;
            self.make_move(mv);
            return Ok(());
        }
        if t == "O-O-O" || t == "0-0-0" {
            let mv = self.build_castle(false)?;
            self.make_move(mv);
            return Ok(());
        }

        // UCI/LAN: e2e4, e7e8Q
        if let Some(mv) = self.try_parse_uci_like(t)? {
            self.make_move(mv);
            return Ok(());
        }

        // SAN 簡易
        let mv = self.parse_san_and_find_move(t)?;
        self.make_move(mv);
        Ok(())
    }

    /// キャスリングの手を構築する
    ///
    /// # 引数
    /// * `kingside` - true ならキングサイド、false ならクイーンサイド
    fn build_castle(&self, kingside: bool) -> Result<Move, String> {
        let (from, to) = match self.side {
            Color::White => {
                if kingside {
                    (idx(4, 0), idx(6, 0))
                } else {
                    (idx(4, 0), idx(2, 0))
                }
            }
            Color::Black => {
                if kingside {
                    (idx(4, 7), idx(6, 7))
                } else {
                    (idx(4, 7), idx(2, 7))
                }
            }
        };
        Ok(Move {
            from,
            to,
            promo: None,
            is_capture: false,
            is_en_passant: false,
            is_castle_kingside: kingside,
            is_castle_queenside: !kingside,
        })
    }

    /// UCI/LAN形式の指し手の解析を試みる
    ///
    /// "e2e4", "e7e8Q" などの形式を解析する
    ///
    /// # 引数
    /// * `t` - 解析する文字列
    ///
    /// # 戻り値
    /// UCI形式と判定できた場合はSome(Move)、そうでなければNone
    fn try_parse_uci_like(&self, t: &str) -> Result<Option<Move>, String> {
        // 例: "e2e4", "e7e8Q"
        // a-h, 1-8, a-h, 1-8, [NBRQ]
        if t.len() < 4 {
            return Ok(None);
        }
        let b = t.as_bytes();
        let is_square = |f: u8, r: u8| (b'a'..=b'h').contains(&f) && (b'1'..=b'8').contains(&r);
        if !is_square(b[0], b[1]) || !is_square(b[2], b[3]) {
            return Ok(None);
        }
        let from = idx((b[0] - b'a') as usize, (b[1] - b'1') as usize);
        let to = idx((b[2] - b'a') as usize, (b[3] - b'1') as usize);

        let promo = if t.len() >= 5 {
            match t.as_bytes()[4] as char {
                'q' | 'Q' => Some(Kind::Queen),
                'r' | 'R' => Some(Kind::Rook),
                'b' | 'B' => Some(Kind::Bishop),
                'n' | 'N' => Some(Kind::Knight),
                _ => None,
            }
        } else {
            None
        };

        // 捕獲かどうかと EP をざっくり判断
        let mut is_capture = self.piece_at(to).is_some();
        let mut is_ep = false;
        if let Some(p) = self.piece_at(from) {
            if p.kind == Kind::Pawn && !is_capture && Some(to) == self.ep_square {
                is_ep = true;
                is_capture = true;
            }
        }

        Ok(Some(Move {
            from,
            to,
            promo,
            is_capture,
            is_en_passant: is_ep,
            is_castle_kingside: false,
            is_castle_queenside: false,
        }))
    }

    /// SAN形式の指し手を解析して対応する手を見つける
    ///
    /// "Nf3", "exd5", "O-O" などの標準代数記法を解析し、
    /// 曖昧性解消を行って正しい手を特定する
    ///
    /// # 引数
    /// * `t` - SAN形式の指し手文字列
    fn parse_san_and_find_move(&self, t: &str) -> Result<Move, String> {
        // SAN の記号除去（+, #, !? など）
        let mut s = t.replace("+", "").replace("#", "");
        s = s.trim_end_matches(['!', '?'].as_ref()).to_string();

        // 昇格表記 e8=Q
        let mut promo: Option<Kind> = None;
        if let Some(eq) = s.find('=') {
            let p = s.as_bytes()[eq + 1] as char;
            promo = match p {
                'Q' => Some(Kind::Queen),
                'R' => Some(Kind::Rook),
                'B' => Some(Kind::Bishop),
                'N' => Some(Kind::Knight),
                _ => None,
            };
            s.truncate(eq);
        }

        // 取り "x" の有無
        let is_capture = s.contains('x');
        let s_clean = s.replace("x", "");

        // 駒種
        let (kind, rest) = match s_clean.chars().next().unwrap_or(' ') {
            'K' => (Kind::King, &s_clean[1..]),
            'Q' => (Kind::Queen, &s_clean[1..]),
            'R' => (Kind::Rook, &s_clean[1..]),
            'B' => (Kind::Bishop, &s_clean[1..]),
            'N' => (Kind::Knight, &s_clean[1..]),
            'O' | '0' => return Err("Use O-O / O-O-O handled earlier".into()),
            _ => (Kind::Pawn, &s_clean[..]),
        };

        // 残りは [disambiguation?] + destination square
        // 末尾2文字が目的地（例: e4）。その前に 0〜2 文字の曖昧性解消（例: "Nbd7", "R1e2", "Qhxe5"→clean後 "Qhe5"）
        if rest.len() < 2 {
            return Err(format!("SAN too short: {}", t));
        }
        let dest_part = &rest[rest.len() - 2..];
        let to = parse_square(dest_part)?;

        let mut dis_file: Option<usize> = None;
        let mut dis_rank: Option<usize> = None;
        let dis = &rest[..rest.len() - 2];
        for ch in dis.chars() {
            if ('a'..='h').contains(&ch) {
                dis_file = Some((ch as u8 - b'a') as usize);
            } else if ('1'..='8').contains(&ch) {
                dis_rank = Some((ch as u8 - b'1') as usize);
            }
        }

        // 目的地に到達できる自軍の候補駒から1つ選ぶ
        let candidates = self.generate_reach(kind, to, is_capture);
        let filtered: Vec<usize> = candidates
            .into_iter()
            .filter(|&sq_from| {
                if let Some(df) = dis_file {
                    if file_of(sq_from) != df {
                        return false;
                    }
                }
                if let Some(dr) = dis_rank {
                    if rank_of(sq_from) != dr {
                        return false;
                    }
                }
                true
            })
            .collect();

        if filtered.is_empty() {
            return Err(format!("No legal (naive) source found for SAN '{}'", t));
        }
        if filtered.len() > 1 {
            // ここでは「素直に」：複数ならエラーにして曖昧性を知らせる
            return Err(format!("Ambiguous SAN '{}': need more disambiguation", t));
        }
        let from = filtered[0];

        // EP の判定（ポーンの斜め進行で相手駒がいない＆ep_square=to）
        let mut is_ep = false;
        if kind == Kind::Pawn
            && is_capture
            && self.piece_at(to).is_none()
            && Some(to) == self.ep_square
        {
            is_ep = true;
        }

        Ok(Move {
            from,
            to,
            promo,
            is_capture,
            is_en_passant: is_ep,
            is_castle_kingside: false,
            is_castle_queenside: false,
        })
    }

    /// 指定された駒種が指定された位置に到達できる元位置の候補を列挙する
    ///
    /// # 引数
    /// * `kind` - 駒の種類
    /// * `to` - 目的地の盤面インデックス
    /// * `is_capture` - 捕獲の手かどうか
    fn generate_reach(&self, kind: Kind, to: usize, is_capture: bool) -> Vec<usize> {
        // 「素直」な到達元探索：現在手番の自軍で、指定種が to に動ける from 候補（最小限のルール）
        let mut v = Vec::new();
        for i in 0..64 {
            if let Some(p) = self.piece_at(i) {
                if p.color != self.side || p.kind != kind {
                    continue;
                }
                if self.naive_can_reach(i, to, is_capture) {
                    v.push(i);
                }
            }
        }
        v
    }

    /// 現在の局面における全ての合法手を生成する
    ///
    /// 自玉がチェックに晒される手は除外される
    fn generate_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        for from in 0..64 {
            if let Some(piece) = self.piece_at(from) {
                if piece.color != self.side {
                    continue;
                }

                match piece.kind {
                    Kind::Pawn => self.generate_pawn_moves(from, &mut moves),
                    Kind::Knight => self.generate_knight_moves(from, &mut moves),
                    Kind::Bishop => self.generate_bishop_moves(from, &mut moves),
                    Kind::Rook => self.generate_rook_moves(from, &mut moves),
                    Kind::Queen => self.generate_queen_moves(from, &mut moves),
                    Kind::King => self.generate_king_moves(from, &mut moves),
                }
            }
        }

        // チェックに晒す手を除外
        moves
            .into_iter()
            .filter(|&m| self.is_legal_move(m))
            .collect()
    }

    /// ポーンの合法手候補を生成する
    ///
    /// 前進、2マス前進、斜め取り、アンパッサン、昇格を含む
    ///
    /// # 引数
    /// * `from` - ポーンの位置
    /// * `moves` - 生成した手を追加するベクタ
    fn generate_pawn_moves(&self, from: usize, moves: &mut Vec<Move>) {
        let piece = self.piece_at(from).unwrap();
        let forward = if piece.color == Color::White { 1 } else { -1 };
        let start_rank = if piece.color == Color::White { 1 } else { 6 };
        let promo_rank = if piece.color == Color::White { 7 } else { 0 };

        let f = file_of(from) as isize;
        let r = rank_of(from) as isize;

        // 前進1マス
        let to_r = r + forward;
        if in_bounds(f, to_r) {
            let to = to_idx(f, to_r);
            if self.piece_at(to).is_none() {
                if rank_of(to) == promo_rank {
                    // 昇格
                    for &promo_kind in &[Kind::Queen, Kind::Rook, Kind::Bishop, Kind::Knight] {
                        moves.push(Move {
                            from,
                            to,
                            promo: Some(promo_kind),
                            is_capture: false,
                            is_en_passant: false,
                            is_castle_kingside: false,
                            is_castle_queenside: false,
                        });
                    }
                } else {
                    moves.push(Move {
                        from,
                        to,
                        promo: None,
                        is_capture: false,
                        is_en_passant: false,
                        is_castle_kingside: false,
                        is_castle_queenside: false,
                    });

                    // 初期位置からの2マス前進
                    if rank_of(from) == start_rank {
                        let to2 = to_idx(f, to_r + forward);
                        if in_bounds(f, to_r + forward) && self.piece_at(to2).is_none() {
                            moves.push(Move {
                                from,
                                to: to2,
                                promo: None,
                                is_capture: false,
                                is_en_passant: false,
                                is_castle_kingside: false,
                                is_castle_queenside: false,
                            });
                        }
                    }
                }
            }
        }

        // 斜め取り
        for &df in &[-1, 1] {
            if in_bounds(f + df, to_r) {
                let to = to_idx(f + df, to_r);
                let is_capture = self.piece_at(to).is_some();
                let is_ep = !is_capture && Some(to) == self.ep_square;

                if is_capture || is_ep {
                    if rank_of(to) == promo_rank {
                        // 昇格取り
                        for &promo_kind in &[Kind::Queen, Kind::Rook, Kind::Bishop, Kind::Knight] {
                            moves.push(Move {
                                from,
                                to,
                                promo: Some(promo_kind),
                                is_capture: true,
                                is_en_passant: is_ep,
                                is_castle_kingside: false,
                                is_castle_queenside: false,
                            });
                        }
                    } else {
                        moves.push(Move {
                            from,
                            to,
                            promo: None,
                            is_capture: true,
                            is_en_passant: is_ep,
                            is_castle_kingside: false,
                            is_castle_queenside: false,
                        });
                    }
                }
            }
        }
    }

    /// ナイトの合法手候補を生成する
    ///
    /// # 引数
    /// * `from` - ナイトの位置
    /// * `moves` - 生成した手を追加するベクタ
    fn generate_knight_moves(&self, from: usize, moves: &mut Vec<Move>) {
        let f = file_of(from) as isize;
        let r = rank_of(from) as isize;

        let knight_moves = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];

        for &(df, dr) in &knight_moves {
            if in_bounds(f + df, r + dr) {
                let to = to_idx(f + df, r + dr);
                if let Some(target) = self.piece_at(to) {
                    if target.color != self.side {
                        moves.push(Move {
                            from,
                            to,
                            promo: None,
                            is_capture: true,
                            is_en_passant: false,
                            is_castle_kingside: false,
                            is_castle_queenside: false,
                        });
                    }
                } else {
                    moves.push(Move {
                        from,
                        to,
                        promo: None,
                        is_capture: false,
                        is_en_passant: false,
                        is_castle_kingside: false,
                        is_castle_queenside: false,
                    });
                }
            }
        }
    }

    /// 長距離駒（ビショップ、ルーク、クイーン）の合法手候補を生成する
    ///
    /// # 引数
    /// * `from` - 駒の位置
    /// * `directions` - 移動方向のリスト（ファイル差、ランク差）
    /// * `moves` - 生成した手を追加するベクタ
    fn generate_sliding_moves(
        &self,
        from: usize,
        directions: &[(isize, isize)],
        moves: &mut Vec<Move>,
    ) {
        let f = file_of(from) as isize;
        let r = rank_of(from) as isize;

        for &(df, dr) in directions {
            let mut cur_f = f + df;
            let mut cur_r = r + dr;

            while in_bounds(cur_f, cur_r) {
                let to = to_idx(cur_f, cur_r);

                if let Some(target) = self.piece_at(to) {
                    if target.color != self.side {
                        moves.push(Move {
                            from,
                            to,
                            promo: None,
                            is_capture: true,
                            is_en_passant: false,
                            is_castle_kingside: false,
                            is_castle_queenside: false,
                        });
                    }
                    break; // 駒があったら止まる
                } else {
                    moves.push(Move {
                        from,
                        to,
                        promo: None,
                        is_capture: false,
                        is_en_passant: false,
                        is_castle_kingside: false,
                        is_castle_queenside: false,
                    });
                }

                cur_f += df;
                cur_r += dr;
            }
        }
    }

    /// ビショップの合法手候補を生成する
    ///
    /// # 引数
    /// * `from` - ビショップの位置
    /// * `moves` - 生成した手を追加するベクタ
    fn generate_bishop_moves(&self, from: usize, moves: &mut Vec<Move>) {
        let diagonals = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
        self.generate_sliding_moves(from, &diagonals, moves);
    }

    /// ルークの合法手候補を生成する
    ///
    /// # 引数
    /// * `from` - ルークの位置
    /// * `moves` - 生成した手を追加するベクタ
    fn generate_rook_moves(&self, from: usize, moves: &mut Vec<Move>) {
        let orthogonals = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        self.generate_sliding_moves(from, &orthogonals, moves);
    }

    /// クイーンの合法手候補を生成する
    ///
    /// # 引数
    /// * `from` - クイーンの位置
    /// * `moves` - 生成した手を追加するベクタ
    fn generate_queen_moves(&self, from: usize, moves: &mut Vec<Move>) {
        let all_directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        self.generate_sliding_moves(from, &all_directions, moves);
    }

    /// キングの合法手候補を生成する
    ///
    /// 通常の移動に加えてキャスリングも含む
    ///
    /// # 引数
    /// * `from` - キングの位置
    /// * `moves` - 生成した手を追加するベクタ
    fn generate_king_moves(&self, from: usize, moves: &mut Vec<Move>) {
        let f = file_of(from) as isize;
        let r = rank_of(from) as isize;

        // 通常の移動
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                if in_bounds(f + df, r + dr) {
                    let to = to_idx(f + df, r + dr);
                    if let Some(target) = self.piece_at(to) {
                        if target.color != self.side {
                            moves.push(Move {
                                from,
                                to,
                                promo: None,
                                is_capture: true,
                                is_en_passant: false,
                                is_castle_kingside: false,
                                is_castle_queenside: false,
                            });
                        }
                    } else {
                        moves.push(Move {
                            from,
                            to,
                            promo: None,
                            is_capture: false,
                            is_en_passant: false,
                            is_castle_kingside: false,
                            is_castle_queenside: false,
                        });
                    }
                }
            }
        }

        // キャスリング
        if self.side == Color::White {
            if self.castle_wk && self.can_castle_kingside(Color::White) {
                moves.push(Move {
                    from,
                    to: idx(6, 0),
                    promo: None,
                    is_capture: false,
                    is_en_passant: false,
                    is_castle_kingside: true,
                    is_castle_queenside: false,
                });
            }
            if self.castle_wq && self.can_castle_queenside(Color::White) {
                moves.push(Move {
                    from,
                    to: idx(2, 0),
                    promo: None,
                    is_capture: false,
                    is_en_passant: false,
                    is_castle_kingside: false,
                    is_castle_queenside: true,
                });
            }
        } else {
            if self.castle_bk && self.can_castle_kingside(Color::Black) {
                moves.push(Move {
                    from,
                    to: idx(6, 7),
                    promo: None,
                    is_capture: false,
                    is_en_passant: false,
                    is_castle_kingside: true,
                    is_castle_queenside: false,
                });
            }
            if self.castle_bq && self.can_castle_queenside(Color::Black) {
                moves.push(Move {
                    from,
                    to: idx(2, 7),
                    promo: None,
                    is_capture: false,
                    is_en_passant: false,
                    is_castle_kingside: false,
                    is_castle_queenside: true,
                });
            }
        }
    }

    /// キングサイドキャスリングが可能かチェックする
    ///
    /// 経路が空で、キングの通過マスが攻撃されていないことを確認
    ///
    /// # 引数
    /// * `color` - キャスリングする側の色
    fn can_castle_kingside(&self, color: Color) -> bool {
        let rank = if color == Color::White { 0 } else { 7 };
        // f, g マスが空で、e, f, g が攻撃されていない
        self.piece_at(idx(5, rank)).is_none()
            && self.piece_at(idx(6, rank)).is_none()
            && !self.is_square_attacked(idx(4, rank), Board::other(color))
            && !self.is_square_attacked(idx(5, rank), Board::other(color))
            && !self.is_square_attacked(idx(6, rank), Board::other(color))
    }

    /// クイーンサイドキャスリングが可能かチェックする
    ///
    /// 経路が空で、キングの通過マスが攻撃されていないことを確認
    ///
    /// # 引数
    /// * `color` - キャスリングする側の色
    fn can_castle_queenside(&self, color: Color) -> bool {
        let rank = if color == Color::White { 0 } else { 7 };
        // b, c, d マスが空で、c, d, e が攻撃されていない
        self.piece_at(idx(1, rank)).is_none()
            && self.piece_at(idx(2, rank)).is_none()
            && self.piece_at(idx(3, rank)).is_none()
            && !self.is_square_attacked(idx(2, rank), Board::other(color))
            && !self.is_square_attacked(idx(3, rank), Board::other(color))
            && !self.is_square_attacked(idx(4, rank), Board::other(color))
    }

    /// 指定されたマスが指定された色の駒に攻撃されているかチェックする
    ///
    /// # 引数
    /// * `square` - チェックするマス
    /// * `by_color` - 攻撃側の色
    fn is_square_attacked(&self, square: usize, by_color: Color) -> bool {
        // 指定されたマスが指定された色の駒に攻撃されているかチェック
        for from in 0..64 {
            if let Some(piece) = self.piece_at(from) {
                if piece.color == by_color && self.can_attack(from, square) {
                    return true;
                }
            }
        }
        false
    }

    /// 指定された位置の駒が別の位置を攻撃できるかチェックする
    ///
    /// # 引数
    /// * `from` - 攻撃元の位置
    /// * `to` - 攻撃先の位置
    fn can_attack(&self, from: usize, to: usize) -> bool {
        if from == to {
            return false;
        }
        let piece = match self.piece_at(from) {
            Some(p) => p,
            None => return false,
        };

        match piece.kind {
            Kind::Pawn => {
                let forward = if piece.color == Color::White { 1 } else { -1 };
                let f_from = file_of(from) as isize;
                let r_from = rank_of(from) as isize;
                let f_to = file_of(to) as isize;
                let r_to = rank_of(to) as isize;

                // ポーンは斜め前方のマスを攻撃
                r_to == r_from + forward && (f_to - f_from).abs() == 1
            }
            Kind::Knight => {
                let df = (file_of(from) as isize - file_of(to) as isize).abs();
                let dr = (rank_of(from) as isize - rank_of(to) as isize).abs();
                (df == 1 && dr == 2) || (df == 2 && dr == 1)
            }
            Kind::King => {
                let df = (file_of(from) as isize - file_of(to) as isize).abs();
                let dr = (rank_of(from) as isize - rank_of(to) as isize).abs();
                df <= 1 && dr <= 1
            }
            Kind::Bishop => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if df.abs() != dr.abs() || df == 0 {
                    return false;
                }
                let stepf = df.signum();
                let stepr = dr.signum();
                self.line_clear(from, to, stepf, stepr)
            }
            Kind::Rook => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if !(df == 0 || dr == 0) {
                    return false;
                }
                let stepf = df.signum();
                let stepr = dr.signum();
                if df == 0 && dr == 0 {
                    return false;
                }
                self.line_clear(from, to, stepf, stepr)
            }
            Kind::Queen => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if !(df == 0 || dr == 0 || df.abs() == dr.abs()) {
                    return false;
                }
                let stepf = df.signum();
                let stepr = dr.signum();
                self.line_clear(from, to, stepf, stepr)
            }
        }
    }

    /// 指し手が合法かどうかをチェックする
    ///
    /// 実際に手を指してみて、自玉がチェックに晒されないことを確認
    ///
    /// # 引数
    /// * `m` - チェックする手
    fn is_legal_move(&self, m: Move) -> bool {
        // 手を実際に指してみてキングがチェックに晒されないかチェック
        let current_side = self.side;
        let mut temp_board = self.clone();
        temp_board.make_move(m);
        // make_moveが手番を切り替えるので、元の手番のキングを探す
        if let Some(king_square) = temp_board.find_king(current_side) {
            !temp_board.is_square_attacked(king_square, Board::other(current_side))
        } else {
            // キングが見つからない場合は不正な手
            false
        }
    }

    /// 指定された色のキングの位置を探す
    ///
    /// # 引数
    /// * `color` - 探すキングの色
    fn find_king(&self, color: Color) -> Option<usize> {
        for i in 0..64 {
            if let Some(piece) = self.piece_at(i) {
                if piece.kind == Kind::King && piece.color == color {
                    return Some(i);
                }
            }
        }
        None
    }

    /// 局面を評価する
    ///
    /// 駒の価値の合計に基づいて評価値を計算する
    /// 白側から見て正の値が有利、負の値が不利
    fn evaluate(&self) -> i32 {
        let mut score = 0;

        for i in 0..64 {
            if let Some(piece) = self.piece_at(i) {
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

    /// チェックメイトかどうかを判定する
    ///
    /// 手番側のキングがチェックされており、合法手がない状態
    fn is_checkmate(&self) -> bool {
        if let Some(king_square) = self.find_king(self.side) {
            self.is_square_attacked(king_square, Board::other(self.side))
                && self.generate_legal_moves().is_empty()
        } else {
            false
        }
    }

    /// ステイルメイトかどうかを判定する
    ///
    /// 手番側のキングがチェックされておらず、合法手がない状態
    fn is_stalemate(&self) -> bool {
        if let Some(king_square) = self.find_king(self.side) {
            !self.is_square_attacked(king_square, Board::other(self.side))
                && self.generate_legal_moves().is_empty()
        } else {
            false
        }
    }

    /// ゲームが終了しているかを判定する
    ///
    /// チェックメイトまたはステイルメイトの場合に true
    fn is_game_over(&self) -> bool {
        self.is_checkmate() || self.is_stalemate()
    }

    /// 反復深化探索で最適な手を見つける
    ///
    /// 指定された時間内に深度を徐々に増やしながら探索を行い、
    /// タイムアウト時点での最良の手を返す
    ///
    /// # 引数
    /// * `timeout` - 探索の制限時間
    ///
    /// # 戻り値
    /// 最適手（合法手がない場合はNone）
    pub fn find_best_move(&self, timeout: Duration) -> Option<Move> {
        let moves = self.generate_legal_moves();
        if moves.is_empty() {
            return None;
        }

        let start_time = Instant::now();
        let mut best_move = moves[0];
        let mut current_depth = 1;

        loop {
            // 各深度での探索
            if let Some(result) = self.search_at_depth(current_depth, start_time, timeout) {
                best_move = result;
                eprintln!("; Completed depth {} (elapsed: {:.2}s)",
                         current_depth,
                         start_time.elapsed().as_secs_f64());
            } else {
                // タイムアウトした場合は前回の結果を返す
                eprintln!("; Timeout at depth {} (elapsed: {:.2}s)",
                         current_depth,
                         start_time.elapsed().as_secs_f64());
                break;
            }

            // タイムアウトチェック
            if start_time.elapsed() >= timeout {
                break;
            }

            current_depth += 1;
        }

        eprintln!("; Final depth reached: {}", current_depth - 1);
        Some(best_move)
    }

    /// 指定深度で最適な手を探索する
    ///
    /// # 引数
    /// * `depth` - 探索深度
    /// * `start_time` - 探索開始時刻
    /// * `timeout` - 探索の制限時間
    ///
    /// # 戻り値
    /// タイムアウト前に完了した場合は最適手、タイムアウトした場合はNone
    fn search_at_depth(&self, depth: u32, start_time: Instant, timeout: Duration) -> Option<Move> {
        let moves = self.generate_legal_moves();
        if moves.is_empty() {
            return None;
        }

        let maximizing = self.side == Color::White;
        let mut best_move = moves[0];
        let mut best_eval = if maximizing { -100001 } else { 100001 };

        for m in moves {
            // タイムアウトチェック
            if start_time.elapsed() >= timeout {
                return None;
            }

            let mut board_copy = self.clone();
            board_copy.make_move(m);
            let eval = board_copy.minimax(depth - 1, !maximizing, start_time, timeout)?;

            if maximizing && eval > best_eval {
                best_eval = eval;
                best_move = m;
            } else if !maximizing && eval < best_eval {
                best_eval = eval;
                best_move = m;
            }
        }

        Some(best_move)
    }

    /// Min-Maxアルゴリズムで局面を評価する
    ///
    /// # 引数
    /// * `depth` - 探索深度
    /// * `maximizing` - 最大化側（白）の手番かどうか
    /// * `start_time` - 探索開始時刻
    /// * `timeout` - 探索の制限時間
    ///
    /// # 戻り値
    /// タイムアウト前に完了した場合は評価値、タイムアウトした場合はNone
    fn minimax(&self, depth: u32, maximizing: bool, start_time: Instant, timeout: Duration) -> Option<i32> {
        // タイムアウトチェック
        if start_time.elapsed() >= timeout {
            return None;
        }

        if depth == 0 || self.is_game_over() {
            if self.is_checkmate() {
                return Some(if maximizing { -100000 } else { 100000 });
            }
            if self.is_stalemate() {
                return Some(0);
            }
            return Some(self.evaluate());
        }

        let moves = self.generate_legal_moves();
        if moves.is_empty() {
            return Some(if maximizing { -100000 } else { 100000 });
        }

        if maximizing {
            let mut max_eval = -100001;
            for m in moves {
                let mut board_copy = self.clone();
                board_copy.make_move(m);
                let eval = board_copy.minimax(depth - 1, false, start_time, timeout)?;
                max_eval = max_eval.max(eval);
            }
            Some(max_eval)
        } else {
            let mut min_eval = 100001;
            for m in moves {
                let mut board_copy = self.clone();
                board_copy.make_move(m);
                let eval = board_copy.minimax(depth - 1, true, start_time, timeout)?;
                min_eval = min_eval.min(eval);
            }
            Some(min_eval)
        }
    }

    /// 盤面状態を正規化された文字列に変換する
    ///
    /// キャッシュキー生成用に、盤面の完全な状態を文字列化する
    pub fn serialize(&self) -> String {
        let mut result = String::new();

        // 盤面の駒配置
        for i in 0..64 {
            match self.sq[i] {
                None => result.push('.'),
                Some(p) => {
                    let c = match (p.kind, p.color) {
                        (Kind::Pawn, Color::White) => 'P',
                        (Kind::Knight, Color::White) => 'N',
                        (Kind::Bishop, Color::White) => 'B',
                        (Kind::Rook, Color::White) => 'R',
                        (Kind::Queen, Color::White) => 'Q',
                        (Kind::King, Color::White) => 'K',
                        (Kind::Pawn, Color::Black) => 'p',
                        (Kind::Knight, Color::Black) => 'n',
                        (Kind::Bishop, Color::Black) => 'b',
                        (Kind::Rook, Color::Black) => 'r',
                        (Kind::Queen, Color::Black) => 'q',
                        (Kind::King, Color::Black) => 'k',
                    };
                    result.push(c);
                }
            }
        }

        // 手番
        result.push('|');
        result.push(match self.side {
            Color::White => 'W',
            Color::Black => 'B',
        });

        // キャスリング権
        result.push('|');
        if self.castle_wk {
            result.push('K');
        }
        if self.castle_wq {
            result.push('Q');
        }
        if self.castle_bk {
            result.push('k');
        }
        if self.castle_bq {
            result.push('q');
        }
        if !self.castle_wk && !self.castle_wq && !self.castle_bk && !self.castle_bq {
            result.push('-');
        }

        // アンパッサン
        result.push('|');
        if let Some(ep) = self.ep_square {
            let f = (file_of(ep) as u8 + b'a') as char;
            let r = (rank_of(ep) + 1).to_string();
            result.push(f);
            result.push_str(&r);
        } else {
            result.push('-');
        }

        result
    }

    /// 指し手をSAN（標準代数記法）形式の文字列に変換する
    ///
    /// # 引数
    /// * `m` - 変換する手
    pub fn move_to_san(&self, m: Move) -> String {
        if m.is_castle_kingside {
            return "O-O".to_string();
        }
        if m.is_castle_queenside {
            return "O-O-O".to_string();
        }

        let piece = self.piece_at(m.from).unwrap();
        let mut san = String::new();

        // 駒種（ポーンは省略）
        if piece.kind != Kind::Pawn {
            san.push(match piece.kind {
                Kind::King => 'K',
                Kind::Queen => 'Q',
                Kind::Rook => 'R',
                Kind::Bishop => 'B',
                Kind::Knight => 'N',
                _ => unreachable!(),
            });
        }

        // 曖昧性解消（簡単版：同じ種類の駒が複数ある場合はファイルまたはランクを追加）
        let similar_moves = self
            .generate_legal_moves()
            .into_iter()
            .filter(|&other| {
                other.to == m.to
                    && self.piece_at(other.from).map(|p| p.kind) == Some(piece.kind)
                    && other.from != m.from
            })
            .collect::<Vec<_>>();

        if !similar_moves.is_empty() {
            let same_file = similar_moves
                .iter()
                .any(|&other| file_of(other.from) == file_of(m.from));
            if !same_file {
                san.push((b'a' + file_of(m.from) as u8) as char);
            } else {
                san.push((b'1' + rank_of(m.from) as u8) as char);
            }
        }

        // 捕獲
        if m.is_capture {
            if piece.kind == Kind::Pawn && san.is_empty() {
                san.push((b'a' + file_of(m.from) as u8) as char);
            }
            san.push('x');
        }

        // 目的地
        san.push((b'a' + file_of(m.to) as u8) as char);
        san.push((b'1' + rank_of(m.to) as u8) as char);

        // 昇格
        if let Some(promo) = m.promo {
            san.push('=');
            san.push(match promo {
                Kind::Queen => 'Q',
                Kind::Rook => 'R',
                Kind::Bishop => 'B',
                Kind::Knight => 'N',
                _ => unreachable!(),
            });
        }

        san
    }

    /// 2つのマス間の経路が空かチェックする
    ///
    /// 長距離駒（ビショップ、ルーク、クイーン）の移動可否判定に使用
    ///
    /// # 引数
    /// * `from` - 開始位置
    /// * `to` - 終了位置
    /// * `df` - ファイル方向の移動量（-1, 0, 1）
    /// * `dr` - ランク方向の移動量（-1, 0, 1）
    fn line_clear(&self, from: usize, to: usize, df: isize, dr: isize) -> bool {
        // from から to へ (df,dr) 方向に一直線で、途中が空か
        let mut f = file_of(from) as isize + df;
        let mut r = rank_of(from) as isize + dr;
        while in_bounds(f, r) {
            let cur = to_idx(f, r);
            if cur == to {
                return true;
            }
            if self.piece_at(cur).is_some() {
                return false;
            }
            f += df;
            r += dr;
        }
        false
    }

    /// 駒が指定された位置に到達できるか素朴にチェックする
    ///
    /// 合法手判定は行わず、駒の動きのルールのみをチェック
    ///
    /// # 引数
    /// * `from` - 駒の位置
    /// * `to` - 目的地
    /// * `is_capture` - 捕獲の手かどうか
    fn naive_can_reach(&self, from: usize, to: usize, is_capture: bool) -> bool {
        if from == to {
            return false;
        }
        let me = match self.piece_at(from) {
            Some(p) => p,
            None => return false,
        };
        if let Some(dst) = self.piece_at(to) {
            if dst.color == me.color {
                return false;
            }
        }
        match me.kind {
            Kind::Knight => {
                let df = (file_of(from) as isize - file_of(to) as isize).abs();
                let dr = (rank_of(from) as isize - rank_of(to) as isize).abs();
                (df == 1 && dr == 2) || (df == 2 && dr == 1)
            }
            Kind::King => {
                // キャスリングは別処理済み。ここは通常1マス
                let df = (file_of(from) as isize - file_of(to) as isize).abs();
                let dr = (rank_of(from) as isize - rank_of(to) as isize).abs();
                df <= 1 && dr <= 1
            }
            Kind::Bishop => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if df.abs() != dr.abs() || df == 0 {
                    return false;
                }
                let stepf = df.signum();
                let stepr = dr.signum();
                self.line_clear(from, to, stepf, stepr)
            }
            Kind::Rook => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if !(df == 0 || dr == 0) {
                    return false;
                }
                let stepf = df.signum();
                let stepr = dr.signum();
                if df == 0 && dr == 0 {
                    return false;
                }
                self.line_clear(from, to, stepf, stepr)
            }
            Kind::Queen => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if !(df == 0 || dr == 0 || df.abs() == dr.abs()) {
                    return false;
                }
                let stepf = df.signum();
                let stepr = dr.signum();
                self.line_clear(from, to, stepf, stepr)
            }
            Kind::Pawn => {
                let dir: isize = if me.color == Color::White { 1 } else { -1 };
                let f_from = file_of(from) as isize;
                let r_from = rank_of(from) as isize;
                let f_to = file_of(to) as isize;
                let r_to = rank_of(to) as isize;

                let df = f_to - f_from;
                let dr = r_to - r_from;

                // 捕獲（通常 or EP）
                if is_capture {
                    if dr == dir && df.abs() == 1 {
                        if self.piece_at(to).is_some() {
                            return true;
                        }
                        if Some(to) == self.ep_square {
                            return true;
                        }
                    }
                    return false;
                } else {
                    // 前進1
                    if df == 0 && dr == dir && self.piece_at(to).is_none() {
                        return true;
                    }
                    // 初手2
                    let start_rank = if me.color == Color::White { 1 } else { 6 };
                    if df == 0 && dr == 2 * dir && rank_of(from) == start_rank {
                        let mid = to_idx(f_from, r_from + dir);
                        if self.piece_at(mid).is_none() && self.piece_at(to).is_none() {
                            return true;
                        }
                    }
                    return false;
                }
            }
        }
    }
}

/// マスの文字列表記（"e4"など）を盤面インデックスに変換する
///
/// # 引数
/// * `s` - マスの文字列表記（例: "a1", "e4", "h8"）
fn parse_square(s: &str) -> Result<usize, String> {
    if s.len() != 2 {
        return Err(format!("Bad square '{}'", s));
    }
    let b = s.as_bytes();
    if !(b'a'..=b'h').contains(&b[0]) || !(b'1'..=b'8').contains(&b[1]) {
        return Err(format!("Bad square '{}'", s));
    }
    Ok(idx((b[0] - b'a') as usize, (b[1] - b'1') as usize))
}
