use crate::error::{ConException, ProtocolError, Result};
use amqp_core::methods::FieldValue;
use anyhow::Context;
use bytes::Bytes;
use smallvec::SmallVec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::trace;

const REQUIRED_FRAME_END: u8 = 0xCE;

mod frame_type {
    pub const METHOD: u8 = 1;
    pub const HEADER: u8 = 2;
    pub const BODY: u8 = 3;
    pub const HEARTBEAT: u8 = 8;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// The type of the frame including its parsed metadata.
    pub kind: FrameType,
    pub channel: u16,
    /// Includes the whole payload, also including the metadata from each type.
    pub payload: Bytes,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Method = 1,
    Header = 2,
    Body = 3,
    Heartbeat = 8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentHeader {
    pub class_id: u16,
    pub weight: u16,
    pub body_size: u64,
    pub property_flags: SmallVec<[u16; 1]>,
    pub property_fields: Vec<FieldValue>,
}

impl ContentHeader {
    pub fn new() -> Self {
        todo!()
    }
}

pub async fn write_frame<W>(frame: &Frame, mut w: W) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    trace!(?frame, "Sending frame");

    w.write_u8(frame.kind as u8).await?;
    w.write_u16(frame.channel).await?;
    w.write_u32(u32::try_from(frame.payload.len()).context("frame size too big")?)
        .await?;
    w.write_all(&frame.payload).await?;
    w.write_u8(REQUIRED_FRAME_END).await?;

    Ok(())
}

pub async fn read_frame<R>(r: &mut R, max_frame_size: usize) -> Result<Frame>
where
    R: AsyncReadExt + Unpin,
{
    let kind = r.read_u8().await.context("read type")?;
    let channel = r.read_u16().await.context("read channel")?;
    let size = r.read_u32().await.context("read size")?;

    let mut payload = vec![0; size.try_into().unwrap()];
    r.read_exact(&mut payload).await.context("read payload")?;

    let frame_end = r.read_u8().await.context("read frame end")?;

    if frame_end != REQUIRED_FRAME_END {
        return Err(ProtocolError::Fatal.into());
    }

    if max_frame_size != 0 && payload.len() > max_frame_size {
        return Err(ConException::FrameError.into_trans());
    }

    let kind = parse_frame_type(kind, channel)?;

    let frame = Frame {
        kind,
        channel,
        payload: payload.into(),
    };

    trace!(?frame, "Received frame");

    Ok(frame)
}

fn parse_frame_type(kind: u8, channel: u16) -> Result<FrameType> {
    match kind {
        frame_type::METHOD => Ok(FrameType::Method),
        frame_type::HEADER => Ok(FrameType::Header),
        frame_type::BODY => Ok(FrameType::Body),
        frame_type::HEARTBEAT => {
            if channel != 0 {
                Err(ProtocolError::ConException(ConException::FrameError).into())
            } else {
                Ok(FrameType::Heartbeat)
            }
        }
        _ => Err(ConException::FrameError.into_trans()),
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{Frame, FrameType};
    use bytes::Bytes;

    #[tokio::test]
    async fn read_small_body() {
        let mut bytes: &[u8] = &[
            /*type*/
            1,
            /*channel*/
            0,
            0,
            /*size*/
            0,
            0,
            0,
            3,
            /*payload*/
            1,
            2,
            3,
            /*frame-end*/
            super::REQUIRED_FRAME_END,
        ];

        let frame = super::read_frame(&mut bytes, 10000).await.unwrap();
        assert_eq!(
            frame,
            Frame {
                kind: FrameType::Method,
                channel: 0,
                payload: Bytes::from_static(&[1, 2, 3]),
            }
        );
    }
}
