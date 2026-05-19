# 日記に画像1枚を添付する機能

## 目的

日記エントリ1件につき画像を1枚アップロード・表示できるようにする。テキストと同じく「今日のエントリのみ編集可」のルールを踏襲する。

## スコープ

- 1日記 = 1画像。差し替え時は上書き。
- 削除も可能（今日のエントリに対してのみ）。
- 過去のエントリの画像は表示のみ。
- 画像のバージョン履歴は持たない（テキスト側のversioningとは独立）。
- 一覧／RSS／OG画像への画像反映は本フェーズでは扱わない（将来検討）。

## アーキテクチャ

### ストレージ

- 画像本体: **Cloudflare R2**
  - `wrangler.toml` に新規バインディング `IMAGES`
  - オブジェクトキー: `entries/{YYYY-MM-DD}`（拡張子はキーに含めない、MIMEはD1側で持つ）
- メタデータ: **D1**
  - `diary_entries` に列追加: `image_mime TEXT NULL`
  - 値の取りうる範囲: `image/jpeg` / `image/png` / `image/webp` / `NULL`

### スキーマ変更

```sql
ALTER TABLE diary_entries ADD COLUMN image_mime TEXT;
```

`schema.sql` を更新し、`darekagakaku-db` のローカル／本番両方に手動でmigrationを適用する。

### ルーティング追加

| パス | メソッド | 用途 |
|------|----------|------|
| `/api/today/image` | POST | 今日の画像アップロード（multipart/form-data） |
| `/api/today/image` | DELETE | 今日の画像削除 |
| `/images/:date` | GET | R2から画像配信 |

### モジュール構成

- `src/image.rs` (新規): マジックバイトでのMIME判定、サイズ上限のバリデーション
- `src/handlers.rs`: 上記APIハンドラ
- `src/lib.rs`: ルーティング登録
- `src/db.rs`: `image_mime` 列の取得・更新・クリア
- `src/models.rs`: `DiaryEntry` に `image_mime: Option<String>` 追加
- `src/templates.rs`: ホームと日記ページのUI更新

## バリデーション

- **MIME判定**: クライアント送信の `Content-Type` を信用せず、ファイル先頭バイトで判定
  - JPEG: `FF D8 FF`
  - PNG: `89 50 4E 47 0D 0A 1A 0A`
  - WebP: `RIFF....WEBP`
- **サイズ上限**: 3 MB（`3 * 1024 * 1024` bytes）
- **Turnstile**: POSTでは必須（DELETEは認証不要、レートリミットのみ）
- **レートリミット**: 既存KV `RATE_LIMIT` を共有

## エラーレスポンス

| 状況 | ステータス | エラーメッセージ |
|------|------------|------------------|
| 不正なMIME | 400 | `"Unsupported image format. Use JPEG, PNG, or WebP."` |
| サイズ超過 | 413 | `"Image too large. Maximum 3MB."` |
| Turnstile失敗 | 400 | `"Turnstile verification failed"` |
| レートリミット | 429 | `"Too Many Requests"` |
| R2エラー | 500 | `"Internal server error"` |

## 配信

- `GET /images/:date`
  - 日付フォーマットチェック
  - D1から `image_mime` を取得 → なければ 404
  - R2から `entries/{date}` を取得 → なければ 404
  - レスポンス: `Content-Type: {image_mime}`、`Cache-Control: public, max-age=31536000, immutable`（同日内の差し替え時はキー単位ではURLが変わらないため即時反映は期待しない。差し替え直後の閲覧者には古い画像が見える可能性があるが、許容範囲）

## UI

### ホーム（`render_home`）

- 既存テキストエリアの下に：
  - 既存画像がある場合: プレビュー画像 + 「画像を削除」ボタン
  - `<input type="file" accept="image/jpeg,image/png,image/webp">`
  - 「画像をアップロード」ボタン
- テキストフォーム送信とは独立。Turnstileは新しいウィジェットを画像用にもう一つ生成 or 既存トークンを共有（既存と同じ`turnstileWidgetId`を流用すれば追加描画不要）

### 日記ページ（`render_entry`）

- `image_mime` がセットされていればテキスト本文の上に `<img src="/images/{date}" alt="">`
- レスポンシブのため `max-width: 100%; height: auto;` のスタイル指定

## テスト方針（TDD）

`src/image.rs` のユニットテスト：
- マジックバイトでのJPEG/PNG/WebPの判定
- 不正バイト列で `None`
- サイズ上限超過の検証

ハンドラレベルは既存パターンに合わせて手動QAで確認。

## 非対応（将来課題）

- 画像のバージョン履歴
- アーカイブ／RSSへの画像反映
- 画像最適化・サムネイル生成
- OG画像へのアップロード画像反映
- EXIF除去（今回は元ファイルをそのまま保存。気になる場合は将来対応）
