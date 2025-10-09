# greedy-chess

## Overview

シンプルな貪欲法によるチェスAI.
途中までの棋譜を標準入力に与えると, 次の一手を標準出力に出力する.

## Algorithm

- min-max による
- n 手先までだけ読む
    - n はコマンドライン引数で指定
- 評価関数は駒の価値の和
    - 駒の価値は以下の通り
        - ポーン: 1
        - ナイト: 3
        - ビショップ: 3
        - ルーク: 5
        - クイーン: 9
        - キング: 999

## Usage

```bash
$ cargo run <<EOF
e4 e5 Nf3 Nc6 Bb5 a6 Ba4 Nf6 O-O Be7
EOF

Re1
```
