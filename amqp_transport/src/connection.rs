use anyhow::{bail, ensure, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error};

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn start(mut self) {
        match self.run().await {
            Ok(()) => {}
            Err(err) => error!(%err, "Error during processing of connection"),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.negotiate_version().await?;
        Ok(())
    }

    async fn negotiate_version(&mut self) -> Result<()> {
        const HEADER_SIZE: usize = 8;
        const PROTOCOL_VERSION: &[u8] = &[0, 9, 1];
        const PROTOCOL_HEADER: &[u8] = b"AMQP\0\0\x09\x01";

        debug!("Negotiating version");

        let mut read_header_buf = [0; HEADER_SIZE];

        self.stream.read_exact(&mut read_header_buf).await?;

        debug!(received_header = ?read_header_buf,"Received protocol header");

        ensure!(
            &read_header_buf[0..5] == b"AMQP\0",
            "Received wrong protocol"
        );

        let version = &read_header_buf[5..8];

        self.stream.write_all(PROTOCOL_HEADER).await?;

        if version == PROTOCOL_VERSION {
            debug!(?version, "Version negotiation successful");
            Ok(())
        } else {
            debug!(?version, expected_version = ?PROTOCOL_VERSION, "Version negotiation failed, unsupported version");
            self.stream.shutdown().await?;
            bail!("Unsupported protocol version {:?}", version);
        }
    }
}