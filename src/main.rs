use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use mem_server_demo::ThreadPool;

type PtrType = *mut u8;
struct CounterPtr(PtrType);

unsafe impl Send for CounterPtr {}

const ADDR: &'static str = "127.0.0.1";
const PORT: usize = 8080;

const DEFAULT_MARKUP: &str = include_str!("../static/index.html");

fn main() {
    let listener = TcpListener::bind(format!("{ADDR}:{PORT}")).unwrap();
    let pool = ThreadPool::new(4);

    let counter = Arc::new(0);
    let raw_counter = Arc::into_raw(counter) as PtrType;

    for stream in listener.incoming().take(5) {
        let stream = stream.unwrap();

        let counter_ptr = CounterPtr(raw_counter);

        pool.execute(|| {
            handle_connection(stream, counter_ptr);
        });
    }
}

/// Handles a connection, serving the appropriate page content.
fn handle_connection(mut stream: TcpStream, counter: CounterPtr) {
    let buf_reader = BufReader::new(&stream);
    let req_line = buf_reader.lines().next().unwrap().unwrap();

    // Get the response status and page contents.
    unsafe {
        let (status, contents) = match &req_line[..] {
            "GET / HTTP/1.1" => (
                HttpStatus::Ok,
                String::from(DEFAULT_MARKUP).replace("{count}", (*counter.0).to_string().as_str()),
            ),
            "GET /incr HTTP/1.1" => {
                *counter.0 += 1;
                (
                    HttpStatus::Ok,
                    String::from(DEFAULT_MARKUP)
                        .replace("{count}", (*counter.0).to_string().as_str()),
                )
            }
            _ => (
                HttpStatus::NotFound,
                String::from("<h1>Page not found.</h1>"),
            ),
        };
        // Get the contents of the file to respond with and create the response
        let res = format!(
            "{status}\r\nContent-Length: {length}\r\n\r\n{contents}",
            length = contents.len()
        );
        stream.write_all(res.as_bytes()).unwrap();
    }
}

/// The response status for an HTTP request.
enum HttpStatus {
    Ok,
    NotFound,
}

impl std::fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            Self::Ok => "200 OK",
            Self::NotFound => "404 NOT FOUND",
        };
        write!(f, "HTTP/1.1 {code}")
    }
}
