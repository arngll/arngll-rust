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

use super::*;

pub trait SecurityContext {
    /// Modifies the `frame_info` (and possibly the `payload`) according to
    /// the security policy represented by this `SecurityContext`.
    fn process_outbound(&self, frame_info: &mut FrameInfo, payload: &mut[u8]) -> anyhow::Result<()>;

    /// Verifies that the inbound frame is secured according to the
    /// security policy represented by this `SecurityContext`. If encrypted,
    /// will also decrypt `payload` in-place.
    fn process_inbound(&self, frame_info: &FrameInfo, payload: &mut[u8]) -> anyhow::Result<()>;
}

/// Null Security Context.
///
/// Sends all packets as plaintext and rejects any inbound
/// packets with a SECINFO field.
pub struct NullSecurityContext;

impl SecurityContext for NullSecurityContext {
    fn process_outbound(&self, frame_info: &mut FrameInfo, _payload: &mut[u8]) -> anyhow::Result<()> {
        frame_info.sec_info = None;

        Ok(())
    }
    fn process_inbound(&self, frame_info: &FrameInfo, _payload: &mut[u8]) -> anyhow::Result<()> {
        if frame_info.sec_info.is_some() {
            bail!("NullSecurityContext: SECINFO present");
        }

        Ok(())
    }
}
