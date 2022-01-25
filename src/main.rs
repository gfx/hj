use lazy_static::lazy_static;
use std::error::Error;
use std::io::{Read, Stdin, Write};


use clap::Parser;
use regex::Regex;
use tinyjson::JsonValue;

#[macro_use(defer)]
extern crate scopeguard;

#[derive(Parser, Debug)]
#[clap(about = "Converts HTTP/1 style HTTP responses into JSON.
This command can parses the output of `curl -sv ...` or `h2o-httpclient ...`", long_about = None)]
struct Cli {
    /// Stop parsing contents according to its corresponding content-type.
    #[clap(long, short)]
    raw: bool,

    /// Wrap responses with a JSON array, assuming multiple responses.
    #[clap(long, short)]
    array: bool,
}

struct LineBufferedStdin {
    reader: Stdin,

    buffer_stack: Vec<String>,
}

impl LineBufferedStdin {
    fn read_line(&mut self) -> Result<String, std::io::Error> {
        if let Some(line) = self.buffer_stack.pop() {
            return Ok(line);
        }

        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        return Ok(line);
    }

    fn unread_line(&mut self, line: String) {
        self.buffer_stack.push(line);
    }

    fn consume_buffer_stack(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        while let Some(line) = self.buffer_stack.pop() {
            buf.extend(line.as_bytes());
        }
        return buf;
    }

    fn read(&mut self, size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buf1 = self.consume_buffer_stack();
        let mut buf2 = vec![0; size - buf1.len()];
        self.reader.read_exact(&mut buf2)?;
        buf1.extend(buf2);
        return Ok(buf1);
    }

    fn read_to_end(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut buf1 = self.consume_buffer_stack();
        let mut buf2 = Vec::new();
        self.reader.read_to_end(&mut buf2)?;
        buf1.extend(buf2);
        return Ok(buf1);
    }

    fn is_eof(&mut self) -> bool {
        return match self.read_line() {
            Ok(line) => {
                if line.is_empty() {
                    true
                } else {
                    self.unread_line(line);
                    false
                }
            },
            Err(_) => true,
        };
    }
}

// Some sort of unrelated stuff are accepted.
// * warnings by UndefinedBehaviorSanitizer may be injected to outputs if the client is built with UBSan.
// * curl outputs extra stuff when -v, while hj requires -v for response headers
fn skip(lbin: &mut LineBufferedStdin) -> Result<(), Box<dyn Error>> {
    // UB warnings look like this:
    // ```
    // /path/to/file.c:80:34: runtime error: blah blah blah
    // SUMMARY: UndefinedBehaviorSanitizer: undefined-behavior /path/to/file.c:80:34 in
    // ```

    loop {
        match lbin.read_line() {
            Ok(line) => {
                // UB warnings
                lazy_static! {
                    static ref PAT1: Regex =
                        Regex::new(r"^[^:]+:[0-9]+:[0-9]+: runtime error:").unwrap();
                    static ref PAT2: Regex = Regex::new(r"^SUMMARY: [a-zA-Z0-9_-]+:").unwrap();
                }
                if PAT1.is_match(&line) || PAT2.is_match(&line) {
                    continue;
                }

                // curl's TLS messages and request headers
                if line.starts_with("* ") || line.starts_with("> ") || line.starts_with("{ ") || line.starts_with("} ") {
                    continue;
                }

                lbin.unread_line(line);
                return Ok(());
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }
}

fn parse_status_line(lbin: &mut LineBufferedStdin) -> Result<(), Box<dyn Error>> {
    // e.g. "HTTP/1.1 200 OK" or "HTTP/3 200"
    lazy_static! {
        static ref PAT: Regex =
            Regex::new(r"^(?:< )?HTTP/(?P<version>[0-9]+(?:\.[0-9]+)?) (?P<status>[0-9]+)").unwrap();
    }

    match lbin.read_line() {
        Ok(line) => {
            if let Some(caps) = PAT.captures(&line) {
                let http_version = caps.name("version").unwrap().as_str();
                let status_code = caps.name("status").unwrap().as_str();

                print!("\"http_version\":\"{http_version}\",\"status_code\":{status_code}");
                return Ok(());
            }
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                std::format!("Invalid status line: {}", str_to_json_string(&line)),
            )));
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    }
}

fn str_to_json_string(s: &str) -> String {
    return JsonValue::String(s.to_string()).stringify().unwrap();
}

