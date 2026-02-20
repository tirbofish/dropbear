use parking_lot::Mutex;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub struct EucalyptusConsole {
    pub buffer: Arc<Mutex<Vec<String>>>,
    pub history: Vec<String>,

    pub show_info: bool,
    pub show_warning: bool,
    pub show_error: bool,
    pub show_debug: bool,
    pub show_trace: bool,
    pub auto_scroll: bool,
}

impl EucalyptusConsole {
    /// Creates a new instance of a [EucalyptusConsole], and
    pub fn new(port: Option<&str>) -> Self {
        let result = Self {
            buffer: Arc::new(Default::default()),
            history: vec![],
            show_info: true,
            show_warning: true,
            show_error: true,
            show_debug: false,
            show_trace: false,
            auto_scroll: true,
        };

        let buf_clone = result.buffer.clone();
        let addr = format!("127.0.0.1:{}", port.unwrap_or("56624"));
        std::thread::spawn(move || {
            let listener = TcpListener::bind(&addr).unwrap();

            log::info!("eucalyptus-editor debug console started at {}", addr);

            loop {
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            let buf_clone = buf_clone.clone();
                            std::thread::spawn(move || {
                                EucalyptusConsole::handle_client(stream, buf_clone)
                            });
                        }
                        Err(e) => {
                            eprintln!("Connection failed: {}", e);
                        }
                    }
                }
            }
        });

        result
    }

    fn handle_client(stream: TcpStream, buf: Arc<Mutex<Vec<String>>>) {
        let peer_addr = stream.peer_addr().unwrap();
        println!("New connection from: {}", peer_addr);

        let reader = BufReader::new(stream);

        for line in reader.lines() {
            match line {
                Ok(text) => {
                    buf.lock().push(text);
                }
                Err(e) => {
                    buf.lock()
                        .push(format!("Error reading from {}: {}", peer_addr, e));
                    break;
                }
            }
        }

        println!("Connection closed: {}", peer_addr);
    }

    /// Drains all from the thread-safe buffer and adds to the history, while returning that value.
    ///
    /// It is recommended to use this function.
    pub fn take(&mut self) -> Vec<String> {
        let buf = self.buffer.lock().drain(..).collect::<Vec<String>>();
        buf.iter().for_each(|v| self.history.push(v.clone()));
        buf
    }
}
