# WireGuard UWP

A Universal Windows Platform (UWP) [VPN Plug-in](https://docs.microsoft.com/en-us/uwp/api/windows.networking.vpn.ivpnplugin) for [WireGuard<sup>®</sup>](https://www.wireguard.com/) written in [Rust](https://www.rust-lang.org/).

Windows provides a plug-in based model for adding 3rd-party VPN protocols. VPN profiles
backed by such a plugin are referred to as **Plugin**/**3rd-party**/**UWP** profiles, as opposed
to **Native** profiles (i.e. built-in SSTP, IKEv2).

**WireGuard** is a VPN protocol that aims to be: Fast, Modern and Secure. Principles
which dovetail quite nicely with the Rust programming language. The actual noise-based WireGuard
implementation comes from Cloudflare's [boringtun](https://github.com/cloudflare/boringtun).

With the rapidly maturing [Rust for Windows](https://github.com/microsoft/windows-rs) bindings,
this projects serve as a fun experiment in putting all the above together.

## Building

Make sure you have Rust installed. Then, once you've cloned the repo just simply run:
```console
$ cargo build --release
```

The project currently only builds on Windows but given the Windows-specific nature, that's not
considered a limitation.

## Installing

Once you've successfully built the project, you can install it by running the following commands
in a powershell prompt from the repo root:
```powershell
copy appx\* target\release
Add-AppxPackage -Register .\target\release\AppxManifest.xml
```

**NOTE:** This does an in-place sort of installation in that the installed app will refer to
the binaries in your `target\release` folder. So you may encounter issues if you modify those
after installation. This is just a stop-gap until a proper `.appx` can be generated.

## Running

To get your VPN tunnel up and running:

1. Open Windows Settings and navigate to the VPN page:
`Network & Internet > VPN`.
2. Select `Add a VPN connection`.
3. From the `VPN provider` dropdown select **WireGuard UWP VPN**.
4. Give your new VPN profile a name under `Connection name`.
5. Enter the remote endpoint hostname or IP address under `Server name or address`.
6. Hit `Save`.

The settings you can tweak from the Windows Settings UI are limited to just the profile name
and remote endpoint's hostname. To modify the private key, public key, remote port etc we must
set those values manually. From a powershell prompt:

```powershell
$vpnConfig = @'
<WireGuard>
    <Interface>
        <PrivateKey>...</PrivateKey>
        <Address>10.0.0.2/32</Address>
        <DNS>1.1.1.1</DNS>
        <DNS>8.8.8.8</DNS>
        <DNSSearch>vpn.example.com</DNSSearch>
        <DNSSearch>foo.corp.example.com</DNSSearch>
    </Interface>
    <Peer>
        <PublicKey>...</PublicKey>
        <Port>51000</Port>
        <AllowedIPs>10.0.0.0/24</AllowedIPs>
        <AllowedIPs>10.10.0.0/24</AllowedIPs>
        <AllowedIPs>10.20.0.0/24</AllowedIPs>
        <PersistentKeepalive>25</PersistentKeepalive>
    </Peer>
</WireGuard>
'@

Set-VpnConnection -Name ProfileNameHere -CustomConfiguration $vpnConfig
```

The only required values are `PrivateKey`, `Address`, `PublicKey`, & `Port`. The rest are optional.
You may repeat `Address` multiple times to assign multiple IPv4 & IPv6 addresses to the virtual
interface. Similarly, you may specify `AllowedIPs` multiple times to define the routes that
should go over the virtual interface.

You should now be able to select the new profile and hit `Connect`.

**NOTE:** Ideally, you could just specify `Port` colon separated with the hostname but the
corresponding API for retrieving that value is statically typed as a HostName.

**NOTE:** You should make sure to set a `PersistentKeepalive` value on the remote
side for each **WireGuard UWP**-based client because the UWP VPN plugin model
offers limited options for the plugin to perform periodic actions. Generally,
the plugin will only be woken if there are packets that need to be encapsulated or
if there are incoming buffers from the remote that need to be decapsulated. The
platform does provide a `GetKeepAlivePayload` it will call on occasion but the
interval in which it will be called cannot be controlled by the plugin author or
the user but rather the platform itself. Hence it's important to make sure the
server will keep the tunnel alive by sending the periodic keep alives in-band.

**NOTE:** The main foreground app is planned to offer a simple UI for setting and modifying these
values.

This has only been tested on Windows 10 21H1 (19043.1348) but should work on any updated
Windows 10 or 11 release. It'll probably work on older versions but no guarantees.

### DNS

DNS servers are plumbed via [Name Resolution Policy Table](https://docs.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2012-r2-and-2012/dn593632(v=ws.11)) (NRPT) Rules.
You can print the current set of rules applied while a VPN profile backed by the plugin
is connected via either:

CMD:
```cmd
netsh namespace show effectivepolicy
```

PowerShell:
```powershell
Get-DnsClientNrptPolicy -Effective
```

If any number of DNS servers are specified in the config, the plugin will plumb one
NRPT rule with the namespace set to `.`; this means the rule will apply to all
domains. While NRPT rules do support matching based on domain suffixes/prefixes
to apply domain-specific DNS servers, we just use a single wildcard rule.

You will note another rule plumbed while the plugin is connected, for the specific
domain used to connect to the remote endpoint. This is added automatically by the
platform so that if the tunnel is interrupted and needs to be re-established, we
don't end up in a situation wherein we can't resolve the remote's hostname.

You can also specify one more optional search domains. These will be added to the
VPN interface's **Connection-specific DNS Suffix Search List**. The first specified
search domain will also be the primary Connection-specific DNS Suffix, as can be
confirmed with `ipconfig /all` after connecting. You'll see an additional suffix-type
NRPT rule for each such search domain configured, with the specific DNS servers set
to whatever was configured.

## Tracing

The plugin emits a number of [ETW](https://docs.microsoft.com/en-us/windows/win32/etw/event-tracing-portal)
under the Event Provider identified by a specific GUID (`c4522a55-401f-4b81-93f9-aa0d1db734c4`). The events
are emitted with the help of the [rust_win_etw](https://github.com/microsoft/rust_win_etw) crate which
provides a way to define & emit ETW events from your Rust code.

To consume these events, there are a number of different tools which can be used. **rust_win_etw** provides
a quick rundown on how to capture them: https://github.com/microsoft/rust_win_etw#how-to-capture-and-view-events

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

---
<sub><sub>"WireGuard" and the "WireGuard" logo are registered trademarks of Jason A. Donenfeld.</sub></sub>