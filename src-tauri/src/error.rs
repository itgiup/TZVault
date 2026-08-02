// src-tauri/src/error.rs
//
// QUY TẮC KIẾN TRÚC: mọi lỗi trả từ Rust ra frontend (qua Tauri command)
// PHẢI là mã lỗi ổn định (VD "ERR_INVALID_PASSWORD"), không bao giờ là
// câu prose (dù tiếng Anh hay Việt) — vì frontend là nơi duy nhất quyết
// định hiển thị ngôn ngữ nào (xem src/i18n/translations.ts).
//
// Với các lỗi "không nên xảy ra" (DB hỏng, mutex poisoned, encode/decode
// lỗi...) — không có action cụ thể nào người dùng có thể làm để tự sửa,
// nên không cần mã lỗi riêng cho từng loại. Thay vào đó: log chi tiết ra
// stderr để dev debug, còn trả về đúng 1 mã chung ERR_INTERNAL cho UI.
// Cách này vừa giữ đúng kiến trúc i18n, vừa tránh rò rỉ chi tiết nội bộ
// (đường dẫn file, cấu trúc DB...) ra giao diện.

pub const ERR_INTERNAL: &str = "ERR_INTERNAL";

/// Dùng khi có lỗi gốc (implement Display) muốn log lại, ví dụ:
///   conn.execute(...).map_err(|e| internal_error("db_insert_key", e))?;
pub fn internal_error<E: std::fmt::Display>(context: &str, err: E) -> String {
    eprintln!("[internal error] {context}: {err}");
    ERR_INTERNAL.to_string()
}

/// Dùng khi không có lỗi gốc cụ thể, chỉ có ngữ cảnh mô tả, ví dụ:
///   .ok_or_else(|| internal_error_msg("nonce_length_mismatch"))?;
pub fn internal_error_msg(context: &str) -> String {
    eprintln!("[internal error] {context}");
    ERR_INTERNAL.to_string()
}
