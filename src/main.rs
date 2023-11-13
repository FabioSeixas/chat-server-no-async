use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{self, TcpListener},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

struct ChannelMessage {
    origin: String,
    message: String,
}

fn main() {
    let listener = TcpListener::bind("localhost:8080").unwrap();
    let mut _clients: HashMap<String, net::TcpStream> = HashMap::new();
    let clients = Arc::new(Mutex::new(_clients));

    let (sender, receiver): (Sender<ChannelMessage>, Receiver<ChannelMessage>) = mpsc::channel();

    let clients_clone = clients.clone();

    thread::spawn(move || {
        for message in receiver.iter() {
            println!("message from {}: {}", message.origin, message.message);
            for (client_id, mut client_stream) in clients.lock().unwrap().iter() {
                if *client_id != message.origin {
                    client_stream
                        .write_all(format!(
                            "{}: {}",
                            message.origin,
                            message.message
                        ).as_bytes())
                        .unwrap();
                }
            }
        }
    });

    for connection in listener.incoming() {
        match connection {
            Ok(mut stream) => {
                clients_clone.lock().unwrap().insert(
                    stream.peer_addr().unwrap().port().to_string(),
                    stream.try_clone().unwrap(),
                );
                let new_sender = sender.clone();
                thread::spawn(move || loop {
                    let mut buf: [u8; 64] = [0; 64];
                    stream.read(&mut buf).unwrap();
                    new_sender
                        .send(ChannelMessage {
                            message: String::from(std::str::from_utf8(&buf).unwrap()),
                            origin: stream.peer_addr().unwrap().port().to_string(),
                        })
                        .unwrap();
                });
            }
            Err(err) => {
                eprintln!("ERROR: {err}");
            }
        };
    }
}
