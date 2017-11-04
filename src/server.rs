use ::*;

pub fn run(port: u16) {
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
                if jar_out.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let line = line.trim_right_matches('\n').trim_right_matches('\r');
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
            let mut jar_in = jar_in.borrow_mut();
            jar_in.write(message.as_bytes()).unwrap();
            jar_in.write("\n".as_bytes()).unwrap();
            jar_in.flush().unwrap();
            Ok(())
        }
    }).unwrap();
}