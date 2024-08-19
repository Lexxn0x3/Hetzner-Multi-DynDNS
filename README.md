# Hetzner Multi DynDNS

## Introduction

`Hetzner Multi DynDNS` is a tool designed to help you manage your DNS records hosted with Hetzner. If you are hosting services at home and need to update your DNS records whenever your IP address changes, this tool automates that process. It supports updating multiple DNS records across different zones, making it ideal for users with multiple domains.

## Features

- **Dynamic DNS Updates**: Automatically updates your DNS records when your IP address changes.
- **Multi-Zone Support**: Manage DNS records across multiple zones (domains).
- **Efficient Zone Caching**: Uses a local cache to reduce API calls by storing zone details.
- **Configurable Logging**: Set log levels to control the verbosity of the output.

## Installation

### Prerequisites

- Rust and Cargo installed on your system. If not, you can install Rust from [here](https://www.rust-lang.org/tools/install).
- A Hetzner account with API access to manage DNS zones.

### Clone and Build
Alternatively, clone the repository and build the project:

```sh
git clone https://github.com/yourusername/hetzner_multi_dyn_dns.git
cd hetzner_multi_dyn_dns
cargo build --release
```

### Configuration
Create a config.toml file in the root of your project or in the directory where the binary will run. This file should contain your Hetzner API token and the DNS records you want to manage.

Example config.toml

```sh
# Hetzner API token
api_token = "your_hetzner_api_token_here"

# Interval in seconds between IP checks
interval_secs = 300

# Log level: "info", "debug", "trace", or "error"
log_level = "info"

# DNS records to be updated
[[records]]
record_id = "your_record_id_1"
name = "subdomain1"
record_type = "A"
ttl = 300
zone_id = "your_zone_id_1"

[[records]]
record_id = "your_record_id_2"
name = "subdomain2"
record_type = "A"
ttl = 300
zone_id = "your_zone_id_2"
```
#### Configuration Details
- api_token: Your Hetzner API token used to authenticate API requests.
- interval_secs: The time in seconds between each IP check and potential DNS update.
- log_level: The verbosity of the logs. Choose between info, debug, trace, or error.
- records: A list of DNS records that you want to manage. Each record includes:
- record_id: The ID of the DNS record in Hetzner's system.
- name: The subdomain or name of the record (without the domain).
- record_type: The type of DNS record (e.g., A, AAAA, CNAME).
- ttl: The time-to-live for the DNS record.
- zone_id: The ID of the zone (domain) to which the record belongs.

### Usage
Once you have your config.toml set up, simply run the binary. The program will continuously monitor your public IP and update the specified DNS records if the IP changes.

### Running the Binary
```sh
./hetzner_multi_dyn_dns
```
This will start the process based on the configuration provided in config.toml.
