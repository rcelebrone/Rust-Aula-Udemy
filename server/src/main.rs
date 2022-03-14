use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::{thread, time};

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    // crio uma variavel com uma instancia de TcpListener vinculado ao endereço IP da variavel LOCAL
    let server = TcpListener::bind(LOCAL)
        // se o vinculo do IP falhar, gero erro
        .expect("Listener failed to bind");

    // configura o server como não bloqueante
    server.set_nonblocking(true)
        // se falhar, gera um erro
        .expect("Failed to initialize non-blocking");

    // crio uma variavel para receber os clients que vão conectar no server
    let mut clients = vec![];

    // crio variaveis de canal para envio e recebimento de mensagem assincronas
    let (tx, rx) = mpsc::channel::<String>();

    // crio um loop para ficar constantemente avaliando o que chega no server
    loop {
        // se o server receber uma mensagem com sucesso de um cliente
        if let Ok((mut socket, addr)) = server.accept() {

            // apresento o IP do cliente que conectou no cliente
            println!("Client {} connected", addr);

            // faço um clone do sender principal para dentro da thread atual
            let tx = tx.clone();
            
            // insiro dentro do array de clients uma referencia para o sender (client que está conectando no server) 
            clients.push(socket.try_clone().expect("Failed to clone client"));

            thread::spawn(move || loop {

                let mut buff = vec![0; MSG_SIZE];
                
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf9 message");

                        println!("{}: {:?}", addr, msg);

                        tx.send(msg).expect("Failed to send msg to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with {}", addr);
                        break;
                    }
                }

                sleep();
            });
        };

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                // converto a mensagem em bytes
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok() 
            }).collect::<Vec<_>>();
        }

        sleep();
    }
}

fn sleep() {
    thread::sleep(time::Duration::from_millis(100));
}