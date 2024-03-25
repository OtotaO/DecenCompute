# DecenCompute
Decentralized Computing 

```markdown
# Distributed Compute Program

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Overview

Distributed Compute Program is a Rust application that enables distributed computation over a network. It allows clients to send computation tasks to a server, which executes them and returns the results. This program is designed to be flexible, scalable, and easy to use.

## Features

- Secure communication using RSA encryption.
- Serialization and compression of data for efficient network transfer.
- Support for various computation tasks through user-defined functions.
- Heartbeat mechanism to detect client disconnections.
- Graceful shutdown handling.

## Installation

1. Make sure you have Rust and Cargo installed on your system.
2. Clone this repository to your local machine:

   ```bash
   git clone https://github.com/yourusername/distributed_compute_program.git
   ```

3. Navigate to the project directory:

   ```bash
   cd distributed_compute_program
   ```

4. Build the project:

   ```bash
   cargo build --release
   ```

## Usage

1. Start the server:

   ```bash
   cargo run --release --bin server
   ```

2. Connect clients to the server using TCP/IP.

3. Send computation tasks to the server and receive results.

## Configuration

- Modify the `HOST`, `PORT`, and `BUFFER_SIZE` constants in `src/main.rs` to customize server settings.
- Replace the placeholder comments in `src/main.rs` with actual implementations for loading private and public keys.

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

Distributed Compute Program is licensed under the [MIT License](LICENSE).

## Acknowledgements

This project was inspired by fomoats ( Fear of moats )
```
