# Plan: UDP Multicast Network Discovery System

## TL;DR
Build an async UDP multicast discovery module in `src/udp_discovery_server.rs` that:
- Listens on multicast group 239.1.1.1:5900 to receive peer discovery packets
- Sends on-demand discovery beacons to the same multicast group
- Maintains a peer registry (node name, IP, HTTP port, system_type)
- Integrates with existing system via broadcast hub (peer events) and dedicated request channels (commands: GetPeers, SendDiscovery)
- Uses Tokio for async I/O with mpsc and broadcast channels for inter-module communication

## Requirements Summary
- **Multicast address**: 239.1.1.1:5900
- **Peer data fields**: node_name, ip_address, http_port, system_type
- **Broadcasting mode**: On-demand only (triggered by command)
- **Peer expiration**: None (keep indefinitely)
- **Message integration**: Dual approach (broadcast for discovery events + separate channels for commands)

## Steps

### Phase 1: Core Data Structures (Foundation)

1. **Define DiscoveryMessage struct** — serializable with serde/bincode
   - Contains: node_name (String), ip_address (IpAddr), http_port (u16), system_type (String)
   - Used for both sending/receiving multicast packets
   - Suggestion: Use bincode for compact binary encoding (faster over network)

2. **Define PeerInfo struct** — representation of a discovered peer
   - Fields: node_name, ip_address, http_port, system_type
   - Timestamp of last discovery (for future expiration logic if needed)
   - Implement Clone for sharing via broadcast

3. **Define DiscoveryCommand enum** — commands from other modules to discovery
   - `GetPeers` variant with oneshot response channel returning Vec<PeerInfo>
   - `SendDiscoveryBeacon` variant with optional oneshot for confirmation
   - `ClearPeers` (utility variant for testing/reset)

4. **Define DiscoveryEvent enum** — events broadcast to modules
   - `PeerDiscovered(PeerInfo)` — when new peer found
   - `PeerUpdated(PeerInfo)` — when peer info refreshed
   - These will be wrapped in existing Message enum as a variant

5. **Integrate with Message enum** (types/src/lib.rs)
   - Add `Discovery(DiscoveryEvent)` variant to Message enum so broadcast hub carries discovery events

### Phase 2: UDP Socket & Network I/O (Network Layer)

6. **Create DiscoveryServer struct** — main server instance
   - Fields:
     - `peers: Arc<RwLock<HashMap<String, PeerInfo>>>` — thread-safe peer registry keyed by node_name
     - `socket: UdpSocket` — shared async UDP socket bound to 239.1.1.1:5900
     - `local_peer_info: PeerInfo` — this node's identity (loaded from config or discovered at startup)
     - `broadcast_sender: Sender<Message>` — clone of hub broadcast channel
     - `command_rx: UnboundedReceiver<DiscoveryCommand>` — receiver for discovery commands

7. **Implement socket setup (bind & join multicast group)**
   - Bind UDP socket to 0.0.0.0:5900 (listen on all interfaces on port 5900)
   - Join multicast group 239.1.1.1 on all available network interfaces
   - Set socket options: SO_REUSEADDR=1 for address reuse, SO_REUSEPORT (platform-specific)
   - Handle platform differences (Windows vs Unix) for SO_REUSEPORT if needed

8. **Implement receive task** — listen loop that accepts packets
   - Read incoming UDP packets (1024+ byte buffer for safety)
   - Deserialize from bincode (with error handling for malformed packets)
   - Check if packet is from self (compare node_name) — discard if yes
   - Call `handle_discovery_packet(DiscoveryMessage)` to process

9. **Implement handle_discovery_packet** — update peer registry
   - Extract PeerInfo from DiscoveryMessage
   - Check if peer already in registry (key: node_name)
   - If new: insert + broadcast `DiscoveryEvent::PeerDiscovered(peer_info)`
   - If existing: update timestamp + broadcast `DiscoveryEvent::PeerUpdated(peer_info)` only if fields changed
   - Use Arc<RwLock> for safe concurrent access

### Phase 3: Command Handling & API (Module Interface)

10. **Implement command handler loop** — process commands from other modules
    - Run in select! with receive task
    - Match on DiscoveryCommand::
      - `GetPeers(oneshot_tx)`: Acquire read lock, clone peers, send via oneshot
      - `SendDiscoveryBeacon(oneshot_tx)`: Serialize local_peer_info to DiscoveryMessage, send UDP broadcast packet, respond via oneshot if Some
      - `ClearPeers`: Clear the registry (utility, for testing)

