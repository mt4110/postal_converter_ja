# Postal Converter JA Launcher 🚀

A CLI tool to manage and launch Postal Converter JA components (Docker, Crawler, API, Frontend).
Built with Go and Bubbletea, providing a rich TUI (Text User Interface).

## Requirements

- Go 1.21+ (Included in the Nix environment)
- Docker & Docker Compose

## Usage

Run the following command from the project root:

```bash
cd launcher
go run main.go
```

Or build and run:

```bash
cd launcher
go build -o postal-launcher
./postal-launcher
```

> [!NOTE]
> The launcher enforces a specific execution order:
> **Databases -> Crawler/API -> Frontend**.
> Subsequent steps are locked until prerequisites are met.

## Features

- **Start Databases**: Runs `docker-compose up -d` to start MySQL/PostgreSQL.
- **Start Crawler**: Opens a new terminal window and runs the Crawler.
- **Start API Server**: Opens a new terminal window and runs the API Server.
- **Start Frontend**: Opens a new terminal window and runs the Next.js Frontend.
- **Stop Databases**: Runs `docker-compose down` to stop databases.

## For Developers

This launcher uses the `bubbletea` framework.
To add new commands, edit `choices` and `executeSelection` in `main.go`.
