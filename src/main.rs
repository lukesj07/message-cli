use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} [server|client] [address:port]", args[0]);
        return;
    }

    let mode = &args[1];
    let address = &args[2];

    if mode == "server" {
        start_server(address).unwrap();
    } else if mode == "client" {
        start_client(address).unwrap();
    } else {
        eprintln!("Invalid mode. Use 'server' or 'client'.");
    }
}

fn start_server(address: &str) -> io::Result<()> {
    let listener = TcpListener::bind(address)?;
    let clients = Arc::new(Mutex::new(Vec::new()));

    println!("Server running on {}", address);

    for stream in listener.incoming() {
        let stream = stream?;
        let clients = Arc::clone(&clients);

        thread::spawn(move || handle_client(stream, clients));
    }

    Ok(())
}

fn handle_client(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) {
    let address = stream.peer_addr().unwrap();
    println!("Client connected: {}", address);

    let mut clients_locked = clients.lock().unwrap();
    clients_locked.push(stream.try_clone().unwrap());
    drop(clients_locked);

    let reader = io::BufReader::new(stream.try_clone().unwrap());

    for line in reader.lines() {
        let line = line.unwrap();
        if line.to_lowercase() == "exit" {
            println!("Client {} disconnected", address);
            break;
        }
        println!("Received from {}: {}", address, line);

        let clients_locked = clients.lock().unwrap();
        for mut client in clients_locked.iter() {
            if client.peer_addr().unwrap() != address {
                let _ = writeln!(client, "{}: {}", address, line);
            }
        }
    }

    let mut clients_locked = clients.lock().unwrap();
    clients_locked.retain(|client| client.peer_addr().unwrap() != address);
    println!("Client {} removed", address);
}

fn start_client(address: &str) -> io::Result<()> {
    let stream = TcpStream::connect(address)?;
    println!("Connected to {}", address);

    let mut stream_clone = stream.try_clone().unwrap();

    thread::spawn(move || loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() == "exit" {
            writeln!(stream_clone, "exit").unwrap();
            break;
        }

        writeln!(stream_clone, "{}", input.trim()).unwrap();
    });

    let reader = io::BufReader::new(stream);
    for line in reader.lines() {
        let line = line.unwrap();
        println!("{}", line);
    }

    Ok(())
}
