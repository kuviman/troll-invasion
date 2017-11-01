extern crate ws;

const PORT: u16 = 8008;

fn main() {
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
    let nick = nick.trim();
    ws::connect(format!("ws://play.kuviman.com:{}", PORT), |connection| {
        connection.send(format!("- {} connected", nick)).unwrap();
        std::thread::spawn(move || {
            loop {
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                connection.send(line).unwrap();
            }
        });
        |message| {
            println!("Received: {}", message);
            Ok(())
        }
    }).unwrap();
}
