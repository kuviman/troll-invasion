use ::*;

#[derive(Clone)]
pub struct Sender {
    nick: String,
    #[cfg(not(target_os = "emscripten"))]
    connection: Arc<Mutex<Option<ws::Sender>>>,
}

pub struct Receiver {
    receiver: std::sync::mpsc::Receiver<ServerMessage>,
}

pub fn connect(nick: &str, host: &str, port: u16) -> (Sender, Receiver) {
    let nick = nick.to_owned();
    let (sender, receiver) = std::sync::mpsc::channel();
    #[cfg(target_os = "emscripten")]
    return {
        let callback = web::Callback::from(move |addr: i32| {
            let line = unsafe { std::ffi::CStr::from_ptr(addr as *mut _).to_string_lossy() };
            sender.send(ServerMessage::parse(&line)).unwrap();
        });
        run_js! {
                TrollInvasion.connect(&*HOST.lock().unwrap(), &*PORT.lock().unwrap(), &nick, callback);
            }
        (Sender { nick }, Receiver { receiver })
    };
    #[cfg(not(target_os = "emscripten"))]
    return {
        let connection = Arc::new(Mutex::new(None));
        thread::spawn({
            let connection = connection.clone();
            let nick = nick.clone();
            let host = host.to_owned();
            move || {
                let address = format!("ws://{}:{}", host, port);
                eprintln!("Connecting to {}", address);
                ws::connect(address, |conn| {
                    struct Handler {
                        nick: String,
                        sender: std::sync::mpsc::Sender<ServerMessage>,
                        connection: Arc<Mutex<Option<ws::Sender>>>,
                        conn: ws::Sender,
                    }
                    impl ws::Handler for Handler {
                        fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
                            self.conn.send(format!("+{}", self.nick)).unwrap();
                            *self.connection.lock().unwrap() = Some(self.conn.clone());
                            Ok(())
                        }
                        fn on_message(&mut self, message: ws::Message) -> ws::Result<()> {
                            let message = message.into_text().unwrap();
                            eprintln!("{}", message);
                            let message = ServerMessage::parse(&message);
                            self.sender.send(message).unwrap();
                            Ok(())
                        }
                    }
                    Handler {
                        nick: nick.clone(),
                        sender: sender.clone(),
                        connection: connection.clone(),
                        conn,
                    }
                }).unwrap();
            }
        });
        (Sender { nick, connection }, Receiver { receiver })
    };
}

impl Sender {
    pub fn send<S: std::borrow::Borrow<str>>(&mut self, message: S) {
        #[cfg(target_os = "emscripten")]
        run_js! {
            TrollInvasion.send(message.borrow());
        }
        #[cfg(not(target_os = "emscripten"))]
        {
            if let Some(connection) = self.connection.lock().unwrap().as_ref() {
                connection.send(message.borrow()).unwrap();
            }
        };
    }
}

impl Receiver {
    pub fn try_recv(&self) -> Option<ServerMessage> {
        match self.receiver.try_recv() {
            Ok(msg) => Some(msg),
            _ => None,
        }
    }
}