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

**NOTE:** At the moment, a lot is hardcoded including your private key and the remote endpoint's
public key. These are pulled into the build from environment variables at compile-time. In
powershell, before running the build command you can set them as so:
```powershell
$Env:PRIVATE_KEY = "..."
$Env:REMOTE_PUBLIC_KEY = "..."
```

The project currently only builds on Windows but given the Windows-specific nature, that's not
considered a limitation.

## Installing

Once you've successfully built the project, you can install it by running the following commands
in an admin powershell prompt from the repo root:
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
2. Select `Add a VPN connection.
3. From the `VPN provider` dropdown select **WireGuard UWP VPN**.
4. Give your new VPN profile a name under `Connection name`.
5. Enter the remote endpoint hostname or IP address under `Server name or address`.
6. Hit `Save`.

You should now be able to select the new profile and hit `Connect`.

**Note:** Some more hardcoded values include the remote endpoint's port to `51000`.
Similarly, the local VPN interface will be assigned a hardcoded IPv4 address of `10.0.0.2`
and plumb just a single route: `10.0.0.0/24`.

This has only been tested on Windows 10 21H1 (19043.1348) but should work on any updated
Windows 10 or 11 release. It'll probably work on older versions but no guarantees.

---
<sub><sub>"WireGuard" and the "WireGuard" logo are registered trademarks of Jason A. Donenfeld.</sub></sub>