use std::io::Read;

use std::io::Stdin;

pub(crate) struct LineBufferedStdin {
    pub(crate) reader: Stdin,

    pub(crate) buffer_stack: Vec<String>,
}

impl LineBufferedStdin {
    pub(crate) fn read_line(&mut self) -> Result<String, std::io::Error> {
        if let Some(line) = self.buffer_stack.pop() {
            return Ok(line);
        }

        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        return Ok(line);
    }

    pub(crate) fn unread_line(&mut self, line: String) {
        self.buffer_stack.push(line);
    }

    pub(crate) fn consume_buffer_stack(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        while let Some(line) = self.buffer_stack.pop() {
            buf.extend(line.as_bytes());
        }
        return buf;
    }

    pub(crate) fn read(&mut self, size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buf1 = self.consume_buffer_stack();
        let mut buf2 = vec![0; size - buf1.len()];
        self.reader.read_exact(&mut buf2)?;
        buf1.extend(buf2);
        return Ok(buf1);
    }

    pub(crate) fn read_to_end(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut buf1 = self.consume_buffer_stack();
        let mut buf2 = Vec::new();
        self.reader.read_to_end(&mut buf2)?;
        buf1.extend(buf2);
        return Ok(buf1);
    }

    pub(crate) fn is_eof(&mut self) -> bool {
        return match self.read_line() {
            Ok(line) => {
                if line.is_empty() {
                    true
                } else {
                    self.unread_line(line);
                    false
                }
            }
            Err(_) => true,
        };
    }
}
