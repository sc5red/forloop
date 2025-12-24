# forloop

**Motto: "Every request is the first."**

A desktop web browser whose primary and non-negotiable guarantee is:
**Websites receive absolutely zero identifying or persistent data.**

## Philosophy

- Stateless by design
- Memory is a vulnerability
- Convenience is the enemy
- If a site breaks, that is acceptable

## Quick Start

```bash
# Linux build (primary target)
cd build && ./build.sh --release

# Run with fresh state
./forloop --new-loop

# Kill all state and exit
./forloop --kill-all-state
```

## Documentation

- [Threat Model](docs/THREAT_MODEL.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Build Instructions](docs/BUILD.md)
- [Engine Patches](docs/ENGINE_PATCHES.md)
- [Comparison](docs/COMPARISON.md)
- [Limitations](docs/LIMITATIONS.md)

## License

GPLv3 - See LICENSE file

