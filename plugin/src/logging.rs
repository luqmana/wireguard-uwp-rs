//! Logging primitives along with our ETW Trace Provider.

use win_etw_macros::trace_logging_provider;

/// The collection of ETW events our plugin emits.
#[allow(non_snake_case)]
#[trace_logging_provider(guid = "c4522a55-401f-4b81-93f9-aa0d1db734c4")]
pub trait WireGuardUWPEvents {
    /// `Connect` event emitted once we've successfully connected
    #[event(level = "info")]
    fn connected(remote_host: &str, remote_port: u16);
    /// Event emitted if we've failed during `Connect`
    #[event(level = "error")]
    fn connect_fail(code: u32, msg: &str);

    /// Event emitted for `Disconnect`
    #[event(level = "warn")]
    fn disconnect(code: u32, msg: &str);

    // Noisy packet encap/decap events

    /// Packet encap begin event.
    /// Indicates how many outgoing packets are ready to be encapsulated.
    #[event(level = "verbose")]
    fn encapsulate_begin(packets: u32);
    /// Packet encap end event.
    /// Indicates how many frames we sent to the remote endpoint.
    #[event(level = "verbose")]
    fn encapsulate_end(frames: u32);

    /// Frame decap begin event.
    /// Indicates the size of the frame received from the remote endpoint.
    #[event(level = "verbose")]
    fn decapsulate_begin(frame_sz: u32);
    /// Frame decap end event.
    /// Indicates how many packets were decapsulated and how many frames sent to the remote.
    #[event(level = "verbose")]
    fn decapsulate_end(packets: u32, control_frames: u32);

    /// KeepAlive packet event.
    /// Indicates how many bytes destined for remote.
    #[event(level = "info")]
    fn keepalive(packet_sz: u32);
}
