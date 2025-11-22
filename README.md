# LastMile

[![CI](https://github.com/dylan-park/LastMile/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/dylan-park/LastMile/actions/workflows/ci.yml) [![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT) [![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE) [![SurrealDB](https://img.shields.io/badge/SurrealDB-2.3-purple.svg)](https://surrealdb.com/)

A self-hosted web application for independent delivery couriers to track shifts, earnings, and vehicle maintenance. Built with Rust and SurrealDB, LastMile helps drivers stay on top of their finances and vehicle health without relying on third-party services.

## Motivation

As a delivery driver, keeping track of earnings, hours, and vehicle maintenance across multiple shifts can be tedious. Most tracking solutions either require subscriptions, share your data with third parties, or lack the flexibility needed for gig work. LastMile was built to solve this problem: a simple, privacy-focused tool that runs entirely on your own hardware, giving you complete control over your data.

## Features

- **Shift Management** - Track start/end times, odometer readings, earnings, tips, and gas costs
- **Real-time Statistics** - View earnings, hours worked, average hourly rate, and total miles driven
- **Flexible Filtering** - Filter shifts by month, all time, or custom date ranges
- **Maintenance Tracking** - Set up mileage-based maintenance reminders for oil changes, tire rotations, and more
- **Inline Editing** - Click any field in the table to edit shift details on the fly
- **CSV Export** - Export all shift data for external analysis or tax preparation
- **Dark Mode** - Easy on the eyes during late-night shifts
- **Fully Self-Hosted** - Your data stays on your machine, no cloud required
- **Responsive Design** - Works on desktop and mobile devices

## Screenshots

### Shifts View
![Shifts View](.github/screenshots/shifts_view.png)

### Maintenance Tracking
![Maintenance View](.github/screenshots/maintenance_view.png)

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.83 or higher
- Modern web browser (Chrome, Firefox, Safari, Edge)

### Local Development

1. Clone the repository:
   ```bash
   git clone https://github.com/dylan-park/LastMile.git
   cd lastmile
   ```

2. Run the application:
   ```bash
   cargo run --release
   ```

3. Open your browser to [http://localhost:3000](http://localhost:3000)

The database will be created automatically in the `./data` directory.

### Docker

Build and run with Docker:

```bash
docker build -t lastmile .
docker run -d \
  --name lastmile \
  --hostname lastmile \
  --restart unless-stopped \
  -p 3000:3000 \
  -v ./data:/app/data \
  -v ./static:/app/static:ro \
  -e DATABASE_PATH=/app/data \
  -e PORT=3000 \
  -e TZ=America/Chicago \
  -e RUST_LOG=info \
  lastmile
```

Access the application at [http://localhost:3000](http://localhost:3000)

### Docker Compose

For the easiest deployment:

```bash
docker-compose up -d
```

The application will be available at [http://localhost:3000](http://localhost:3000) with persistent data storage.

See [docker-compose.yaml](docker-compose.yaml) for configuration options.

## Configuration

Environment variables (all optional):

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_PATH` | Database storage location | `./data` |
| `PORT` | Application port | `3000` |
| `TZ` | Timezone for logs | `America/Chicago` |
| `RUST_LOG` | Log level (`error`, `warn`, `info`, `debug`, `trace`) | `info` |

## Database Backup

If you have SurrealDB CLI installed, you can backup your database:

```bash
surreal export --endpoint file://./data --namespace lastmile --database main export.surql
```

## Technology Stack

- **Backend**: Rust with [Axum](https://github.com/tokio-rs/axum) web framework
- **Database**: [SurrealDB](https://surrealdb.com/) with embedded RocksDB backend
- **Frontend**: Vanilla JavaScript, HTML5, CSS3 (no frameworks)

## Contributing

Contributions are welcome! Here's how you can help:

1. **Fork the repository** and create a new branch for your feature or bugfix
2. **Write tests** for your changes (unit tests in their related modules, integration tests in `tests/`, E2E tests in `scripts/e2e.py`)
3. **Follow the existing code style**:
   - Rust: Use `cargo fmt` and `cargo clippy`
   - JavaScript: 2-space indentation, double quotes
4. **Submit a pull request** with a clear description of your changes

### Running Tests

**Unit/Integration Tests:**
```bash
cargo test
```

**E2E Tests** (requires Selenium Grid on port 4444):
```bash
# Optionally run in a venv
pip install -r requirements.txt
pytest scripts/e2e.py -v
```

## Future Work

- [ ] Improve CSS styling rules
  - [ ] Focus more on mobile experience
    - [ ] Improve table view
  - [ ] Investigate desktop site improvements
    - [ ] Improve table scrolling
- [ ] Improve tests
  - [ ] Look into locking teardown endpoint behind test flag
  - [ ] Make E2E tests preserve original database
  - [ ] Save test outputs to .log files so actions script can upload artifacts on failure

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

Built with ❤️ for delivery drivers who want to take control of their data.
