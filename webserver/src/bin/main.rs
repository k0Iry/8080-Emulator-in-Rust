use std::ffi::OsStr;
use std::fs;
use std::io::copy;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use webserver::ThreadPool;
use std::thread;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:9000").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn read_line<T>(reader: &mut BufReader<T>) -> Result<String, Error>
where
    T: Read,
{
    let mut line = String::new();
    loop {
        match reader.read_line(&mut line) {
            Ok(_) => break,
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                // eprintln!("EWOULDBLOCK occured on non-blocking IO");
                thread::yield_now()
            },
            Err(e) => return Err(e)
        }
    }
    Ok(line)
}

fn content_type(filename: &str) -> &str {
    if let Some(extension) = Path::new(filename).extension().and_then(OsStr::to_str) {
        if extension == "html" {
            return "Content-Type: text/html";
        } else if extension == "mkv" {
            return "Content-Type: video/x-matroska";
        } else if extension == "jpg" || extension == "jpeg" {
            return "Content-Type: image/jpeg";
        } else if extension == "css" {
            return "Content-Type: text/css";
        }
    }
    "Content-Type: application/octet-stream"
}

fn handle_connection(mut stream: TcpStream) {
    // need to be mutable, the low-level offset of socket fd is changed
    let mut reader = BufReader::new(&stream);

    stream.set_nonblocking(true).unwrap();

    if let Ok(request_line) = read_line(&mut reader) {
        println!("HTTP request line read: {}", request_line);

        let mut iter = request_line.split_whitespace();
        // MATCH: method(GET,POST...) uri(/index.html...) version(which we don't care)
        let (method, uri) = (iter.next(), iter.next());
        if None == method || None == uri {
            return;
        }

        let method = method.unwrap();
        let mut uri = uri.unwrap();

        // println!("HTTP Request Headers:");
        // loop {
        //     let request_header = read_line(&mut reader);
        //     if request_header == "\r\n" {
        //         break
        //     }
        //     println!("{}", request_header)
        // }

        drop(reader);

        let (status_line, filename) = if method == "GET" {
            if uri == "/" {
                ("HTTP/1.1 200 OK", "index.html")
            } else {
                uri = &uri[1..];
                match fs::metadata(Path::new(uri)) {
                    Ok(_) => ("HTTP/1.1 200 OK", uri),
                    Err(_) => ("HTTP/1.1 404 NOT FOUND", "404.html"),
                }
            }
        } else {
            ("HTTP/1.1 501 NOT IMPLEMENTED", "404.html")
        };

        let mut file = std::fs::File::open(filename).unwrap();
        let response_headers = format!(
            "{}\r\nContent-Length: {}\r\n{}\r\n\r\n",
            status_line,
            file.metadata().unwrap().len(),
            content_type(filename)
        );

        stream.set_nonblocking(false).unwrap();

        if let Ok(_) = stream.write_all(response_headers.as_bytes()) {
            copy(&mut file, &mut stream).unwrap_or_else(|e| {
                eprintln!("{:?}", e.kind());
                0
            });
        }

        stream.flush().unwrap();
    }
}