fn parse_header_fields(
    lbin: &mut LineBufferedStdin,
    content_type: &mut Option<String>,
    content_length: &mut Option<usize>,
) -> Result<(), Box<dyn Error>> {
    std::io::stdout().write_all(b",\"headers\":{")?;
    defer! {
        let _ = std::io::stdout().write_all(b"}");
    }

    // e.g. "Content-Type: text/html"
    lazy_static! {
        static ref PAT: Regex = Regex::new(r"^(?:< )?(?P<name>[^:]+):(?P<value>.+)").unwrap();
    }
    let mut initial = true;
    loop {
        match lbin.read_line() {
            Ok(line) => {
                match line.trim() {
                    "" |  "<" => {
                        return Ok(());
                    },
                    _ => {}
                }
                if let Some(caps) = PAT.captures(&line) {
                    let raw_name = caps.name("name").unwrap().as_str().trim();
                    let raw_value = caps.name("value").unwrap().as_str().trim();
                    let name = str_to_json_string(raw_name);
                    let value = str_to_json_string(raw_value);
                    if initial {
                        initial = false;
                    } else {
                        std::io::stdout().write_all(b",")?
                    }
                    print!("{name}:{value}");

                    if raw_name.eq_ignore_ascii_case("content-type") {
                        *content_type = Some(raw_value.to_string());
                    } else if raw_name.eq_ignore_ascii_case("content-length") {
                        if let Ok(len) = raw_value.parse::<usize>() {
                            *content_length = Some(len);
                        }
                    }
                } else {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        std::format!("Invalid header field: {}", str_to_json_string(&line)),
                    )));
                }
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }
}

fn parse_content_raw(
    lbin: &mut LineBufferedStdin,
    content_length: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    let buf = if let Some(len) = content_length {
        lbin.read(len)?
    } else {
        lbin.read_to_end()?
    };
    // TODO: handle binary data
    let content = str_to_json_string(&String::from_utf8_lossy(&buf));
    print!(",\"content\":{content}");
    return Ok(());
}

fn parse_content(
    lbin: &mut LineBufferedStdin,
    content_type: Option<String>,
    content_length: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    // FIXME: parse more complicated media types, e.g. "application/vnd.github+json; charset=utf-8"
    if content_type.is_some()
        && content_type
            .unwrap()
            .eq_ignore_ascii_case("application/json")
    {
        let buf = if let Some(len) = content_length {
            lbin.read(len)?
        } else {
            lbin.read_to_end()?
        };
        let content = String::from_utf8_lossy(&buf).to_string();
        let json_value: JsonValue = content.parse().unwrap();
        let json_stringified = json_value.stringify().unwrap();

        print!(",\"content\":{json_stringified}");
    } else {
        parse_content_raw(lbin, content_length)?;
    }

    return Ok(());
}

fn process_response(cli: &Cli, lbin: &mut LineBufferedStdin) -> Result<(), Box<dyn Error>> {
    std::io::stdout().write(b"{")?;

    skip(lbin)?;
    parse_status_line(lbin)?;

    skip(lbin)?;

    let mut content_type: Option<String> = None;
    let mut content_length: Option<usize> = None;
    parse_header_fields(lbin, &mut content_type, &mut content_length)?;

    skip(lbin)?;

    if cli.raw {
        parse_content_raw(lbin, content_length)?;
    } else {
        parse_content(lbin, content_type, content_length)?;
    }
    skip(lbin)?;

    std::io::stdout().write(b"}")?;

    return Ok(());
}

fn main() {
    let cli = Cli::parse();

    let mut lbin = LineBufferedStdin {
        reader: std::io::stdin(),
        buffer_stack: Vec::new(),
    };

    let mut initial = true;
    if cli.array {
        std::io::stdout().write(b"[").unwrap();
    }
    loop {
        if initial {
            initial = false;
        } else {
            if cli.array {
                std::io::stdout().write(b",").unwrap();
            }
        }
        if let Err(err) = process_response(&cli, &mut lbin) {
            eprintln!("Error: {err}");
            break;
        }
        if !cli.array {
            std::io::stdout().write(b"\n").unwrap();
        }
        if lbin.is_eof() {
            break;
        }
    }
    if cli.array {
        std::io::stdout().write(b"]\n").unwrap();
    }
}
