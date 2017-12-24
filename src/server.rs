use ::*;

pub fn run(port: u16) {
    let mut jar = std::process::Command::new("java");
    jar.arg("-jar").arg(std::path::Path::new("..").join("target").join("troll-invasion.jar"));
    jar.stdin(std::process::Stdio::piped());
    jar.stdout(std::process::Stdio::piped());
    let jar = jar.spawn().expect("Failed to start troll-invasion.jar");
    let jar_in = std::rc::Rc::new(std::cell::RefCell::new(jar.stdin.unwrap()));
    let mut jar_out = std::io::BufReader::new(jar.stdout.unwrap());
    let connections: std::sync::Arc<std::sync::Mutex<HashMap<String, ws::Sender>>> = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
    std::thread::spawn({
        let connections = connections.clone();
        move || {
            let mut line = String::new();
            loop {
                use std::io::BufRead;
                line.clear();
                if jar_out.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let line = line.trim_right_matches('\n').trim_right_matches('\r');
                eprintln!("> {}", line);
                let (nicks, message) = line.split_at(line.find(':').expect("No ':' found"));
                let message = message[1..].trim();
                for nick in nicks.split(',') {
                    if let Some(connection) = connections.lock().unwrap().get(nick) {
                        connection.send(message).unwrap();
                    }
                }
            }
        }
    });
    ws::listen(("0.0.0.0", port), move |connection| {
        let jar_in = jar_in.clone();
        let nick = RefCell::new(String::new());
        let connections = connections.clone();
        move |message: ws::Message| {
            let mut message = message.into_text().unwrap();
            if message.starts_with('+') {
                assert!(nick.borrow().len() == 0);
                *nick.borrow_mut() = message[1..].to_owned();
                connections.lock().unwrap().insert(nick.borrow().clone(), connection.clone());
            } else {
                assert!(nick.borrow().len() != 0);
                message = format!("{}:{}", &nick.borrow(), message);
            }
            eprintln!("< {}", message);
            use std::io::Write;
            let mut jar_in = jar_in.borrow_mut();
            jar_in.write(message.as_bytes()).unwrap();
            jar_in.write("\n".as_bytes()).unwrap();
            jar_in.flush().unwrap();
            Ok(())
        }
    }).unwrap();
}