//! Our implementation of `IVpnPlugIn` which is the bulk of the UWP VPN plugin.

use std::sync::{Arc, RwLock};

use boringtun::noise::{Tunn, TunnResult};
use windows::{
    self as Windows,
    core::*,
    Networking::Sockets::*,
    Networking::Vpn::*,
    Networking::*,
    Win32::Foundation::{E_BOUNDS, E_UNEXPECTED},
};

use crate::utils::{IBufferExt, Vector};

struct Inner {
    tunn: Option<Box<Tunn>>,
}

impl Inner {
    fn new() -> Self {
        Self { tunn: None }
    }
}

/// The VPN plugin object which provides the hooks that the UWP VPN platform will call into.
#[implement(Windows::Networking::Vpn::IVpnPlugIn)]
pub struct VpnPlugin {
    inner: RwLock<Inner>,
}

impl VpnPlugin {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Inner::new()),
        }
    }

    /// Called by the platform so that we may connect and setup the VPN tunnel.
    fn Connect(&self, channel: &Option<VpnChannel>) -> Result<()> {
        let channel = channel.as_ref().ok_or(Error::from(E_UNEXPECTED))?;
        let mut inner = self.inner.write().unwrap();

        let config = channel.Configuration()?;

        // TODO: read these from config
        let static_private = Arc::new(env!("PRIVATE_KEY").parse().unwrap());
        let peer_static_public = Arc::new(env!("REMOTE_PUBLIC_KEY").parse().unwrap());
        let persistent_keepalive = Some(25);
        let preshared_key = None;

        // Create WG tunnel object
        let tunn = Tunn::new(
            static_private,
            peer_static_public,
            preshared_key,
            persistent_keepalive,
            0,    // Peer index. we only have one peer
            None, // TODO: No rate limiter
        )
        // TODO: is E_UNEXPECTED the right error here?
        .map_err(|e| Error::new(E_UNEXPECTED, e.into()))?;

        // Stuff it into our inner state
        // TODO: handle reconnects
        let old_tunn = std::mem::replace(&mut inner.tunn, Some(tunn));
        assert!(old_tunn.is_none());

        // Create socket and register with VPN platform
        let sock = DatagramSocket::new()?;
        channel.AddAndAssociateTransport(&sock, None)?;

        // Just use the first server listed to connect to remote endpoint
        let server = config.ServerHostNameList()?.GetAt(0)?;

        // TODO: use config for port value
        // We "block" here with the call to `.get()` but given this is a UDP socket
        // connect isn't actually something that will hang (DNS aside perhaps?).
        sock.ConnectAsync(server, "51000")?.get()?;

        // TODO: use config for local address choice
        let ipv4 = Vector::new(vec![Some(HostName::CreateHostName("10.0.0.2")?)]);

        // TODO: use config for routes
        let routes = VpnRouteAssignment::new()?;
        let route = VpnRoute::CreateVpnRoute(HostName::CreateHostName("10.0.0.0")?, 24)?;
        routes.SetIpv4InclusionRoutes(Vector::new(vec![Some(route)]))?;

        // Kick off the VPN setup
        channel.Start(
            ipv4,
            None,   // TODO: No IPv6
            None,   // Interface ID portion of IPv6 address for VPN tunnel
            routes,
            None,   // TODO: DNS
            1500,   // MTU size of VPN tunnel interface
            1600,   // Max frame size of incoming buffers from remote endpoint
            false,  // Disable low cost network monitoring
            sock,   // Pass in the socket to the remote endpoint
            None,   // No secondary socket used.
        )?;

        Ok(())
    }

    /// Called by the platform to indicate we should disconnect and cleanup the VPN tunnel.
    fn Disconnect(&self, channel: &Option<VpnChannel>) -> Result<()> {
        let channel = channel.as_ref().ok_or(Error::from(E_UNEXPECTED))?;

        let mut inner = self.inner.write().unwrap();
        inner.tunn = None;

        channel.Stop()?;

        Ok(())
    }

    /// Called by the platform to indicate there are outgoing packets ready to be encapsulated.
    ///
    /// `packets` contains outgoing L3 IP packets that we should encapsulate in whatever protocol
    /// dependant manner before placing them in `encapsulatedPackets` so that they may be sent to
    /// the remote endpoint.
    fn Encapsulate(
        &self,
        channel: &Option<VpnChannel>,
        packets: &Option<VpnPacketBufferList>,
        encapsulatedPackets: &Option<VpnPacketBufferList>,
    ) -> Result<()> {
        let channel = channel.as_ref().ok_or(Error::from(E_UNEXPECTED))?;
        let packets = packets.as_ref().ok_or(Error::from(E_UNEXPECTED))?;
        let encapsulatedPackets = encapsulatedPackets
            .as_ref()
            .ok_or(Error::from(E_UNEXPECTED))?;

        let inner = self.inner.read().unwrap();
        let tunn = if let Some(tunn) = &inner.tunn {
            &**tunn
        } else {
            // We haven't initalized tunn yet, just return
            return Ok(());
        };

        let mut ret_buffers = vec![];
        let mut encap_err = None;

        // Process outgoing packets from VPN tunnel.
        // TODO: Not using the simpler `for packet in packets` because
        //       `packets.First()?` fails with E_NOINTERFACE for some reason.
        let packets_sz = packets.Size()? as usize;
        for _ in 0..packets_sz {
            let packet = packets.RemoveAtBegin()?;
            let src = packet.get_buf()?;

            // Grab a destination buffer for the encapsulated packet
            let mut encapPacket = channel.GetVpnSendPacketBuffer()?;
            let dst = encapPacket.get_buf_mut()?;

            // Try to encapsulate packet
            let res = tunn.encapsulate(src, dst);

            if let TunnResult::WriteToNetwork(packet) = res {
                // Packet was encap'd successfully, make sure to update length on the WinRT side
                let new_len = u32::try_from(packet.len()).map_err(|_| Error::from(E_BOUNDS))?;
                drop(packet);
                encapPacket.Buffer()?.SetLength(new_len)?;

                // Now, tack it onto `encapsulatedPackets` to send to remote endpoint
                encapsulatedPackets.Append(encapPacket)?;
            } else {
                match res {
                    // Handled above
                    TunnResult::WriteToNetwork(_) => {}

                    // Packet was queued while we complete the handshake
                    TunnResult::Done => {}

                    // Encountered an error while trying to encapsulate
                    TunnResult::Err(err) => {
                        if encap_err.is_none() {
                            encap_err = Some(Error::new(
                                E_UNEXPECTED,
                                format!("encap error: {:?}", err).into(),
                            ));
                        }
                    }

                    // Impossible cases for encapsulate
                    TunnResult::WriteToTunnelV4(_, _) | TunnResult::WriteToTunnelV6(_, _) => {
                        panic!("unexpected result from encapsulate")
                    }
                }

                // We must return the `encapPacket` we requested
                ret_buffers.push(encapPacket);
            }

            // Note: this loop does not consume the items in packets which is important
            //       as ANY `VpnPacketBuffer` we get (whether as some argument to a `IVpnPlugIn`
            //       method or via methods on `VpnChannel`) we are expected to return to the
            //       platform. Since we're not en/decapsulating in-place, it works out to leave
            //       the buffers in `packets` so that the platform may clean them up.
            packets.Append(packet)?;
        }

        // Just stick the unneeded buffers onto `packets` so the platform can clean them up
        for packet in ret_buffers {
            packets.Append(packet)?;
        }

        // If we encountered an error, return it
        if let Some(err) = encap_err {
            Err(err)
        } else {
            Ok(())
        }
    }

    /// Called by the platform to indicate we've received a frame from the remote endpoint.
    ///
    /// `buffer` will contain whatever data we received from the remote endpoint which may
    /// either contain control or data payloads. For data payloads, we will decapsulate into
    /// 1 (or more) L3 IP packet(s) before returning them to the platform by placing them in
    /// `decapsulatedPackets`, making them ready to be injected into the virtual tunnel. If
    /// we need to send back control payloads or otherwise back to the remote endpoint, we
    /// may place such frames into `controlPackets`.
    fn Decapsulate(
        &self,
        channel: &Option<VpnChannel>,
        buffer: &Option<VpnPacketBuffer>,
        decapsulatedPackets: &Option<VpnPacketBufferList>,
        controlPackets: &Option<VpnPacketBufferList>,
    ) -> Result<()> {
        let channel = channel.as_ref().ok_or(Error::from(E_UNEXPECTED))?;
        let buffer = buffer.as_ref().ok_or(Error::from(E_UNEXPECTED))?;
        let decapsulatedPackets = decapsulatedPackets
            .as_ref()
            .ok_or(Error::from(E_UNEXPECTED))?;
        let controlPackets = controlPackets.as_ref().ok_or(Error::from(E_UNEXPECTED))?;

        let inner = self.inner.read().unwrap();
        let tunn = if let Some(tunn) = &inner.tunn {
            &**tunn
        } else {
            // We haven't initalized tunn yet, just return
            return Ok(());
        };

        // Allocate a buffer for the decapsulate packet
        let mut decapPacket = channel.GetVpnReceivePacketBuffer()?;
        let dst = decapPacket.get_buf_mut()?;

        // Get a slice to the datagram we just received from the remote endpoint and try to decap
        let datagram = buffer.get_buf()?;
        let res = tunn.decapsulate(None, datagram, dst);

        match res {
            // Nothing to do with this decap result
            TunnResult::Done => {
                // TODO: Return unused `decapPacket` buffer
            }

            // Encountered an error while trying to decapsulate
            TunnResult::Err(err) => {
                // TODO: Return unused `decapPacket` buffer
                return Err(Error::new(
                    E_UNEXPECTED,
                    format!("encap error: {:?}", err).into(),
                ));
            }

            // We need to send response back to remote endpoint
            TunnResult::WriteToNetwork(packet) => {
                // Make sure to update length on WinRT buffer
                let new_len = u32::try_from(packet.len()).map_err(|_| Error::from(E_BOUNDS))?;
                drop(packet);

                // TODO: technically, we really should've used `GetVpnSendPacketBuffer` for this
                //       buffer but boringtun doesn't really have a way to know in advance if it'll
                //       be giving back control packets instead of data packets.
                //       We could just use temp buffers and copy as appropriate?
                let controlPacket = decapPacket;
                controlPacket.Buffer()?.SetLength(new_len)?;

                // Tack onto `controlPackets` so that they get sent to remote endpoint
                controlPackets.Append(controlPacket)?;

                // We need to probe for any more packets queued to send
                loop {
                    // Allocate a buffer for control packet
                    let mut controlPacket = channel.GetVpnSendPacketBuffer()?;
                    let dst = controlPacket.get_buf_mut()?;

                    let res = tunn.decapsulate(None, &[], dst);
                    if let TunnResult::WriteToNetwork(packet) = res {
                        // Make sure to update length on WinRT buffer
                        let new_len =
                            u32::try_from(packet.len()).map_err(|_| Error::from(E_BOUNDS))?;
                        drop(packet);
                        controlPacket.Buffer()?.SetLength(new_len)?;
                        controlPackets.Append(controlPacket)?;
                    } else {
                        // TODO: Return unused `controlPacket` buffer
                        // Nothing more to do
                        break;
                    }
                }
            }

            // Successfully decapsulated data packet
            TunnResult::WriteToTunnelV4(packet, _) | TunnResult::WriteToTunnelV6(packet, _) => {
                // Make sure to update length on WinRT buffer
                let new_len = u32::try_from(packet.len()).map_err(|_| Error::from(E_BOUNDS))?;
                drop(packet);
                decapPacket.Buffer()?.SetLength(new_len)?;

                // Tack onto `decapsulatedPackets` to inject into VPN interface
                decapsulatedPackets.Append(decapPacket)?;
            }
        }

        Ok(())
    }

    /// Called by the platform from time to time so that we may send some keepalive payload.
    ///
    /// If we decide we want to send any keepalive payload, we place it in `keepAlivePacket`.
    fn GetKeepAlivePayload(
        &self,
        _channel: &Option<VpnChannel>,
        _keepAlivePacket: &mut Option<VpnPacketBuffer>,
    ) -> Result<()> {
        Ok(())
    }
}
