# Sessio - P2P SSH

Cross-platform SSH 2.0 Over QUIC Client & Server implementation written in Rust.

## Table of contents

- [About](#about)
- [Features](#features)
- [Installation & Usage](#installation--usage)

## About

> Sessio is a minimal SSH 2.0 over QUIC implementation that uses UDP Hole punching to facilitate Peer-to-peer SSH connections between devices behind firewalls and NATs.

Currently only pub-key auth is supported.

## Features

### Why QUIC?
By using QUIC as the transport protocol, Sessio is able keep sessions uninterrupted even when switching between networks or in otherwise unreliable network conditions. 
QUIC also uses UDP, which makes the p2p connection creation process much simpler and more reliable.

### Hole punching
Peer-to-Peer SSH connections in Sessio are achieved using QUIC as the transport protocol. By employing the UDP hole punching technique, Sessio enables direct connections between devices behind firewalls and NATs. This allows seamless and secure SSH communication without the need for a middleman server to relay traffic or open ports.

> No publicly open ports are required to create P2P SSH Connections in Sessio.

### Coordination server

You will however, require a publicly open coordination server. This server is only used by the client and server to exchange public IP addresses and ports. After which they will perform UDP Hole punching to complete the connection.

### IPv6 Support

Sessio is primarily intended to be used with IPv6, but IPv4 is also supported for most NAT Types. Sessio does not work with Address and Port-Dependent Mapping (Symmetric NAT).

### Multiplexing
All SSH connections to a host are multiplexed through the same QUIC connection by opening a new bi-directional stream for each ssh connection to ensure no Head-of-line blocking.

### SFTP
A minimal SFTP implementation is also included.

### Port-forwarding
Only local port forwarding is supported at the moment.

### GUI
Sessio also exposes a gRPC interface for developers wanting to develop a GUI for the client in the language they prefer. I have made one cross-platform (Android, Linux, Windows) implementation here: https://github.com/0xc0ffee1/sessio-gui

## Installation & Usage
> ! **Sessio is in beta and not recommended for production use !**

### Server
1. Download the `sessio-server` binary for your platform from releases.
2. Run `sessio-server --help` to view the usage
3. Add the public key of a client to your `authorized_keys` file in `/your_home/.sessio/`

### Coordination server
1. Download the `sessio-coordinator` binary for your platform from releases.
2. Generate your certificate and key with certbot for example (certbot certonly --standalone -d yourdomain.com)
3. You must allow each device to connect to the coordination server by adding authorized keys in /home/(your user)/.sessio/authorized_keys
4. Run it

### Client (daemon)
1. Download the `sessio-clientd` binary for your platform from releases.
2. Run it

### Client (CLI)
1. In progress. Use the GUI to interact with the daemon for now.

### Client (GUI)
See https://github.com/0xc0ffee1/sessio-gui

