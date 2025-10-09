# TODO.md

## ✅ Completed Core Features (2024-10-09)

### Core AI Algorithm
- [x] **Min-Max探索アルゴリズム実装**: 完全なmin-maxアルゴリズム実装済み
- [x] **コマンドライン引数処理**: 探索深度nをコマンドライン引数から受け取る機能（デフォルト3）
- [x] **評価関数実装**: 駒の価値に基づく局面評価関数（P:1, N/B:3, R:5, Q:9, K:999）
- [x] **次の一手生成**: SAN形式でAIの指し手を出力 + コメント形式でボード表示

### Legal Move Generation
- [x] **合法手生成**: 全駒種の完全な合法手生成システム実装
- [x] **チェック判定**: キングが攻撃されているかの判定機能
- [x] **チェックメイト/ステイルメイト判定**: ゲーム終了条件の検出機能
- [x] **ピン/スキュワー処理**: 自分のキングをチェックに晒す手の除外機能

### Move Validation Enhancement
- [x] **厳密なキャスリング条件**: 完全なキャスリング可能性チェック
  - [x] キング・ルークの移動履歴管理
  - [x] キングとルークの間が空いている
  - [x] キングが攻撃されていない
  - [x] 通過地点が攻撃されていない
  - [x] 到着地点が攻撃されていない
- [x] **アンパッサン条件**: 完全なen passant判定実装

## 🔴 High Priority Remaining Items

### Algorithm Improvements
- [ ] **アルファベータ剪定**: Min-maxの高速化（現在はfull tree search）
- [ ] **移動順序付け**: より効率的な探索のための手の順序付け
- [ ] **置換表**: 既に評価した局面のキャッシュ（メモ化）
- [ ] **反復深化**: 時間制限内での最適解探索

### Chess Rules Compliance
- [ ] **三回同型局面**: 同じ局面が3回現れた場合の引き分け判定
- [ ] **50手ルール**: 駒取りやポーン移動がない状態での引き分け判定
- [ ] **不十分な駒による引き分け**: K vs K, KB vs K等の判定

## 🟡 Medium Priority Items

### Performance Optimization
- [ ] **探索の最適化**: 現在の実装は深い探索で時間がかかる可能性
- [ ] **評価関数の改善**: ポジション要素（中央制御、駒の活動性等）追加
- [ ] **メモリ使用量削減**: 不要なボードクローンの最適化

## 🟢 Enhancement Features

### User Interface
- [x] **SAN出力**: SAN形式での指し手出力実装済み + コメント形式ボード表示
- [ ] **PGN対応**: 完全なPGNファイル読み込み/出力
- [ ] **FEN対応**: FEN記法での局面入出力
- [ ] **対話モード**: 継続的に手を入力して対戦できるモード

### Advanced Features
- [ ] **並列探索**: マルチスレッドでの探索高速化
- [ ] **開始局面データベース**: オープニングブック
- [ ] **終盤データベース**: エンドゲームテーブルベース

## 🟢 Low Priority Items

### Code Quality
- [ ] **未使用コード削除**: 現在警告が出ている未使用関数の整理
  - [ ] `is_own`メソッドの削除または使用
  - [ ] `kind_from_char`関数の削除または使用
  - [ ] `print`メソッドの削除（`print_as_comment`で代替）
- [ ] **エラー処理改善**: より詳細で親切なエラーメッセージ
- [ ] **テスト追加**: ユニットテストとインテグレーションテスト
- [ ] **ドキュメント**: 関数とモジュールのドキュメント化

### Configuration & Debugging
- [ ] **設定ファイル**: 評価パラメータの外部設定
- [ ] **ログ機能**: 探索過程の詳細ログ出力
- [ ] **デバッグモード**: 探索ツリーの可視化

### Code Organization
- [ ] **モジュール分割**: 大きなmain.rsを機能別モジュールに分割
- [ ] **型安全性**: newtype patternによる座標系の型安全化
- [ ] **エラー型**: カスタムエラー型の定義

## 📊 Current Status Summary

**✅ WORKING**: 基本的なチェスAIとして動作中
- READMEの使用例が正常に動作
- 探索深度をコマンドライン引数で指定可能
- SAN形式での指し手出力 + ボード状態表示

**⚡ PERFORMANCE**: 深い探索（depth > 4）では時間がかかる場合がある

**🎯 NEXT STEPS**: アルファベータ剪定の実装が最も効果的な改善項目

## Priority Order (Updated)

1. **Immediate**: Algorithm Improvements (Alpha-Beta pruning)
2. **Short-term**: Chess Rules Compliance, Performance Optimization
3. **Long-term**: Advanced Features, Code Quality improvements

## Implementation Notes

- ✅ 基本的なAI機能は完全に実装済み - READMEの仕様を満たしている
- 🚀 パフォーマンス改善（アルファベータ剪定）が次の重要な改善点
- 📈 現在の実装は機能的に正しく、段階的に最適化可能な構造