# LastMile

A self-hosted web application for tracking independent delivery courier shifts, and other related data. Built with Rust and SurrealDB, it helps drivers monitor earnings, hours worked, mileage, maintenance, and calculate real-time statistics across all their shifts.

## Features

- **Shift Tracking**: Start and end shifts with odometer readings, earnings, tips, and gas costs
- **Maintenance Task Tracking**: Stay up to date on your vehicle maitenance with simple to program alerts
- **Real-time Stats**: View earnings, hours worked, hourly rate, and miles driven
- **Flexible Filtering**: Filter shifts by month, all time, or custom date ranges
- **Editable Data**: Click any field in the table to edit shift details inline
- **CSV Export**: Export all shift data to CSV format
- **Dark Mode**: Toggle between light and dark themes
- **Responsive Design**: Works on desktop and mobile devices

## Technology Stack

- **Backend**: Rust with Axum web framework
- **Database**: SurrealDB with embedded RocksDB backend
- **Frontend**: Vanilla JavaScript, HTML5, CSS3

## Usage

### Local Development

1. Install Rust: https://rustup.rs/
2. Clone the repository
3. Run the application:
```bash
   cargo run --release
```
4. Open your browser to http://localhost:3000

The database will be created automatically in the `./data` directory.

If you have Surreal installed, and you would like to backup the database for any reason, you can run : `surreal export --endpoint file://./data --namespace lastmile --database main export.surql` from the project directory.

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

Access the application at http://localhost:3000

### Docker Compose

For the easiest deployment:
```bash
docker-compose up -d
```

The application will be available at http://localhost:3000 with persistent data storage.

See [docker-compose.yaml](docker-compose.yaml) for further configuration.

## Configuration

Environment variables (optional):

- `DATABASE_PATH`: Database storage location (default: `./data`)
- `PORT`: SurrealDB port (default: `3000`)
- `TZ`: Timezone (default: `America/Chicago`)
- `RUST_LOG`: Log level (`error`, `warn`, `info`, `debug`, `trace`)

## Future Work
- [x] Convert database from MySQL into SurrealDB
  - [x] Have SurrealDB build and run in a local context, no server
- [x] Make filtered views filter from the backend instead of the frontend
- [x] Improve frontend caching rules, and implement gzip
- [ ] Improve CSS styling rules
  - [ ] Focus more on mobile experience
    - [ ] Improve table view
  - [ ] Investigate desktop site improvements
    - [x] Improve logo and header styling
    - [ ] Improve table scrolling
- [x] Track maintenance tasks
  - [x] Create maintenance page and backend systems
  - [x] Calculate remaining milage per maintenance task
- [x] Allow editing of shift TimeDates
- [x] Allow deletion of shifts
  - [x] Create shift deletion endpoint
  - [x] Add shift deletion to UI
- [ ] Improve tests
  - [ ] Look into locking teardown endoint behind test flag
  - [ ] Make e2e tests preserve original database
  - [ ] Save test outputs to .log files so actions script can upload artifacts on failure
