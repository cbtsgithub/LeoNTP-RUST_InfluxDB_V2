# LeoNTP Data Query Tool

> **Acknowledgment:**  
> This project was inspired by the great work of the [LeoNTP (Leo BODNAR) & UPTRONICS team] (https://leontp.com/firmware/ & https://store.uputronics.com) and their **amazing LeoNTP time server**.  
> A huge thank you to them for providing such a solid and reliable time synchronization solution.

## 📌 Purpose

This Rust-based tool queries a **LeoNTP time server** for its status and statistics and optionally sends the retrieved data to an **InfluxDB V2** database for monitoring and analysis.  
It is designed for system administrators, time synchronization enthusiasts, or anyone wanting to collect NTP statistics in a modern and automated way.

---

## ⚙ Features

- Connects to a LeoNTP server via TCP/UDP and retrieves status.
- Displays the collected data in the console (if enabled).
- Sends the same data to InfluxDB (if enabled).
- Configurable via an easy-to-edit `.ini` configuration file.

---

## 📥 Installation

### **1. Clone the repository**
```bash
git clone https://github.com/yourusername/LeoNTP_QUERY.git
cd LeoNTP_QUERY
```

### **2. File structure**
```bash
LeoNTP_QUERY/
├── Cargo.toml
├── Makefile
├── LeoNTP-config.ini       # Configuration file
├── src/
│   ├── LeoNTP-main_full.rs # Main program
│   └── config.rs           # Configuration loader
└── target/                 # Created automatically after build
```

### **3. Install Rust (if not already installed)**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### **4. Build the project:**
```bash
cargo build --release --bin main_full
```
### **or:**

**Compile:**
```bash
make build
```

**Run the program:**
```bash 
make run
```

**Clean:**
```bash 
make clean
```

**Compile and run with debug:**
```bash
make debug
```

**Complete rebuild:**
```bash
make rebuild
```
### **5. 📂 Configuration**

All configuration is stored in LeoNTP-config.ini at the root of the project.
You can edit this file to match your environment.

Example:
```bash
[LEONTP]
IPADDR = 192.168.1.50
PORTNUM = 123

[INFLUXDB]
HOST = 192.168.1.200
PORT = 8086
TOKEN = my_super_secret_token
BUCKET = leontp_data
ORG = my_org_name

[OPTIONS]
SHOW_STATS = true
SEND_TO_INFLUXDB = true
```

🔧 Parameters to Customize
```bash
Section	Parameter	Description
[LEONTP]	IPADDR	IP address of your LeoNTP time server.
	PORTNUM	Port number (usually 123 for NTP).
[INFLUXDB]	HOST	IP or hostname of your InfluxDB instance.
	PORT	InfluxDB port (default: 8086).
	TOKEN	API token for authentication with InfluxDB.
	BUCKET	Target bucket where data will be stored.
	ORG	Organization name in InfluxDB.
[OPTIONS]	SHOW_STATS	true to display statistics in the console, false to disable output.
	SEND_TO_INFLUXDB	true to send data to InfluxDB, false to disable sending.
```

### **6. 🖥 Output Types**
Console Output (when ```bash SHOW_STATS = true```)

Example:
```bash
===== LeoNTP Statistics (192.168.15.123) =====
UTC time       : 2025-08-14 09:53:47.1
NTP time       : 3964154028
Uptime         : 5196545 s (60.15 days)
NTP requests   : 2126367972
Mode 6 requests: 0
GPS lock time  : 92529 s (1.07 days)
GPS flags      : 1
Active satellites: 11
Firmware ver.  : 97793.36
Serial number  : 253
==========================================
```

InfluxDB Output (when ```bash SEND_TO_INFLUXDB = true```)

Data is written to InfluxDB in the following format:
Measurement	Tags	Fields
leontp	server	uptime, packets_sent, packets_received, sync_status

### **Example query in InfluxDB v2:**
```bash
from(bucket: "DB_LEONTP")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r["_measurement"] == "Measurements")
  |> filter(fn: (r) => r["_field"] == "Nb_NTP_Requests")
  |> filter(fn: (r) => r["host"] == "NTP01")
  |> derivative(unit: 1s, nonNegative: true, columns: ["_value"], timeColumn: "_time")  
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
  |> yield(name: "mean")
```

### **7. 🚀 Usage**
Run the binary
```bash
./target/release/main_full
```

Make sure LeoNTP-config.ini is present in the same directory as the binary.
### **📜 License**

This project is released under the MIT License.


---
