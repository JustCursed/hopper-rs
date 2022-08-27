use crate::{
    protocol::{
        error::ProtoError,
        lazy::LazyPacket,
        packets::{Disconnect, Handshake, LoginStart, State},
        PacketReadExtAsync, PacketWriteExtAsync,
    },
    HopperError,
};
use std::{error::Error, net::SocketAddr, time::Duration};
use tokio::net::TcpStream;

pub enum NextState {
    Login(LazyPacket<LoginStart>),
    Status,
}

pub struct IncomingClient {
    pub address: SocketAddr,
    pub stream: TcpStream,

    pub handshake: LazyPacket<Handshake>,
    pub next_state: NextState,
}

impl IncomingClient {
    pub async fn disconnect(mut self, reason: impl Into<String>) {
        if !matches!(
            self.handshake.data().map(|data| data.next_state),
            Ok(State::Login)
        ) {
            return;
        }

        self.stream
            .write_serialize(Disconnect::new(reason))
            .await
            .ok();
    }

    pub async fn disconnect_err(self, err: impl Error) {
        self.disconnect(err.to_string()).await;
    }

    async fn handshake_inner(
        (mut stream, address): (TcpStream, SocketAddr),
    ) -> Result<Self, ProtoError> {
        let mut handshake: LazyPacket<Handshake> = stream.read_packet().await?.try_into()?;

        // only read LoginStart information (containing the username)
        // if the next_state is login
        let next_state = match handshake.data()?.next_state {
            State::Status => NextState::Status,
            State::Login => NextState::Login(stream.read_packet().await?.try_into()?),
        };

        Ok(IncomingClient {
            address,
            stream,
            handshake,
            next_state,
        })
    }

    pub async fn handshake(connection: (TcpStream, SocketAddr)) -> Result<Self, HopperError> {
        tokio::time::timeout(Duration::from_secs(2), Self::handshake_inner(connection))
            .await
            .map_err(|_| HopperError::TimeOut)?
            .map_err(HopperError::Protocol)
    }
}

impl std::fmt::Display for IncomingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)
    }
}
