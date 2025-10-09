use std::collections::HashSet;
use std::io::{self, Read};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Color { White, Black }

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Kind { Pawn, Knight, Bishop, Rook, Queen, King }

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Piece { kind: Kind, color: Color }

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Move {
    from: usize, // 0..63
    to: usize,   // 0..63
    promo: Option<Kind>,
    is_capture: bool,
    is_en_passant: bool,
    is_castle_kingside: bool,
    is_castle_queenside: bool,
}

#[derive(Clone)]
struct Board {
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

fn idx(file: usize, rank: usize) -> usize { rank * 8 + file } // file:0..7(a..h), rank:0..7(1..8 だが0が1段)

fn file_of(i: usize) -> usize { i % 8 }
fn rank_of(i: usize) -> usize { i / 8 }

fn in_bounds(file: isize, rank: isize) -> bool {
    (0..8).contains(&file) && (0..8).contains(&rank)
}
fn to_idx(file: isize, rank: isize) -> usize { (rank as usize) * 8 + (file as usize) }

impl Board {
    fn new_startpos() -> Self {
        use Color::*;
        use Kind::*;
        let mut sq = [None; 64];

        // 白
        let back_white = [
            Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook
        ];
        for f in 0..8 {
            sq[idx(f,0)] = Some(Piece{kind: back_white[f], color: White});
            sq[idx(f,1)] = Some(Piece{kind: Pawn, color: White});
        }
        // 黒
        let back_black = [
            Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook
        ];
        for f in 0..8 {
            sq[idx(f,7)] = Some(Piece{kind: back_black[f], color: Black});
            sq[idx(f,6)] = Some(Piece{kind: Pawn, color: Black});
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

    fn piece_at(&self, i: usize) -> Option<Piece> { self.sq[i] }
    fn set_piece(&mut self, i: usize, p: Option<Piece>) { self.sq[i] = p; }

    fn print(&self) {
        println!();
        for r in (0..8).rev() {
            print!("{} ", r+1);
            for f in 0..8 {
                let i = idx(f,r);
                match self.sq[i] {
                    None => print!(". "),
                    Some(p) => {
                        let c = match (p.kind, p.color) {
                            (Kind::Pawn,   Color::White) => 'P',
                            (Kind::Knight, Color::White) => 'N',
                            (Kind::Bishop, Color::White) => 'B',
                            (Kind::Rook,   Color::White) => 'R',
                            (Kind::Queen,  Color::White) => 'Q',
                            (Kind::King,   Color::White) => 'K',
                            (Kind::Pawn,   Color::Black) => 'p',
                            (Kind::Knight, Color::Black) => 'n',
                            (Kind::Bishop, Color::Black) => 'b',
                            (Kind::Rook,   Color::Black) => 'r',
                            (Kind::Queen,  Color::Black) => 'q',
                            (Kind::King,   Color::Black) => 'k',
                        };
                        print!("{} ", c);
                    }
                }
            }
            println!();
        }
        println!("  a b c d e f g h");
        println!("Side to move: {:?}", self.side);
        println!("Castling: {}{}{}{}",
            if self.castle_wk {'K'} else {'-'},
            if self.castle_wq {'Q'} else {'-'},
            if self.castle_bk {'k'} else {'-'},
            if self.castle_bq {'q'} else {'-'});
        if let Some(ep) = self.ep_square {
            let f = (file_of(ep) as u8 + b'a') as char;
            let r = (rank_of(ep) + 1).to_string();
            println!("En passant: {}{}", f, r);
        } else {
            println!("En passant: -");
        }
        println!();
    }

    fn is_own(&self, p: Piece) -> bool { p.color == self.side }

    fn other(c: Color) -> Color { if let Color::White = c { Color::Black } else { Color::White } }

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
                if m.is_castle_kingside { (idx(4,0), idx(6,0), idx(7,0), idx(5,0)) }
                else { (idx(4,0), idx(2,0), idx(0,0), idx(3,0)) }
            } else {
                if m.is_castle_kingside { (idx(4,7), idx(6,7), idx(7,7), idx(5,7)) }
                else { (idx(4,7), idx(2,7), idx(0,7), idx(3,7)) }
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
                if m.from == idx(4,0) { self.castle_wk = false; self.castle_wq = false; }
                if m.from == idx(0,0) || m.to == idx(0,0) { self.castle_wq = false; }
                if m.from == idx(7,0) || m.to == idx(7,0) { self.castle_wk = false; }
                // 黒ルークが取られたら黒権利調整
                if m.to == idx(0,7) { self.castle_bq = false; }
                if m.to == idx(7,7) { self.castle_bk = false; }
            }
            Color::Black => {
                if m.from == idx(4,7) { self.castle_bk = false; self.castle_bq = false; }
                if m.from == idx(0,7) || m.to == idx(0,7) { self.castle_bq = false; }
                if m.from == idx(7,7) || m.to == idx(7,7) { self.castle_bk = false; }
                if m.to == idx(0,0) { self.castle_wq = false; }
                if m.to == idx(7,0) { self.castle_wk = false; }
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
        if moved.kind == Kind::Pawn || m.is_capture { self.halfmove_clock = 0; }
        else { self.halfmove_clock += 1; }

        if self.side == Color::Black { self.fullmove_number += 1; }
        self.side = Board::other(self.side);
    }

    // ============ ここから指し手解釈（UCI/LAN 先、SAN 簡易後） ============

    fn parse_and_play_token(&mut self, token: &str) -> Result<(), String> {
        let t = token.trim();
        if t.is_empty() { return Ok(()); }

        // 結果記号や注釈はスキップ
        if t == "1-0" || t == "0-1" || t == "1/2-1/2" || t == "*" { return Ok(()); }
        if t.ends_with('.') { return Ok(()); } // move number like "12."

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

    fn build_castle(&self, kingside: bool) -> Result<Move, String> {
        let (from, to) = match self.side {
            Color::White => if kingside {(idx(4,0), idx(6,0))} else {(idx(4,0), idx(2,0))},
            Color::Black => if kingside {(idx(4,7), idx(6,7))} else {(idx(4,7), idx(2,7))},
        };
        Ok(Move{
            from, to, promo: None, is_capture: false, is_en_passant: false,
            is_castle_kingside: kingside, is_castle_queenside: !kingside
        })
    }

    fn try_parse_uci_like(&self, t: &str) -> Result<Option<Move>, String> {
        // 例: "e2e4", "e7e8Q"
        // a-h, 1-8, a-h, 1-8, [NBRQ]
        if t.len() < 4 { return Ok(None); }
        let b = t.as_bytes();
        let is_square = |f: u8, r: u8| (b'a'..=b'h').contains(&f) && (b'1'..=b'8').contains(&r);
        if !is_square(b[0], b[1]) || !is_square(b[2], b[3]) {
            return Ok(None);
        }
        let from = idx((b[0]-b'a') as usize, (b[1]-b'1') as usize);
        let to   = idx((b[2]-b'a') as usize, (b[3]-b'1') as usize);

        let promo = if t.len() >= 5 {
            match t.as_bytes()[4] as char {
                'q'|'Q' => Some(Kind::Queen),
                'r'|'R' => Some(Kind::Rook),
                'b'|'B' => Some(Kind::Bishop),
                'n'|'N' => Some(Kind::Knight),
                _ => None
            }
        } else { None };

        // 捕獲かどうかと EP をざっくり判断
        let mut is_capture = self.piece_at(to).is_some();
        let mut is_ep = false;
        if let Some(p) = self.piece_at(from) {
            if p.kind == Kind::Pawn && !is_capture && Some(to) == self.ep_square {
                is_ep = true;
                is_capture = true;
            }
        }

        Ok(Some(Move{
            from, to, promo, is_capture, is_en_passant: is_ep,
            is_castle_kingside: false, is_castle_queenside: false
        }))
    }

    fn parse_san_and_find_move(&self, t: &str) -> Result<Move, String> {
        // SAN の記号除去（+, #, !? など）
        let mut s = t.replace("+", "").replace("#", "");
        s = s.trim_end_matches(['!','?'].as_ref()).to_string();

        // 昇格表記 e8=Q
        let mut promo: Option<Kind> = None;
        if let Some(eq) = s.find('=') {
            let p = s.as_bytes()[eq+1] as char;
            promo = match p {
                'Q' => Some(Kind::Queen),
                'R' => Some(Kind::Rook),
                'B' => Some(Kind::Bishop),
                'N' => Some(Kind::Knight),
                _ => None
            };
            s.truncate(eq);
        }

        // 取り "x" の有無
        let is_capture = s.contains('x');
        let s_clean = s.replace("x", "");

        // 駒種
        let (kind, rest) = match s_clean.chars().next().unwrap_or(' ') {
            'K' => (Kind::King,   &s_clean[1..]),
            'Q' => (Kind::Queen,  &s_clean[1..]),
            'R' => (Kind::Rook,   &s_clean[1..]),
            'B' => (Kind::Bishop, &s_clean[1..]),
            'N' => (Kind::Knight, &s_clean[1..]),
            'O' | '0' => return Err("Use O-O / O-O-O handled earlier".into()),
            _   => (Kind::Pawn,   &s_clean[..]),
        };

        // 残りは [disambiguation?] + destination square
        // 末尾2文字が目的地（例: e4）。その前に 0〜2 文字の曖昧性解消（例: "Nbd7", "R1e2", "Qhxe5"→clean後 "Qhe5"）
        if rest.len() < 2 { return Err(format!("SAN too short: {}", t)); }
        let dest_part = &rest[rest.len()-2..];
        let to = parse_square(dest_part)?;

        let mut dis_file: Option<usize> = None;
        let mut dis_rank: Option<usize> = None;
        let dis = &rest[..rest.len()-2];
        for ch in dis.chars() {
            if ('a'..='h').contains(&ch) {
                dis_file = Some((ch as u8 - b'a') as usize);
            } else if ('1'..='8').contains(&ch) {
                dis_rank = Some((ch as u8 - b'1') as usize);
            }
        }

        // 目的地に到達できる自軍の候補駒から1つ選ぶ
        let candidates = self.generate_reach(kind, to, is_capture);
        let filtered: Vec<usize> = candidates.into_iter().filter(|&sq_from| {
            if let Some(df) = dis_file { if file_of(sq_from) != df { return false; } }
            if let Some(dr) = dis_rank { if rank_of(sq_from) != dr { return false; } }
            true
        }).collect();

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
        if kind == Kind::Pawn && is_capture && self.piece_at(to).is_none() && Some(to) == self.ep_square {
            is_ep = true;
        }

        Ok(Move{
            from, to, promo, is_capture, is_en_passant: is_ep,
            is_castle_kingside: false, is_castle_queenside: false
        })
    }

    fn generate_reach(&self, kind: Kind, to: usize, is_capture: bool) -> Vec<usize> {
        // 「素直」な到達元探索：現在手番の自軍で、指定種が to に動ける from 候補（最小限のルール）
        let mut v = Vec::new();
        for i in 0..64 {
            if let Some(p) = self.piece_at(i) {
                if p.color != self.side || p.kind != kind { continue; }
                if self.naive_can_reach(i, to, is_capture) {
                    v.push(i);
                }
            }
        }
        v
    }

    fn line_clear(&self, from: usize, to: usize, df: isize, dr: isize) -> bool {
        // from から to へ (df,dr) 方向に一直線で、途中が空か
        let mut f = file_of(from) as isize + df;
        let mut r = rank_of(from) as isize + dr;
        while in_bounds(f,r) {
            let cur = to_idx(f,r);
            if cur == to { return true; }
            if self.piece_at(cur).is_some() { return false; }
            f += df; r += dr;
        }
        false
    }

    fn naive_can_reach(&self, from: usize, to: usize, is_capture: bool) -> bool {
        if from == to { return false; }
        let me = match self.piece_at(from) { Some(p)=>p, None=>return false };
        if let Some(dst) = self.piece_at(to) {
            if dst.color == me.color { return false; }
        }
        match me.kind {
            Kind::Knight => {
                let df = (file_of(from) as isize - file_of(to) as isize).abs();
                let dr = (rank_of(from) as isize - rank_of(to) as isize).abs();
                (df==1 && dr==2) || (df==2 && dr==1)
            }
            Kind::King => {
                // キャスリングは別処理済み。ここは通常1マス
                let df = (file_of(from) as isize - file_of(to) as isize).abs();
                let dr = (rank_of(from) as isize - rank_of(to) as isize).abs();
                df<=1 && dr<=1
            }
            Kind::Bishop => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if df.abs()!=dr.abs() || df==0 { return false; }
                let stepf = df.signum();
                let stepr = dr.signum();
                self.line_clear(from, to, stepf, stepr)
            }
            Kind::Rook => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if !(df==0 || dr==0) { return false; }
                let stepf = df.signum();
                let stepr = dr.signum();
                if df==0 && dr==0 { return false; }
                self.line_clear(from, to, stepf, stepr)
            }
            Kind::Queen => {
                let df = file_of(to) as isize - file_of(from) as isize;
                let dr = rank_of(to) as isize - rank_of(from) as isize;
                if !(df==0 || dr==0 || df.abs()==dr.abs()) { return false; }
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
                    if dr == dir && df.abs()==1 {
                        if self.piece_at(to).is_some() { return true; }
                        if Some(to) == self.ep_square { return true; }
                    }
                    return false;
                } else {
                    // 前進1
                    if df==0 && dr==dir && self.piece_at(to).is_none() { return true; }
                    // 初手2
                    let start_rank = if me.color==Color::White { 1 } else { 6 };
                    if df==0 && dr==2*dir && rank_of(from)==start_rank {
                        let mid = to_idx(f_from, r_from + dir);
                        if self.piece_at(mid).is_none() && self.piece_at(to).is_none() { return true; }
                    }
                    return false;
                }
            }
        }
    }
}

