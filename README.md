# LocalChain

## Overview

A simple UI tool to run and manage local blockchain nodes.

- [x] Multiple chains
- [x] Ethereum, using [Anvil](https://getfoundry.sh/anvil/overview/)

## Development

Monorepo with an Axum server and a Leptos client.

### Requirements
- Rust toolchain (stable)
- Trunk (`cargo install trunk`)
- wasm32 target (`rustup target add wasm32-unknown-unknown`)

### Project layout
- `server/`: Axum HTTP server serving API and static client assets
- `client/`: Leptos CSR app built with Trunk to `client/dist`
- `shared/`: Shared types between server and client

### First-time setup
```bash
# from repo root
rustup target add wasm32-unknown-unknown
cargo install trunk
```

### Build the client
```bash
cd client
trunk build --release
```

This produces `client/dist` with `index.html`, JS, and wasm artifacts.

### Run the server
```bash
# from repo root
cargo run -p server
```

Open `http://127.0.0.1:3000` in your browser.

The server serves:
- `/api/health` → `ok`
- `/` → `client/dist/index.html` if present; otherwise a placeholder page

### Dev mode (optional)
In two terminals:
```bash
# Terminal A: client with hot reload on a different port (serves its own dev server)
cd client && trunk serve --port 8080

# Terminal B: server API only
cargo run -p server
```

TODO: 
- Configure server base url for client; this is needed to enable dev mode
