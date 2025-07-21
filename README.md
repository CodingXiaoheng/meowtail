# Meowtail

This project provides a small web service with DHCP and port mapping management.

## Port Mapping

Port mapping rules are stored in `portmap.toml` next to the executable. Example
configuration:

```toml
external_interface = "eth0"

[[rules]]
protocol = "tcp"
external_port = 8080
internal_ip = "192.168.1.10"
internal_port = 80
```

Rules are loaded on startup and translated into `iptables` commands. The REST
API under `/api/portmap` allows querying and updating these rules.

### REST Endpoints

- `GET /api/portmap/config` – return current configuration
- `POST /api/portmap/rule` – add a rule (fields: `protocol`, `external_port`,
  `internal_ip`, `internal_port`)
- `DELETE /api/portmap/rule` – remove a rule with the same fields
- `POST /api/portmap/interface` – set external interface for all rules

Changing the interface reapplies existing rules automatically.