fn parse_square(s: &str) -> Result<usize, String> {
    if s.len()!=2 { return Err(format!("Bad square '{}'", s)); }
    let b = s.as_bytes();
    if !(b'a'..=b'h').contains(&b[0]) || !(b'1'..=b'8').contains(&b[1]) {
        return Err(format!("Bad square '{}'", s));
    }
    Ok(idx((b[0]-b'a') as usize, (b[1]-b'1') as usize))
}

fn kind_from_char(c: char) -> Option<Kind> {
    match c {
        'q'|'Q' => Some(Kind::Queen),
        'r'|'R' => Some(Kind::Rook),
        'b'|'B' => Some(Kind::Bishop),
        'n'|'N' => Some(Kind::Knight),
        _ => None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 標準入力から棋譜（空白区切りの手）を読み、順次適用
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let tokens: Vec<String> = buf.split_whitespace().map(|s| s.to_string()).collect();

    let mut board = Board::new_startpos();

    for (ply, tok) in tokens.iter().enumerate() {
        if tok.ends_with('.') { continue; } // "12." など無視
        if tok.starts_with('{') && tok.ends_with('}') { continue; } // コメント { ... } 簡易無視
        if tok.starts_with(';') { continue; } // セミコロ解説行を無視

        if let Err(e) = board.parse_and_play_token(tok) {
            eprintln!("Failed at ply {} on token '{}': {}", ply+1, tok, e);
            break;
        }
    }

    board.print();
    Ok(())
}
