# Understory MCP Server

A Rust-based [Model Context Protocol](https://modelcontextprotocol.io/) (MCP) server for the [Understory API](https://developer.understory.io/). Built with [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) using stdio transport.

## Prerequisites

- Rust toolchain (1.80+)
- An Understory integration key ([how to create one](https://developer.understory.io/docs/usage/authentication/integration-keys))

## Setup

### 1. Build

```bash
cargo build --release
```

### 2. Configure credentials

Create a `.env.mcp` file (gitignored):

```
UNDERSTORY_CLIENT_ID=your_client_id
UNDERSTORY_CLIENT_SECRET=your_secret_key
```

### 3. Add to Claude Code

Add the server to your Claude Code configuration (`~/.claude.json` or `.claude/settings.json`):

```json
{
  "mcpServers": {
    "understory": {
      "type": "stdio",
      "command": "/path/to/target/release/understory-mcp",
      "env": {
        "UNDERSTORY_CLIENT_ID": "your_client_id",
        "UNDERSTORY_CLIENT_SECRET": "your_secret_key"
      }
    }
  }
}
```

Or using an env file:

```json
{
  "mcpServers": {
    "understory": {
      "type": "stdio",
      "command": "/path/to/target/release/understory-mcp",
      "envFile": "/path/to/.env.mcp"
    }
  }
}
```

Then restart Claude Code or run `/mcp` to connect.

## Authentication

The server uses OAuth2 client credentials flow to authenticate with the Understory API. It automatically fetches and refreshes access tokens using the provided client ID and secret. The requested scopes are:

`openid booking.read booking.write event.read experience.read marketing.read order.read webhook.read webhook.write`

Ensure your integration key has the necessary permissions for the tools you intend to use.

## Tools

### Bookings

| Tool | Description |
|---|---|
| `get_bookings` | List all bookings with optional date filtering and sorting |
| `get_booking` | Get a booking by ID |
| `get_tickets` | Get all tickets for a booking |
| `create_booking` | Create a new booking for an event |

### Events

| Tool | Description |
|---|---|
| `get_events` | List events with optional date range and experience filtering |
| `get_event` | Get an event by ID |

### Event Availability

| Tool | Description |
|---|---|
| `get_event_availability` | Get availability for a single event |
| `list_event_availability` | List availability for events belonging to an experience |

### Experiences

| Tool | Description |
|---|---|
| `get_experiences` | List all experiences |
| `get_experience` | Get an experience by ID |
| `get_information_requests` | Get additional questions for an experience's booking flow |
| `get_ticket_variants` | Get ticket pricing options for an experience |

### Orders

| Tool | Description |
|---|---|
| `get_orders` | List orders with optional date filtering and sorting |
| `get_order` | Get an order by ID |
| `get_line_items` | Get line items for an order |
| `get_transactions` | Get transactions for an order |
| `get_refunds` | Get refunds for an order |

### Marketing

| Tool | Description |
|---|---|
| `get_marketing_consents` | List marketing consents collected through checkouts |

### Webhooks

| Tool | Description |
|---|---|
| `list_webhook_subscriptions` | List all webhook subscriptions |
| `get_webhook_subscription` | Get a webhook subscription by ID |
| `create_webhook_subscription` | Create a new webhook subscription |
| `update_webhook_subscription` | Update a webhook subscription |
| `delete_webhook_subscription` | Delete a webhook subscription |

### Test

| Tool | Description |
|---|---|
| `who_am_i` | Verify authentication and get current user/company info |

## Project structure

```
src/
├── main.rs      # Entry point, reads env vars, starts stdio server
├── client.rs    # HTTP client with OAuth2 token management
└── server.rs    # MCP tool definitions (24 tools)
```

## API Reference

See the full Understory API documentation at [developer.understory.io](https://developer.understory.io/).
