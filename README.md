# Callux - Calendar Agenda Utility

A fast, reliable Rust CLI utility that displays Google Calendar agenda
information, optimized for Waybar integration on Hyprland systems.

## Features

- **Multi-format output**: JSON (for Waybar), human-readable, and colored
  terminal output
- **Google Calendar integration** with OAuth2 authentication
- **Caching system** for performance and offline capability
- **Multi-calendar support** with filtering
- **Configurable display options**
- **Timezone-aware** date handling

## Installation

`bash cargo build --release`

`sudo cp target/release/callux /usr/local/bin/`

## Quick Start

1. **Initialize configuration:**

   `bash callux config init`

2. **Set up Google Calendar credentials:**
   - Go to [Google Cloud Console](https://console.developers.google.com/)
   - Create a new project or select an existing one
   - Enable the Google Calendar API
   - Create OAuth 2.0 credentials (Desktop application)
   - Download the credentials JSON file
   - Replace the content in `~/.config/callux/credentials.json`

3. **Authenticate:**

   `bash callux auth`

4. **View your agenda:**

   `bash callux agenda`

## Usage

### Show Calendar Agenda

````bash # Default format (human-readable) callux agenda

# JSON format for Waybar callux agenda --format json

# Colored terminal output callux agenda --format colored

# Limit number of events callux agenda --limit 5

# Look ahead 14 days callux agenda --days 14 ```

### List Available Calendars

```bash callux list-calendars ```

### Configuration Management

```bash # Show current configuration callux config show

# Initialize default configuration callux config init ```

## Waybar Integration

Add this to your Waybar configuration:

```json { "custom/calendar": { "format": "{} 📅", "return-type": "json",
"exec": "callux agenda --format json", "interval": 300, "tooltip": true,
"on-click": "callux agenda --format colored", "max-length": 50 } } ```

## Configuration

The configuration file is located at `~/.config/callux/config.toml`:

```toml [auth] credentials_path = "~/.config/callux/credentials.json"
token_cache_path = "~/.config/callux/token.json"

[cache] ttl_seconds = 300        # Cache TTL in seconds max_entries = 1000
# Maximum cache entries

[display] max_events = 10          # Default number of events to show
date_format = "%Y-%m-%d %H:%M"  # Date format string timezone = "local"       #
Timezone handling

[[calendars]] id = "primary"           # Calendar ID from Google name =
"Personal"        # Display name color = "#1976d2"        # Color for terminal
output enabled = true           # Whether to include this calendar ```

## Output Formats

### JSON (Waybar)

```json { "text": "Meeting at 14:00", "tooltip": "Today: Friday, July 4th,
2025\n\n• Meeting at 2:00 PM\n• Dentist at 4:30 PM", "class":
"calendar-single", "percentage": 50 } ```

### Human-readable

``` Friday, July 4, 2025 14:00: Team Meeting 16:30: Dentist Appointment

Saturday, July 5, 2025 All day: Weekend Trip ```

### Colored Terminal

Similar to human-readable but with ANSI color codes for better terminal
display.

## Dependencies

- **google-calendar3**: Google Calendar API client
- **yup-oauth2**: OAuth2 authentication
- **clap**: CLI argument parsing
- **tokio**: Async runtime
- **moka**: High-performance caching
- **chrono**: Date and time handling
- **serde**: Serialization/deserialization

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details.
````
