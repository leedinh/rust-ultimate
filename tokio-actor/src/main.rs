use tokio::{
    net::{TcpStream, TcpListener},
    sync::{mpsc, oneshot},
    io::{AsyncReadExt, AsyncWriteExt, self}
};

struct MyActor {
    receiver: mpsc::Receiver<ActorMessage>,
    connection: TcpStream,
}

enum ActorMessage {
    SendMessage {
        message: String,
        respond_to: oneshot::Sender<u32>,
    },
}

impl MyActor {
    pub fn new(receiver: mpsc::Receiver<ActorMessage>, connection: TcpStream) -> Self {
        Self { receiver, connection }
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> io::Result<()> {
        match msg {
            ActorMessage::SendMessage { message, respond_to } => {
                println!("Sending message: {}", message);
                self.connection.write_all(message.as_bytes()).await?;
                let response = self.connection.read_u32().await?;
                let _ = respond_to.send(response);
                Ok(())
            }
        }
    }
}

async fn run_my_actor(mut actor: MyActor) {
    while let Some(msg) = actor.receiver.recv().await {
        println!("Received message");
        actor.handle_message(msg).await.unwrap();
    }
}

#[derive(Clone)]
pub struct MyActorHandle {
    sender: mpsc::Sender<ActorMessage>,
}

impl MyActorHandle {
    pub fn new(conn: TcpStream) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor =  MyActor::new(receiver, conn);
        println!("Spawning actor");
        tokio::spawn(run_my_actor(actor));

        Self { sender }
    }

    pub async fn send_message(&self, msg: String) -> u32 {
        let (sender, receiver) = oneshot::channel();
        let msg = ActorMessage::SendMessage {
            message: msg,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;
        receiver.await.expect("Actor handle dropped")
    }
}

async fn process(mut socket: TcpStream) {
    let mut buf = vec![0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        let n = socket
            .read(&mut buf)
            .await
            .expect("failed to read data from socket");

        if n == 0 {
            return;
        }

        socket
            .write_all(&buf[0..n])
            .await
            .expect("failed to write data to socket");
    }
}

async fn spwan_server() {
    println!("Spawing server");
    let listener = TcpListener::bind("127.0.0.1:8888").await.unwrap();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("Accepted connection from {}", addr);
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}


#[tokio::main]
async fn main() {

    // std::thread::spawn(|| {
    //     tokio::runtime::Runtime::new().unwrap().block_on(spwan_server());
    // });

    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let stream = TcpStream::connect("127.0.0.1:8888").await.unwrap();
    let new_actor = MyActorHandle::new(stream);
    let response = new_actor.send_message("Hello".to_string()).await;
    println!("Response: {}", response);
}
