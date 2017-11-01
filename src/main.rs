extern crate ws;
extern crate env_logger;

const PORT: u16 = 8008;

fn main() {
    env_logger::init().unwrap();

    std::thread::spawn(|| {
        use std::cell::RefCell;
        let connections: RefCell<Vec<ws::Sender>> = RefCell::new(Vec::new());
        ws::listen(("0.0.0.0", PORT), |connection| {
            connections.borrow_mut().push(connection);
            |message: ws::Message| {
                for connection in connections.borrow().iter() {
                    connection.send(message.clone()).unwrap();
                }
                Ok(())
            }
        }).unwrap();
    });

    println!("nick: ");
    let mut nick = String::new();
    std::io::stdin().read_line(&mut nick).unwrap();
    let nick = nick.trim().to_owned();
    ws::connect(format!("ws://play.kuviman.com:{}", PORT), |connection| {
        std::thread::spawn({
            let nick = nick.clone();
            move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                connection.send(format!("- {} connected", nick)).unwrap();
                loop {
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line).unwrap();
                    connection.send(line).unwrap();
                }
            }
        });
        |message: ws::Message| {
            println!("{}: {}", nick, message.into_text().unwrap().trim());
            Ok(())
        }
    }).unwrap();
}
