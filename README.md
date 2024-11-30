
# Harbinger

Harbinger is an educational Rust project for exploring TCP headers and raw socket programming. It allows users to:
- Parse raw TCP packets and extract header details.
- Build and send TCP packets using raw sockets.
---
## Features

1. **TCP Header Parsing**:
   - Extract fields like source port, destination port, sequence number, and flags.

2. **TCP Packet Construction**:
   - Build TCP packets with custom headers and payloads.

3. **Raw Socket Communication**:
   - Send and receive raw TCP packets for experimentation and learning.


## Run
- run the receiver first, it requires root privileges:
```cli
sudo cargo run --bin receiver
```
- Then, run sender in order to send a TCP packet:
```cli
sudo cargo run --bin sender
```
## Future Improvements.
- Add support for simulating TCP three-way handshakes.
