use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
  // connects to server
  let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
  client.set_nonblocking(true).expect("failed to initiate non-blocking");

  // creates channel and sets it as string type
  let (tx, rx) = mpsc::channel::<String>();

  // creates thread
  thread::spawn(move || loop {
    let mut buffer = vec![0; MSG_SIZE];

    // reads message through buffer
    match client.read_exact(&mut buffer) {
      Ok(_) => {

        // if it returns good then we turn message into an iterator
        let msg = buffer.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
        println!("message recv {:?}", msg);
      },
      
      // error handling
      Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
      Err(_) => {
        println!("connection with server was severed");
        break;
      }
    }

    // checks to make sure server got the message
    match rx.try_recv() {
      Ok(msg) => {

        // takes the message and concerts it into bytes
        let mut buffer = msg.clone().into_bytes();
        buffer.resize(MSG_SIZE, 0);

        // writes the buffer into the client
        client.write_all(&buffer).expect("writing to socket failed");
        println!("message sent {:?}", msg);
      }, 

      // more error handling
      Err(TryRecvError::Empty) => (),
      Err(TryRecvError::Disconnected) => break
    }

    // makes the thread sleep
    thread::sleep(Duration::from_millis(100));
  });

  // ui
  println!("Write a Message:");
  loop {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("reading from stdin failed");
    let msg = buffer.trim().to_string();
    
    // exit case
    if msg == ":quit" || tx.send(msg).is_err() {break}
  }
  println!("goodbye!");

}