//! 启动器输入与命令模型。
//!
//! 这里不依赖 Win32/D2D，窗口层只负责把消息翻译成这些纯函数的调用。

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum QueryMode {
    Apps(String),
    Command(String),
    Calculation(String),
}

pub(crate) fn parse_query_mode(query: &str) -> QueryMode {
    if let Some(rest) = query.strip_prefix('>') {
        QueryMode::Command(rest.trim().to_owned())
    } else if let Some(rest) = query.strip_prefix('=') {
        QueryMode::Calculation(rest.trim().to_owned())
    } else {
        QueryMode::Apps(query.to_lowercase())
    }
}

pub(crate) fn append_utf16_unit(
    buffer: &mut String,
    pending_high_surrogate: &mut Option<u16>,
    unit: u16,
) {
    const HIGH_START: u16 = 0xD800;
    const HIGH_END: u16 = 0xDBFF;
    const LOW_START: u16 = 0xDC00;
    const LOW_END: u16 = 0xDFFF;

    if (HIGH_START..=HIGH_END).contains(&unit) {
        if let Some(previous) = pending_high_surrogate.replace(unit) {
            buffer.push(char::REPLACEMENT_CHARACTER);
            *pending_high_surrogate = Some(unit);
            if !(HIGH_START..=HIGH_END).contains(&previous) {
                *pending_high_surrogate = Some(unit);
            }
        }
        return;
    }

    if (LOW_START..=LOW_END).contains(&unit) {
        if let Some(high) = pending_high_surrogate.take() {
            let scalar = 0x1_0000
                + (((high - HIGH_START) as u32) << 10)
                + (unit - LOW_START) as u32;
            if let Some(ch) = char::from_u32(scalar) {
                buffer.push(ch);
            } else {
                buffer.push(char::REPLACEMENT_CHARACTER);
            }
        } else {
            buffer.push(char::REPLACEMENT_CHARACTER);
        }
        return;
    }

    if pending_high_surrogate.take().is_some() {
        buffer.push(char::REPLACEMENT_CHARACTER);
    }
    if let Some(ch) = char::from_u32(unit as u32) {
        buffer.push(ch);
    }
}

pub(crate) fn cap_command_output(bytes: &[u8], limit: usize) -> String {
    String::from_utf8_lossy(bytes).chars().take(limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_command_prefix_without_lowercasing_payload() {
        assert_eq!(
            parse_query_mode("> Get-ChildItem"),
            QueryMode::Command("Get-ChildItem".into())
        );
    }

    #[test]
    fn parses_unicode_app_search_as_lowercase() {
        assert_eq!(
            parse_query_mode("  Notepad"),
            QueryMode::Apps("  notepad".into())
        );
    }

    #[test]
    fn appends_bmp_utf16_input_without_truncating_unicode() {
        let mut text = String::new();
        let mut pending = None;
        append_utf16_unit(&mut text, &mut pending, '中' as u16);
        assert_eq!(text, "中");
        assert_eq!(pending, None);
    }

    #[test]
    fn combines_surrogate_pair_input() {
        let mut text = String::new();
        let mut pending = None;
        append_utf16_unit(&mut text, &mut pending, 0xD83D);
        append_utf16_unit(&mut text, &mut pending, 0xDE80);
        assert_eq!(text, "🚀");
    }

    #[test]
    fn caps_command_output_on_character_boundary() {
        assert_eq!(cap_command_output("a🚀b".as_bytes(), 2), "a🚀");
    }
}