11. **Implement send_discovery_packet** — send multicast beacon (on-demand)
    - Serialize local_peer_info as DiscoveryMessage using bincode
    - Send UDP packet to 239.1.1.1:5900 from socket
    - Include error handling (socket may fail temporarily, retry or log)
    - Return Result for caller to handle

12. **Create public async entry point: run()** — main event loop
    - Spawns receive task and command loop in select!
    - Selects between:
      - `socket.recv_from()` → handle_discovery_packet()
      - `command_rx.recv()` → command handler
    - Runs indefinitely until channel closes or fatal error

### Phase 4: Main Integration (System Orchestration)

13. **Update main.rs to initialize discovery module**
    - Add DiscoveryCommand channel: `let (discovery_tx, discovery_rx) = tokio::sync::unbounded_channel();`
    - Create DiscoveryServer instance with local_peer_info from config (or defaults)
    - Spawn: `tokio::spawn(discovery_server.run())`
    - Store `discovery_tx` in AppState for handlers to access
    - *Depends on: Phase 1 (data structures), Phase 2 (DiscoveryServer impl)*

14. **Update types/src/lib.rs Message enum** to include Discovery variant
    - Add `Discovery(DiscoveryEvent)` variant
    - *Depends on: Phase 1*

15. **Create example HTTP endpoint** (optional, for testing) — GET /peers
    - Use discovery_tx to send GetPeers command
    - Wait for oneshot response
    - Return JSON list of discovered peers
    - *Depends on Phase 3, 4*

### Phase 5: Testing & Validation (Verification)

16. **Add unit tests in udp_discovery_server.rs**
    - Test DiscoveryMessage serialization/deserialization
    - Test peer registry insert/update logic
    - Test command dispatch (GetPeers, SendDiscoveryBeacon)
    - Run via: `cargo test --release`

## Relevant Files
- `src/udp_discovery_server.rs` — main implementation (all data structures, DiscoveryServer, run loop)
- `types/src/lib.rs` — extend Message enum with DiscoveryEvent variant
- `src/main.rs` — initialize and spawn discovery module, pass broadcast_sender + create discovery_tx
- `src/http.rs` (optional) — add /peers endpoint for testing
- `src/config.rs` — extract/provide node_name, http_port for local_peer_info
- `Cargo.toml` — ensure bincode is available (for serialization)

## Architecture Decisions

### Data Serialization
- Use **bincode** for compact binary encoding (faster, smaller packets)
- Fallback to serde_json for debugging if needed
- Choice: bincode for production, allows JSON in comments for documentation

### Thread Safety
- `Arc<RwLock<HashMap>>` for peer registry: multiple readers (queries), single writer (updates)
- RwLock preferred over Mutex for read-heavy workloads
- All broadcast/oneshot channels are Tokio's (safe for async)

### On-Demand vs Periodic
- Confirmed: on-demand only (no background beacon thread)
- Other modules call SendDiscoveryBeacon when they want to advertise themselves
- Scales better for large networks, reduces network churn

### No Peer Expiration
- Peers persist indefinitely until explicitly cleared
- Allows slow/intermittent peers to remain discoverable
- Future enhancement: add optional TTL/expiration if needed

### Dual Message Integration
- Broadcast events (`DiscoveryEvent::PeerDiscovered/Updated`) → all modules notified
- Dedicated channels for commands (`GetPeers`, `SendDiscoveryBeacon`) → explicit request-response
- Mirrors existing ConfigRequest pattern, familiar to codebase

## Verification Steps

1. **Compilation**: `cargo check` passes with no errors or warnings
2. **Formatting**: `cargo fmt` and `cargo check` again
3. **Unit tests**: `cargo test --release` all tests pass

## Future Considerations

1. **Peer Expiration** — Currently no TTL; can add optional timeout in Phase 3 if stale peers become a problem
   - Recommendation: Monitor first, add expiration if peer list grows unbounded
   
2. **Cross-Subnet Routing** — UDP multicast is routing-dependent
   - Multicast class D (239.x.x.x) usually limited to LAN without IGMP Snooping/PIM
   - If cross-router needed: may require IGMP/PIM config on network or fall back to unicast discovery
   - Recommendation: Test in target network; if doesn't work, switch to unicast broadcast (255.255.255.255:port) or TCP tracker
   