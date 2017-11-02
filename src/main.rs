extern crate ws;
extern crate env_logger;
extern crate argparse;

fn server(port: u16) {
    let mut jar = std::process::Command::new("java");
    jar.arg("-jar").arg("TrollInvasion.jar");
    jar.stdin(std::process::Stdio::piped());
    jar.stdout(std::process::Stdio::piped());
    let jar = jar.spawn().expect("Failed to start TrollInvasion.jar");
    let jar_in = std::rc::Rc::new(std::cell::RefCell::new(jar.stdin.unwrap()));
    let mut jar_out = std::io::BufReader::new(jar.stdout.unwrap());
    let connections: std::sync::Arc<std::sync::Mutex<Vec<ws::Sender>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    std::thread::spawn({
        let connections = connections.clone();
        move || {
            let mut line = String::new();
            loop {
                use std::io::BufRead;
                line.clear();
                jar_out.read_line(&mut line).unwrap();
                let line = line.trim_right_matches('\n');
                eprintln!("> {}", line);
                for connection in connections.lock().unwrap().iter() {
                    connection.send(line).unwrap();
                }
            }
        }
    });
    ws::listen(("0.0.0.0", port), move |connection| {
        connections.lock().unwrap().push(connection.clone());
        let jar_in = jar_in.clone();
        move |message: ws::Message| {
            let message = message.into_text().unwrap();
            eprintln!("< {}", message);
            use std::io::Write;
            jar_in.borrow_mut().write(message.as_bytes()).unwrap();
            Ok(())
        }
    }).unwrap();
}

fn client(server: &str, port: u16) {
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
                        connection.send(format!("{}: {}", nick, line.trim())).unwrap();
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

fn main() {
    env_logger::init().unwrap();

    let mut port: u16 = 8008;
    let mut host = None;
    let mut start_server = false;

    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("TrollInvasion client/server. By default starts client connecting to play.kuviman.com.");
        ap.refer(&mut port).add_option(&["-p", "--port"], argparse::Store, "Specify port");
        ap.refer(&mut host).add_option(&["-c", "--connect"], argparse::StoreOption, "Start client, connect to specified host");
        ap.refer(&mut start_server).add_option(&["-s", "--server"], argparse::StoreTrue, "Start server");
        ap.parse_args_or_exit();
    }

    if start_server {
        if host.is_some() {
            std::thread::spawn(move || { server(port) });
        } else {
            server(port);
        }
    } else {
        host = Some(String::from("play.kuviman.com"));
    }
    if let Some(host) = host {
        if start_server {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        client(&host, port);
    }
}
