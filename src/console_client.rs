use ::*;

pub fn run(server: &str, port: u16) {
    ws::connect(format!("ws://{}:{}", server, port), |connection| {
        struct Handler { nick: String, connection: ws::Sender }
        impl ws::Handler for Handler {
            fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
                print!("nick: ");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                let mut nick = String::new();
                std::io::stdin().read_line(&mut nick).unwrap();
                let nick = nick.trim().to_owned();
                self.nick = nick.clone();

                let connection = self.connection.clone();
                connection.send(format!("+{}", nick)).unwrap();

                std::thread::spawn(move || {
                    loop {
                        let mut line = String::new();
                        std::io::stdin().read_line(&mut line).unwrap();
                        connection.send(format!("{}:{}", nick, line.trim())).unwrap();
                    }
                });

                Ok(())
            }
            fn on_message(&mut self, message: ws::Message) -> ws::Result<()> {
                println!("{}", message.into_text().unwrap());
                Ok(())
            }
        }
        Handler { nick: String::from("<unnamed>"), connection }
    }).unwrap();
}