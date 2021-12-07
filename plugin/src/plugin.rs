//! Our implementation of `IVpnPlugIn` which is the bulk of the UWP VPN plugin.

use windows::{
    self as Windows,
    core::*,
    Networking::Vpn::*,
};

/// The VPN plugin object which provides the hooks that the UWP VPN platform will call into.
#[implement(Windows::Networking::Vpn::IVpnPlugIn)]
struct VpnPlugin;

impl VpnPlugin {
    /// Called by the platform so that we may connect and setup the VPN tunnel.
    fn Connect(&self, _channel: &Option<VpnChannel>) -> Result<()> {
        Ok(())
    }

    /// Called by the platform to indicate we should disconnect and cleanup the VPN tunnel.
    fn Disconnect(&self, _channel: &Option<VpnChannel>) -> Result<()> {
        Ok(())
    }

    /// Called by the platform to indicate there are outgoing packets ready to be encapsulated.
    ///
    /// `packets` contains outgoing L3 IP packets that we should encapsulate in whatever protocol
    /// dependant manner before placing them in `encapsulatedPackets` so that they may be sent to
    /// the remote endpoint.
    fn Encapsulate(
        &self,
        _channel: &Option<VpnChannel>,
        _packets: &Option<VpnPacketBufferList>,
        _encapsulatedPackets: &Option<VpnPacketBufferList>,
    ) -> Result<()> {
        Ok(())
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
        _channel: &Option<VpnChannel>,
        _buffer: &Option<VpnPacketBuffer>,
        _decapsulatedPackets: &Option<VpnPacketBufferList>,
        _controlPackets: &Option<VpnPacketBufferList>,
    ) -> Result<()> {
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