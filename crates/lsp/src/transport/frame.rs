use std::io::{BufRead, Write};

pub const CONTENT_LENGTH_HEADER: &str = "Content-Length";
pub const HEADER_BODY_SEPARATOR: &str = "\r\n\r\n";
pub const MAX_HEADER_LINE_BYTES: usize = 4096;
pub const MAX_CONTENT_BYTES: usize = 16 * 1024 * 1024;

pub fn read_message<R: BufRead>(reader: &mut R) -> std::io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            if content_length.is_none() {
                return Ok(None);
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "连接在消息中间被关闭",
            ));
        }
        if line.len() > MAX_HEADER_LINE_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "头行超过长度上限",
            ));
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        let (name, value) = match trimmed.split_once(':') {
            Some((n, v)) => (n.trim(), v.trim()),
            None => continue,
        };
        if name.eq_ignore_ascii_case(CONTENT_LENGTH_HEADER) {
            content_length = value
                .parse::<usize>()
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Content-Length 不是合法数字")
                })
                .map(Some)?;
        }
    }
    let length = match content_length {
        Some(len) => len,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "缺少 Content-Length 头",
            ))
        }
    };
    if length > MAX_CONTENT_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "消息体超过长度上限",
        ));
    }
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let body = String::from_utf8(body).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "消息体不是合法 UTF-8")
    })?;
    Ok(Some(body))
}

pub fn write_message<W: Write>(writer: &mut W, body: &str) -> std::io::Result<()> {
    let length = body.as_bytes().len();
    let header = format!("Content-Length: {length}\r\n\r\n");
    writer.write_all(header.as_bytes())?;
    writer.write_all(body.as_bytes())?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_single_frame() {
        let frame = "Content-Length: 24\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1}";
        let mut reader = Cursor::new(frame.as_bytes());
        let body = read_message(&mut reader)
            .expect("读取帧失败")
            .expect("不应是 EOF");
        assert_eq!(body, "{\"jsonrpc\":\"2.0\",\"id\":1}");
    }

    #[test]
    fn write_then_read_roundtrip() {
        let body = "{\"jsonrpc\":\"2.0\",\"method\":\"initialized\"}";
        let mut buf = Vec::new();
        write_message(&mut buf, body).expect("写帧失败");
        let mut reader = Cursor::new(buf);
        let read_back = read_message(&mut reader)
            .expect("读帧失败")
            .expect("不应是 EOF");
        assert_eq!(read_back, body);
    }

    #[test]
    fn read_two_consecutive_frames() {
        let frame1 = "Content-Length: 5\r\n\r\nhello";
        let frame2 = "Content-Length: 6\r\n\r\nworld!";
        let mut reader = Cursor::new(format!("{frame1}{frame2}").into_bytes());
        assert_eq!(read_message(&mut reader).expect("读帧失败").unwrap(), "hello");
        assert_eq!(read_message(&mut reader).expect("读帧失败").unwrap(), "world!");
        assert!(read_message(&mut reader).expect("读帧失败").is_none());
    }

    #[test]
    fn missing_content_length_is_error() {
        let frame = "Content-Type: application/json\r\n\r\n{}";
        let mut reader = Cursor::new(frame.as_bytes());
        assert!(read_message(&mut reader).is_err());
    }

    #[test]
    fn oversized_header_is_error() {
        let mut frame = String::from("Content-Length: 1\r\n");
        frame.push_str(&"x".repeat(MAX_HEADER_LINE_BYTES + 1));
        frame.push_str("\r\n\r\n{}");
        let mut reader = Cursor::new(frame.into_bytes());
        assert!(read_message(&mut reader).is_err());
    }

    #[test]
    fn oversized_content_is_error() {
        let mut frame = format!("Content-Length: {}\r\n\r\n", MAX_CONTENT_BYTES + 1);
        frame.push_str("{}");
        let mut reader = Cursor::new(frame.into_bytes());
        assert!(read_message(&mut reader).is_err());
    }

    #[test]
    fn header_case_insensitive() {
        let frame = "content-length: 2\r\n\r\n{}";
        let mut reader = Cursor::new(frame.as_bytes());
        let body = read_message(&mut reader).expect("读取帧失败").unwrap();
        assert_eq!(body, "{}");
    }

    #[test]
    fn eof_at_start_returns_none() {
        let mut reader = Cursor::new(Vec::new());
        assert!(read_message(&mut reader).expect("读帧失败").is_none());
    }

    #[test]
    fn unicode_body_length_in_bytes() {
        let body = "{\"text\":\"你好\"}";
        let mut buf = Vec::new();
        write_message(&mut buf, body).expect("写帧失败");
        let header = String::from_utf8(buf.clone()).expect("头是 UTF-8");
        assert!(header.starts_with(&format!(
            "Content-Length: {}\r\n\r\n",
            body.as_bytes().len()
        )));
        let mut reader = Cursor::new(buf);
        assert_eq!(read_message(&mut reader).expect("读帧失败").unwrap(), body);
    }
}
