# Configuration Schema

This document defines the **TOML** configuration format required by the network simulator. All fields are mandatory unless explicitly marked as optional. The configuration is parsed at startup; any validation error aborts execution.

## Top‑Level Structure
```toml
# Path: config.toml
[simulation]
# Global simulation parameters
mtu = 1500               # Default MTU for links (bytes)
seed = 12345             # Optional RNG seed for deterministic simulations

[interfaces]
# Definition of the two TUN interfaces used by the simulator
tun_a = "tunA"
 tun_b = "tunB"

# Ingress routers – packets arriving on the TUN interface are injected here.
# These must correspond to router IDs defined in the topology section.
[tun_ingress]
 tun_a_ingress = "Rx0y0"   # Router that receives packets from tunA
 tun_b_ingress = "Rx5y5"   # Router that receives packets from tunB

# -------------------------------------------------------------------
# Topology – a list of routers and links that make up the fabric.
# -------------------------------------------------------------------
```

## Router Definitions
Routers are identified by a **router ID** of the form `Rx{X}y{Y}` where `X` and `Y` are integers in the range `0..=5` for the default 6×6 fabric.

```toml
[routers]
# No additional per‑router fields are required at the moment.
# The presence of a router ID in this table signals that the router exists.
Rx0y0 = {}
Rx0y1 = {}
# ... continue for all routers you need.
```

## Link Definitions
Each link connects two routers. Links are represented by a TOML table whose **name** is the two router IDs concatenated with an underscore (`_`). The order does **not** matter; `Rx0y0_Rx0y1` and `Rx0y1_Rx0y0` refer to the same bidirectional link. If both names appear, they must have identical parameters.

```toml
# Example link between Rx0y0 and Rx0y1
[links.Rx0y0_Rx0y1]
mtu = 1500            # Optional – overrides global mtu for this link
delay_ms = 10         # Base propagation delay (milliseconds)
jitter_ms = 2         # Max random jitter added to delay (ms)
loss_percent = 0.1    # Packet loss probability (0‑100)
load_balance = true   # If true, a per‑packet counter participates in the hash for load‑balancing.

# Link to the TUN interfaces (treated as virtual routers "tunA" and "tunB")
[links.tunA_Rx0y0]
mtu = 1500
delay_ms = 1
loss_percent = 0.0
load_balance = false
```

### Link Fields
| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `mtu` | integer | Maximum Transmission Unit for the link. Overrides the global `simulation.mtu`. | simulation.mtu |
| `delay_ms` | integer | Fixed propagation delay in milliseconds. | 0 |
| `jitter_ms` | integer | Maximum additional random delay (± jitter) in ms. | 0 |
| `loss_percent` | float | Probability (0‑100) that a packet is dropped on this link. | 0 |
| `load_balance` | bool | When `true`, the per‑link counter is added to the 5‑tuple hash, enabling per‑packet load‑balancing. | false |

## Validation Rules
1. **Router existence** – every router ID referenced in a link must be defined in the `[routers]` table.
2. **Bidirectional consistency** – if both `A_B` and `B_A` sections exist, all fields must match.
3. **Unique links** – duplicate link definitions (same unordered pair) are not allowed.
4. **TUN interface names** – `tun_a` and `tun_b` must be distinct and correspond to existing system TUN devices (or will be created by the simulator).
5. **Ingress routers** – `tun_ingress.tun_a_ingress` and `tun_ingress.tun_b_ingress` must reference valid router IDs.

## Example Full Configuration
```toml
[simulation]
mtu = 1500
seed = 42

[interfaces]
tun_a = "tunA"
tun_b = "tunB"

[tun_ingress]
tun_a_ingress = "Rx0y0"
tun_b_ingress = "Rx5y5"

[routers]
# Define the 6x6 fabric (only a subset shown for brevity)
Rx0y0 = {}
Rx0y1 = {}
Rx0y2 = {}
Rx1y0 = {}
Rx5y5 = {}
# ... add the rest as needed

[links.Rx0y0_Rx0y1]
mtu = 1500
delay_ms = 5
jitter_ms = 1
loss_percent = 0.0
load_balance = false

[links.Rx0y1_Rx0y2]
mtu = 1500
delay_ms = 5
jitter_ms = 1
loss_percent = 0.0
load_balance = false

[links.tunA_Rx0y0]
mtu = 1500
delay_ms = 1
loss_percent = 0.0
load_balance = false

[links.tunB_Rx5y5]
mtu = 1500
delay_ms = 1
loss_percent = 0.0
load_balance = false
```

Save this file as `config.toml` in the project root and run the simulator with `cargo run -- -c config.toml`.
