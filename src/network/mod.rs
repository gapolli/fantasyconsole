use std::net::UdpSocket;
use std::io;

pub struct NetworkManager {
    pub socket: UdpSocket,
    pub remote_address: Option<String>,
    pub is_server: bool,
}

impl NetworkManager {
    // Inicializa uma conexão UDP não-bloqueante
    pub fn new(local_port: &str, remote_addr: Option<String>, is_server: bool) -> io::Result<Self> {
        let bind_addr = format!("0.0.0.0:{}", local_port);
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?; // Define como não-bloqueante para não travar o Game Loop

        Ok(Self {
            socket,
            remote_address: remote_addr,
            is_server,
        }
    }

    // Transmite o array de botões locais comprimido em bytes para a rede
    pub fn send_input_state(&self, local_buttons: &[bool; 6]) -> io::Result<()> {
        if let Some(ref target) = self.remote_address {
            let mut payload = 0u8;
            // Compacta os 6 booleanos em um único byte (bitmask) para economizar banda
            for i in 0..6 {
                if local_buttons[i] {
                    payload |= 1 << i;
                }
            }
            self.socket.send_to(&[payload], target)?;
        }
        Ok(())
    }

    // Escuta pacotes de rede sem travar os frames por segundo da engine
    pub fn receive_remote_input(&self) -> io::Result<Option<[bool; 6]>> {
        let mut buffer = [0u8; 1];
        match self.socket.recv_from(&mut mut buffer) {
            Ok((_amt, _src)) => {
                let payload = buffer[0];
                let mut remote_buttons = [false; 6];
                for i in 0..6 {
                    remote_buttons[i] = (payload & (1 << i)) != 0;
                }
                Ok(Some(remote_buttons))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Retorna None graciosamente se não houver dados novos na fila da rede
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}
