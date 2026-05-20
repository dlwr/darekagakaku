-- 添付画像のMIMEタイプ列を追加（NULLなら画像なし）
ALTER TABLE diary_entries ADD COLUMN image_mime TEXT;
