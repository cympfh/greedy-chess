import os
import subprocess
from dataclasses import dataclass
from enum import Enum

import streamlit as st


class PieceColor(Enum):
    """駒の色"""

    WHITE = "white"
    BLACK = "black"


class PieceType(Enum):
    """駒の種類"""

    PAWN = "pawn"
    KNIGHT = "knight"
    BISHOP = "bishop"
    ROOK = "rook"
    QUEEN = "queen"
    KING = "king"


@dataclass
class Piece:
    """チェスの駒"""

    piece_type: PieceType
    color: PieceColor

    def get_unicode(self) -> str:
        """駒のUnicode文字を返す"""
        pieces = {
            (PieceColor.WHITE, PieceType.KING): "♔",
            (PieceColor.WHITE, PieceType.QUEEN): "♕",
            (PieceColor.WHITE, PieceType.ROOK): "♖",
            (PieceColor.WHITE, PieceType.BISHOP): "♗",
            (PieceColor.WHITE, PieceType.KNIGHT): "♘",
            (PieceColor.WHITE, PieceType.PAWN): "♙",
            (PieceColor.BLACK, PieceType.KING): "♚",
            (PieceColor.BLACK, PieceType.QUEEN): "♛",
            (PieceColor.BLACK, PieceType.ROOK): "♜",
            (PieceColor.BLACK, PieceType.BISHOP): "♝",
            (PieceColor.BLACK, PieceType.KNIGHT): "♞",
            (PieceColor.BLACK, PieceType.PAWN): "♟",
        }
        return pieces[(self.color, self.piece_type)]


