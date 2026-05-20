/// 画像アップロードのサイズ上限（3MB）
pub const MAX_IMAGE_SIZE: usize = 3 * 1024 * 1024;

/// バイト列の先頭からMIMEタイプを判定する。
/// JPEG / PNG / WebP のみをサポート。判定できない場合は None。
pub fn detect_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_jpeg() {
        let bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_image_mime(&bytes), Some("image/jpeg"));
    }

    #[test]
    fn detect_png() {
        let bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        assert_eq!(detect_image_mime(&bytes), Some("image/png"));
    }

    #[test]
    fn detect_webp() {
        let mut bytes = vec![0u8; 12];
        bytes[0..4].copy_from_slice(b"RIFF");
        bytes[8..12].copy_from_slice(b"WEBP");
        assert_eq!(detect_image_mime(&bytes), Some("image/webp"));
    }

    #[test]
    fn reject_gif() {
        let bytes = b"GIF89a\x00\x00";
        assert_eq!(detect_image_mime(bytes), None);
    }

    #[test]
    fn reject_text() {
        let bytes = b"hello world";
        assert_eq!(detect_image_mime(bytes), None);
    }

    #[test]
    fn reject_empty() {
        assert_eq!(detect_image_mime(&[]), None);
    }

    #[test]
    fn reject_riff_but_not_webp() {
        let mut bytes = vec![0u8; 12];
        bytes[0..4].copy_from_slice(b"RIFF");
        bytes[8..12].copy_from_slice(b"WAVE");
        assert_eq!(detect_image_mime(&bytes), None);
    }

    #[test]
    fn max_size_is_3mb() {
        assert_eq!(MAX_IMAGE_SIZE, 3 * 1024 * 1024);
    }
}
