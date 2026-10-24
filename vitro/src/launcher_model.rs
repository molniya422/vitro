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
    todo!("query parsing is implemented in the launcher adapter")
}

pub(crate) fn append_utf16_unit(
    buffer: &mut String,
    pending_high_surrogate: &mut Option<u16>,
    unit: u16,
) {
    todo!("UTF-16 input handling is implemented in the launcher adapter")
}

pub(crate) fn cap_command_output(bytes: &[u8], limit: usize) -> String {
    todo!("command output handling is implemented in the launcher adapter")
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
