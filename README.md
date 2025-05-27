# MiniTectonic-RS

A simplified, multitenant block storage system inspired by Meta's Tectonic Filesystem. This prototype demonstrates the core principles of scalable, multitenant block storage with a focus on data integrity through checksums.

## Features

- Multitenant blob storage with tenant isolation
- SHA-256 checksum verification for data integrity
- Efficient metadata storage using sled
- Command-line interface for all operations
- Support for large files through streaming I/O

## Installation

1. Make sure you have Rust installed (https://rustup.rs/)
2. Clone this repository
3. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

The CLI supports the following commands:

### Register a Tenant

```bash
mini-tectonic-rs register-tenant -t posts
```

### List Tenants

```bash
mini-tectonic-rs list-tenants
```

### Store a Blob

```bash
mini-tectonic-rs put -t posts -f path/to/file.txt
```

### Retrieve a Blob

```bash
# To file
mini-tectonic-rs get -t posts -b <blob-id> -o output.txt

# To stdout
mini-tectonic-rs get -t posts -b <blob-id>
```

### List Blobs

```bash
mini-tectonic-rs list-blobs -t posts
```

### Delete a Blob

```bash
mini-tectonic-rs delete -t posts -b <blob-id>
```

## Storage Layout

- `storage/` - Root storage directory
  - `chunks/` - Blob storage
    - `{uuid}.blob` - Blob files
    - `{uuid}.blob.chk` - Checksum files
  - `metadata/` - sled database for metadata

## Architecture

The system consists of several key components:

1. **CLI Layer**: Command-line interface using clap
2. **Coordinator**: Orchestrates operations between components
3. **Chunk Store**: Manages blob storage and checksums
4. **Metadata Store**: Manages blob metadata using sled
5. **Tenant Manager**: Handles tenant isolation

## Development

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
cargo doc --no-deps --open
```

## Limitations

- Single-node operation only
- No replication or redundancy
- In-memory tenant management
- Basic error handling

## Future Work

- Block-level sealing and reencoding
- Multi-node deployment support
- Replication strategies
- Web UI for monitoring
- Improved error handling and recovery
- Persistent tenant management

## License

MIT License 