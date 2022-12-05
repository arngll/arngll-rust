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

use anyhow::Error;
use core::task::Context;
use core::task::Poll;
use std::net::Ipv6Addr;
use futures::stream::BoxStream;

/// Events that are vended
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TunEvent {
    Enabled(bool),
    Ipv6AddressAdded(Ipv6Addr,u8),
    Ipv6AddressRemoved(Ipv6Addr),
}

/// Trait for using a TUN interface. IPv6 specific.
pub trait TunInterface : Send + Sync {
    /// Sends a packet on the TUN interface.
    fn poll_send(&self, cx: &mut Context, packet: &[u8]) -> Poll<Result<(),Error>>;

    /// Receives a packet on the TUN interface.
    fn poll_recv<'a>(&self, cx: &mut Context, buffer: &'a mut [u8]) -> Poll<Result<&'a [u8],Error>>;

    /// Sets the "running" flag on the interface.
    fn set_running(&self, running: bool) -> Result<(),Error>;

    /// Sets the "up" flag on the interface.
    fn set_up(&self, is_up: bool) -> Result<(),Error>;

    /// Adds an IPv6 address with the given network prefix to the interface.
    fn ipv6_add_address(&self, addr: Ipv6Addr, prefix_len: u8) -> Result<(),Error>;

    /// Removes a IPv6 address from the interface.
    fn ipv6_remove_address(&self, addr: Ipv6Addr) -> Result<(),Error>;

    /// Joins the given IPv6 multicast group
    fn ipv6_join_mcast_group(&self, group: Ipv6Addr) -> Result<(),Error>;

    /// Leaves the given IPv6 multicast group
    fn ipv6_leave_mcast_group(&self, group: Ipv6Addr) -> Result<(),Error>;

    /// Takes the event stream. Must only be called once.
    fn take_event_stream(&self) -> BoxStream<'_, Result<TunEvent,Error>>;
}

