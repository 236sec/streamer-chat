## Problem Statement
The project requires a real-time backend to eventually aggregate and broadcast chat events from various streaming platforms. Currently, no backend exists, so we need a foundational scaffold that supports WebSocket connections, can be tested automatically, and is architecturally sound for future growth.

## Solution
A basic Rust backend using Tokio and Axum that accepts WebSocket connections and broadcasts ping/pong messages. The backend will be structured using clean architecture principles within a single crate and will be built using Test-Driven Development (TDD) to ensure reliability from day one.

## User Stories
1. As a system component, I want to connect to a WebSocket endpoint, so that I can establish a real-time communication channel.
2. As a connected client, I want to send a ping message and receive a pong response, so that I can verify the connection is active and responsive.

## Implementation Decisions
- **Frameworks**: `tokio` for the async runtime and `axum` for the web framework and WebSocket handling.
- **Architecture**: A single Rust crate structured into `domain`, `application`, and `infrastructure` modules to enforce separation of concerns.
- **State Management**: `tokio::sync::broadcast` combined with a shared `Arc<AppState>` passed to Axum handlers to manage concurrent connections and message broadcasting.
- **Message Format**: JSON payloads (e.g. `{"type": "ping"}`) for WebSocket messages to align with future chat event structures.

## Testing Decisions
- **Testing Seam**: The primary test will be an integration test (`tests/websocket_test.rs`) that starts the Axum server on a random port and uses a test client (`tokio-tungstenite`) to perform a real network connection.
- **TDD Flow**: The test will be written first to fail (verifying the server isn't running), then the implementation will be built to make it pass.

## Out of Scope
- Integration with Redis (to be added when multi-node scaling or cross-service pub-sub is needed).
- Connecting to actual external platforms (Twitch, YouTube, Kick).
- Authentication for the WebSocket endpoint (the widget endpoint is public as per architecture rules).
- Complex message routing (all connections simply receive the broadcasted pong for now).

## Further Notes
- This ticket establishes the pattern for how the Rust backend will be tested and structured going forward.