class ChessBoard:
    """チェス盤の状態管理"""

    def __init__(self) -> None:
        """初期配置でチェス盤を初期化"""
        self.board: list[list[Piece | None]] = [
            [None for _ in range(8)] for _ in range(8)
        ]
        self.selected_square: tuple[int, int] | None = None
        self.move_history: list[str] = []
        self._setup_initial_position()

    def _setup_initial_position(self) -> None:
        """初期配置をセットアップ"""
        # 黒の駒(8th rank)
        self.board[7][0] = Piece(PieceType.ROOK, PieceColor.BLACK)
        self.board[7][1] = Piece(PieceType.KNIGHT, PieceColor.BLACK)
        self.board[7][2] = Piece(PieceType.BISHOP, PieceColor.BLACK)
        self.board[7][3] = Piece(PieceType.QUEEN, PieceColor.BLACK)
        self.board[7][4] = Piece(PieceType.KING, PieceColor.BLACK)
        self.board[7][5] = Piece(PieceType.BISHOP, PieceColor.BLACK)
        self.board[7][6] = Piece(PieceType.KNIGHT, PieceColor.BLACK)
        self.board[7][7] = Piece(PieceType.ROOK, PieceColor.BLACK)

        # 黒のポーン(7th rank)
        for file in range(8):
            self.board[6][file] = Piece(PieceType.PAWN, PieceColor.BLACK)

        # 白のポーン(2nd rank)
        for file in range(8):
            self.board[1][file] = Piece(PieceType.PAWN, PieceColor.WHITE)

        # 白の駒(1st rank)
        self.board[0][0] = Piece(PieceType.ROOK, PieceColor.WHITE)
        self.board[0][1] = Piece(PieceType.KNIGHT, PieceColor.WHITE)
        self.board[0][2] = Piece(PieceType.BISHOP, PieceColor.WHITE)
        self.board[0][3] = Piece(PieceType.QUEEN, PieceColor.WHITE)
        self.board[0][4] = Piece(PieceType.KING, PieceColor.WHITE)
        self.board[0][5] = Piece(PieceType.BISHOP, PieceColor.WHITE)
        self.board[0][6] = Piece(PieceType.KNIGHT, PieceColor.WHITE)
        self.board[0][7] = Piece(PieceType.ROOK, PieceColor.WHITE)

    def get_piece(self, rank: int, file: int) -> Piece | None:
        """指定位置の駒を取得"""
        return self.board[rank][file]

    def move_piece(
        self, from_rank: int, from_file: int, to_rank: int, to_file: int
    ) -> bool:
        """駒を移動する"""
        piece: Piece | None = self.board[from_rank][from_file]
        if piece is None:
            return False

        # 移動を記録
        from_square: str = self._square_to_notation(from_rank, from_file)
        to_square: str = self._square_to_notation(to_rank, to_file)
        captured: Piece | None = self.board[to_rank][to_file]

        # 駒の種類記号を取得
        piece_prefix: str = self._get_piece_prefix(piece)

        # プロモーションをチェック
        is_promotion: bool = self._is_promotion(piece, to_rank)

        # 移動の表記を作成
        if captured:
            move_str = f"{piece_prefix}{from_square}x{to_square}"
        else:
            move_str = f"{piece_prefix}{from_square}{to_square}"

        # プロモーションの場合は=Qを追加
        if is_promotion:
            move_str += "=Q"

        # 駒を移動
        self.board[to_rank][to_file] = piece
        self.board[from_rank][from_file] = None

        # 棋譜に追加
        self.move_history.append(move_str)

        return True

    def _is_promotion(self, piece: Piece, to_rank: int) -> bool:
        """プロモーションかどうかをチェック"""
        if piece.piece_type != PieceType.PAWN:
            return False

        # 白のポーンが8th rank(rank=7)に到達
        if piece.color == PieceColor.WHITE and to_rank == 7:
            return True

        # 黒のポーンが1st rank(rank=0)に到達
        if piece.color == PieceColor.BLACK and to_rank == 0:
            return True

        return False

    def _get_piece_prefix(self, piece: Piece) -> str:
        """駒の種類に応じた接頭辞を返す"""
        piece_prefixes = {
            PieceType.KING: "K",
            PieceType.QUEEN: "Q",
            PieceType.ROOK: "R",
            PieceType.BISHOP: "B",
            PieceType.KNIGHT: "N",
            PieceType.PAWN: "",  # ポーンは接頭辞なし
        }
        return piece_prefixes.get(piece.piece_type, "")

    def _square_to_notation(self, rank: int, file: int) -> str:
        """座標をチェス記法に変換"""
        files: str = "abcdefgh"
        return f"{files[file]}{rank + 1}"

    def select_square(self, rank: int, file: int) -> None:
        """マスを選択"""
        # 既に選択されている場合は移動を試みる
        if self.selected_square is not None:
            from_rank, from_file = self.selected_square
            if (from_rank, from_file) != (rank, file):
                # キャスリングかチェック
                castling_notation: str | None = self._check_castling(
                    from_rank, from_file, rank, file
                )
                if castling_notation:
                    self._record_castling(castling_notation)
                else:
                    self.move_piece(from_rank, from_file, rank, file)
                # 移動後、Rust AIと同期
                self.sync_board_with_rust()
            self.selected_square = None
        else:
            # 駒がある場合のみ選択
            if self.board[rank][file] is not None:
                self.selected_square = (rank, file)

    def _check_castling(
        self, from_rank: int, from_file: int, to_rank: int, to_file: int
    ) -> str | None:
        """キャスリングかどうかをチェックし、該当する場合は記法を返す"""
        if from_rank != to_rank:
            return None
        piece: Piece | None = self.board[from_rank][from_file]
        if piece is None or piece.piece_type != PieceType.KING:
            return None

        # キングの移動が2マス以上の場合はキャスリング
        file_diff: int = abs(to_file - from_file)
        if file_diff == 2:
            # キングサイド（右）: O-O
            if to_file > from_file:
                return "O-O"
            # クイーンサイド（左）: O-O-O
            else:
                return "O-O-O"

        return None

    def _record_castling(self, notation: str) -> None:
        """キャスリングを記録"""
        self.move_history.append(notation)

    def get_kifu_string(self) -> str:
        """棋譜を文字列として取得(URL用)"""
        return " ".join(self.move_history)

    def load_kifu_from_string(self, kifu_string: str) -> bool:
        """棋譜文字列を読み込んで盤面を再構築"""
        if not kifu_string:
            return False

        try:
            # 棋譜を空白で分割して移動履歴に設定
            self.move_history = kifu_string.strip().split()

            # Rust AIと同期して正しい盤面を取得
            return self.sync_board_with_rust()
        except Exception:
            return False

    def get_best_move(self, timeout: int) -> str | None:
        """Rust AIから最善手を取得（並列探索）"""
        kifu: str = self.get_kifu_string()
        process = subprocess.Popen(
            ["cargo", "run", "--release", "--", "-t", str(timeout)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            cwd=os.path.dirname(os.path.abspath(__file__)),
        )
        try:
            stdout, stderr = process.communicate(input=kifu, timeout=timeout + 1.0)
            print("# get_best_move")
            print(stdout)
            print(stderr)
            if process.returncode == 0:
                output_lines: list[str] = stdout.strip().split("\n")
                if output_lines:
                    lines = [line for line in output_lines if not line.startswith(";")]
                    return lines[0].strip()
            return None
        except subprocess.TimeoutExpired:
            process.kill()
            return None
        except Exception:
            return None

    def sync_board_with_rust(self) -> bool:
        """Rust AIから盤面の状態を取得して同期"""
        kifu: str = self.get_kifu_string()

        # 棋譜が空の場合は同期不要
        if not kifu:
            return True

        try:
            # cargo run --release -- -p を実行
            process = subprocess.Popen(
                ["cargo", "run", "--release", "--", "-p"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                cwd=os.path.dirname(os.path.abspath(__file__)),
            )

            # 棋譜を標準入力に渡す
            stdout, stderr = process.communicate(input=kifu, timeout=5)
            print("# sync_board_with_rust")
            print(stdout)
            print(stderr)

            if process.returncode == 0:
                # 出力を解析して盤面を更新
                self._parse_and_update_board(stdout)
                return True

            return False
        except Exception:
            return False

    def _parse_and_update_board(self, rust_output: str) -> None:
        """Rust AIの出力を解析して盤面を更新"""
        lines: list[str] = rust_output.strip().split("\n")

        # 駒の文字とPieceへのマッピング
        piece_map = {
            "P": Piece(PieceType.PAWN, PieceColor.WHITE),
            "N": Piece(PieceType.KNIGHT, PieceColor.WHITE),
            "B": Piece(PieceType.BISHOP, PieceColor.WHITE),
            "R": Piece(PieceType.ROOK, PieceColor.WHITE),
            "Q": Piece(PieceType.QUEEN, PieceColor.WHITE),
            "K": Piece(PieceType.KING, PieceColor.WHITE),
            "p": Piece(PieceType.PAWN, PieceColor.BLACK),
            "n": Piece(PieceType.KNIGHT, PieceColor.BLACK),
            "b": Piece(PieceType.BISHOP, PieceColor.BLACK),
            "r": Piece(PieceType.ROOK, PieceColor.BLACK),
            "q": Piece(PieceType.QUEEN, PieceColor.BLACK),
            "k": Piece(PieceType.KING, PieceColor.BLACK),
            ".": None,
        }

        # 盤面をクリア
        self.board = [[None for _ in range(8)] for _ in range(8)]

        # 各行を解析
        for line in lines:
            if not line.startswith(";"):
                continue

            # "; 8 r n b q k b n r" のような形式
            parts: list[str] = line.split()
            if len(parts) < 10:  # "; " + rank + 8 pieces
                continue

            # ランク番号を取得 (1-8)
            try:
                rank_num: int = int(parts[1])
            except (ValueError, IndexError):
                continue

            # ランク番号を配列インデックスに変換 (0-7)
            rank: int = rank_num - 1

            # 各ファイルの駒を解析
            for file in range(8):
                if file + 2 < len(parts):
                    piece_char: str = parts[file + 2]
                    if piece_char in piece_map:
                        self.board[rank][file] = piece_map[piece_char]


def render_chess_board(chess_board: ChessBoard) -> None:
    """チェス盤をレンダリング"""
    # ファイル名のラベル
    col_labels: list[str] = ["a", "b", "c", "d", "e", "f", "g", "h"]

    # 8x8の盤面を表示(上から8th rank)
    for rank in range(7, -1, -1):
        cols = st.columns([0.5] + [1] * 8 + [0.5])

        # ランク番号(左)
        with cols[0]:
            st.markdown(
                f"<div style='text-align: center; font-weight: bold;'>{rank + 1}</div>",
                unsafe_allow_html=True,
            )

        # 各マス
        for file in range(8):
            with cols[file + 1]:
                piece: Piece | None = chess_board.get_piece(rank, file)
                piece_symbol: str = piece.get_unicode() if piece else ""

                # 明るいマス（白）か暗いマス（灰色）かを判定
                is_light: bool = (rank + file) % 2 == 0
                button_type: str = "secondary" if is_light else "tertiary"

                # ボタンで各マスを作成
                if st.button(
                    piece_symbol,
                    key=f"sq_{rank}_{file}",
                    type=button_type,
                    help=f"{col_labels[file]}{rank + 1}",
                    use_container_width=True,
                ):
                    chess_board.select_square(rank, file)
                    st.rerun()

        # ランク番号(右)
        with cols[9]:
            st.markdown(
                f"<div style='text-align: center; font-weight: bold;'>{rank + 1}</div>",
                unsafe_allow_html=True,
            )

    # ファイル名のラベル
    cols = st.columns([0.5] + [1] * 8 + [0.5])
    for i, label in enumerate(col_labels):
        with cols[i + 1]:
            st.markdown(
                f"<div style='text-align: center; font-weight: bold;'>{label}</div>",
                unsafe_allow_html=True,
            )


def main() -> None:
    st.set_page_config(page_title="Chess Visualizer", layout="wide")
    st.title("♔  Chess Simulator ♙")

    # セッション状態の初期化
    if "chess_board" not in st.session_state:
        st.session_state.chess_board = ChessBoard()

        # URLパラメータから棋譜を読み込む
        kifu_from_url: str | None = st.query_params.get("kifu", None)
        if kifu_from_url:
            st.session_state.chess_board.load_kifu_from_string(kifu_from_url)

    # AI推奨手のキャッシュを初期化
    if "best_move_cache" not in st.session_state:
        st.session_state.best_move_cache = {}

    chess_board: ChessBoard = st.session_state.chess_board

    # 棋譜をURLパラメータに反映
    kifu_string: str = chess_board.get_kifu_string()
    if kifu_string:
        st.query_params["kifu"] = kifu_string
    elif "kifu" in st.query_params:
        # 棋譜が空の場合はパラメータを削除
        del st.query_params["kifu"]

    # レイアウト: 左に盤面、右に棋譜
    col1, col2 = st.columns([2, 1])

    with col1:
        st.subheader("盤面")
        # 棋譜の手数から現在のターンを判定（偶数=白、奇数=黒）
        total_moves: int = len(chess_board.move_history)
        turn_text: str = "白" if total_moves % 2 == 0 else "黒"
        st.info(f"現在のターン: {turn_text}")
        render_chess_board(chess_board)

        # リセットボタン
        if st.button("盤面をリセット", type="secondary"):
            st.session_state.chess_board = ChessBoard()
            st.rerun()

    with col2:
        st.subheader("🤖 AI推奨手")

        if "ai_timeout" not in st.session_state:
            st.session_state.ai_timeout = 3

        ai_timeout = st.slider(
            "AI思考時間の上限を選択 (秒)",
            min_value=1,
            max_value=30,
            value=st.session_state.ai_timeout,
            step=1,
            key="ai_timeout_slider",
        )
        st.session_state.ai_timeout = ai_timeout

        current_kifu: str = chess_board.get_kifu_string()
        cache_key = f"{current_kifu}_t{ai_timeout}"
        result = None
        if cache_key in st.session_state.best_move_cache:
            result = st.session_state.best_move_cache[cache_key]
        else:
            with st.spinner("AIが思考中..."):
                result = chess_board.get_best_move(ai_timeout)
                st.session_state.best_move_cache[cache_key] = result
        if result:
            st.success(f"**{result}**")
        else:
            st.warning("最善手を取得できませんでした")

        # 棋譜をスクロール可能な領域に表示
        st.divider()
        st.subheader("棋譜 (Move History)")

        if chess_board.move_history:
            # スクロール可能なコンテナを作成
            moves_text: str = "\n".join(chess_board.move_history)
            st.markdown(
                f"""
                <div style="height: 400px; overflow-y: auto; border: 1px solid #ddd; padding: 10px; background-color: #f9f9f9; border-radius: 5px;">
                <pre style="margin: 0; font-family: monospace;">{moves_text}</pre>
                </div>
                """,
                unsafe_allow_html=True,
            )
        else:
            st.info("まだ手が指されていません")


if __name__ == "__main__":
    main()
