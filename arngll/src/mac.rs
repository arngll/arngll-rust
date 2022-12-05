// Copyright (c) 2022, The ARNGLL-Rust Authors.
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
// TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::collections::HashSet;
use futures::{Sink, SinkExt, Stream, StreamExt};
use futures::lock::Mutex;
use quick_dsp::filter::IteratorExt;
use super::*;

pub struct Mac<FrameSink, FrameStream, SC=NullSecurityContext>
where
    FrameSink: Sink<Vec<u8>> + Unpin,
    FrameStream: Stream<Item=Vec<u8>> + Unpin,
{
    sink: Mutex<FrameSink>,
    stream: Mutex<FrameStream>,

    callsign: HamAddr,
    groups: HashSet<HamAddr>,
    netid: NetworkId,
    security_context: SC,
}

impl <FrameSink, FrameStream, SC> Mac <FrameSink, FrameStream, SC>
    where
        FrameSink: Sink<Vec<u8>> + Unpin,
        FrameStream: Stream<Item=Vec<u8>> + Unpin,
        SC: SecurityContext,
        FrameSink::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(sink: FrameSink, stream: FrameStream, callsign: HamAddr, netid: NetworkId, sc: SC) -> Self {
        Mac {
            sink: Mutex::new(sink),
            stream: Mutex::new(stream),
            callsign,
            groups: HashSet::new(),
            netid,
            security_context: sc,
        }
    }

    pub async fn listen(&self) -> Result<Option<(FrameInfo, Vec<u8>)>, anyhow::Error> {
        while let Some(frame) = self.stream.lock().await.next().await {
            let (frame_info, payload) = match FrameInfo::try_from_bytes(&frame) {
                Ok(x) => x,
                Err(err) => {
                    log::info!("Frame parse failed: {:?}", err);
                    continue;
                }
            };

            if frame_info.network_id.unwrap_or(NetworkId(0)) != self.netid {
                // Wrong network.
                continue;
            }

            let direct_unicast =  frame_info.dst_addr == self.callsign;

            // TODO: eventually only listen to specific groups
            let direct_multicast =  frame_info.dst_addr.is_multicast();

            if direct_unicast {
                if let Some(ack_frame) = frame_info.generate_ack_frame(payload) {
                    let ack_bytes = ack_frame
                        .bytes_with_payload(&[])
                        .append_crc(&X25)
                        .collect::<Vec<_>>();
                    self.sink.lock().await.send(ack_bytes).await?;
                }
            }

            if !direct_unicast && !direct_multicast {
                // Not for us.
                continue
            }

            let mut mut_payload = payload.to_vec();
            if let Err(err) = self.security_context.process_inbound(&frame_info, &mut mut_payload) {
                log::info!("Frame security failed: {:?}", err);
                continue;
            }

            return Ok(Some((frame_info, mut_payload)))
        }
        Ok(None)
    }
}