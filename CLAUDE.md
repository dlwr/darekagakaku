# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

「誰かが書く日記」- Cloudflare Workers + D1で動作する共有日記アプリ。誰でも今日の日記を書けるが、日付が変わると編集不可になる。

## 開発コマンド

タスクランナーに [mise](https://mise.jdx.dev/) を使用。`mise install` でツールのセットアップ完了。

```bash
# 全workerデプロイ（OG worker → main workerの順）
mise run deploy

# 個別デプロイ
mise run deploy:main
mise run deploy:og

# ローカル開発サーバー起動
mise run dev        # main worker
mise run dev:og     # OG worker

# テスト実行
mise run test       # 全テスト
mise run test:rust  # Rustテストのみ
mise run test:og    # OGワーカーテストのみ

# タスク一覧
mise tasks ls
```

### D1データベース操作

```bash
# D1データベース作成（初回のみ）
npx wrangler d1 create darekagakaku-db

# スキーマ適用（ローカル）
npx wrangler d1 execute darekagakaku-db --local --file=schema.sql

# スキーマ適用（本番）
npx wrangler d1 execute darekagakaku-db --remote --file=schema.sql
```

## アーキテクチャ

Cloudflare Workers (Rust/WASM) + D1データベースで構成。`worker` crateを使用。

### モジュール構成

- [lib.rs](src/lib.rs) - エントリポイント、ルーティング定義
- [pages.rs](src/pages.rs) - HTMLページハンドラ（フォーム処理含む）
- [handlers.rs](src/handlers.rs) - JSON APIハンドラ
- [templates.rs](src/templates.rs) - HTMLテンプレート生成（手書きHTML）
- [models.rs](src/models.rs) - データ構造とSerdeシリアライズ
- [db.rs](src/db.rs) - D1データベース操作
- [time.rs](src/time.rs) - JST時刻処理（js-sys経由）

### ルーティング

| パス | メソッド | 用途 |
|------|----------|------|
| `/` | GET | 今日の日記フォーム |
| `/a` | GET | Aboutページ |
| `/feed` | GET | RSSフィード |
| `/entries` | GET | 過去の日記一覧 |
| `/entries/:date` | GET | 特定日の日記 |
| `/admin/login` | GET/POST | 管理者ログイン（Cookie認証） |
| `/admin/logout` | GET | 管理者ログアウト |
| `/admin/versions` | GET | 管理者用：日付選択ページ |
| `/admin/entries/:date/versions` | GET | 管理者用：バージョン一覧 |
| `/admin/entries/:date/versions/:version` | GET | 管理者用：バージョン詳細 |
| `/api/today` | GET/POST | 今日の日記API |
| `/api/entries` | GET | 一覧API |
| `/api/entries/:date` | GET | 特定日API |
| `/api/admin/entries/:date/versions` | GET | 管理者用：バージョン一覧API |
| `/api/admin/entries/:date/versions/:version` | GET | 管理者用：バージョン詳細API |

### データベース

D1バインディング名: `DB`

```sql
diary_entries (
    date TEXT PRIMARY KEY,    -- YYYY-MM-DD (JST)
    content TEXT NOT NULL,    -- 最大10000文字
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)
```

### 時刻処理

WASM環境ではシステム時刻が使えないため、`js-sys::Date::now()`経由で取得し、`chrono-tz`でJST変換。`time.rs`を必ず使用すること。
