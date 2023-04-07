use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

// creates a sleeper function
fn sleep() {
  thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
  // makes sure we have a good connection to our server
  let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
  server.set_nonblocking(true).expect("failed to initialize non-blocking");
  
  // lets us handle multiple clients
  let mut clients = vec![];

  // init channel and sets it to string type
  let (tx, rx) = mpsc::channel::<String>();
  loop {
    // lets us accept connections to server
    if let Ok((mut socket, addr)) = server.accept() {
      println!("Client {} connected", addr);

      // cloning so we can push it into our thread
      let tx = tx.clone();
      clients.push(socket.try_clone().expect("failed to clone client"));

      // creates thread and a loop
      thread::spawn(move || loop {
        let mut buffer = vec![0; MSG_SIZE];

        // reads the message into our buffer
        match socket.read_exact(&mut buffer) {
          Ok(_) => {

            // converts what we read into an iterator
            let msg = buffer.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
            
            // converts string into an actual string
            let msg = String::from_utf8(msg).expect("Invalid utf8 message");

            println!("{}: {:?}", addr, msg);

            // sends message from transmitter to receiver
            tx.send(msg).expect("failed to send msg to rx");
          }, 

          // error handling
          Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
          Err(_) => {
            // tells us that we are closing connection with the server
            println!("closing connection with: {}", addr);
            break;
            }
          }

          sleep();
      });
    }

    // has our server try to receive the message
    if let Ok(msg) = rx.try_recv() {
      
      clients = clients.into_iter().filter_map(|mut client| {

        // converts message into bytes
        let mut buffer = msg.clone().into_bytes();
        buffer.resize(MSG_SIZE, 0);

        client.write_all(&buffer).map(|_| client).ok()
      }).collect::<Vec<_>>();
    }

    sleep();
  }
}