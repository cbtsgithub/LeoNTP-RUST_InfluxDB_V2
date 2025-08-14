mod config;

use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::time::Duration;
use config::Config;

// ===== Little-endian helpers =====
fn read_u16_le(b: &[u8]) -> u16 {
    u16::from_le_bytes([b[0], b[1]])
}
fn read_u32_le(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

// ===== Unix -> date/time conversion =====
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100 + yoe / 400);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (mp + if mp < 10 { 3 } else { -9 }) as i32;
    if m <= 2 { y += 1; }
    (y, m as u32, d)
}
fn unix_to_ymdhms(t: i64) -> (i32, u32, u32, u32, u32, u32) {
    let secs_per_day = 86_400i64;
    let days = t.div_euclid(secs_per_day);
    let mut rem = t.rem_euclid(secs_per_day);
    let (y, m, d) = civil_from_days(days);
    let h = (rem / 3600) as u32; rem %= 3600;
    let min = (rem / 60) as u32;
    let s = (rem % 60) as u32;
    (y, m, d, h, min, s)
}

fn main() {
    // Résolution du chemin vers le fichier config
    let config_path = Config::resolve_path_from_args_or_exe("LeoNTP-config.ini")
        .expect("Impossible de déterminer le chemin du fichier de configuration");

    let cfg = Config::load(config_path.to_str().unwrap())
        .unwrap_or_else(|e| panic!("Erreur de lecture du fichier config {:?}: {}", config_path, e));

    let version: u8 = 4;
    let mode: u8 = 7;
    let time1970: u64 = 2_208_988_800;

    // Préparation paquet NTP
    let mut packet = [0u8; 8];
    packet[0] = (version << 3) | mode;
    packet[1] = 0;
    packet[2] = 0x10;
    packet[3] = 1;
    packet[4..8].copy_from_slice(&[0, 0, 0, 0]);

    // UDP vers serveur NTP
    let addr = format!("{}:{}", cfg.leontp.ipaddr, cfg.leontp.portnum);
    let sock = UdpSocket::bind("0.0.0.0:0").expect("bind UDP");
    sock.set_read_timeout(Some(Duration::from_millis(2500))).unwrap();
    sock.send_to(&packet, &addr).expect("sendto UDP");

    let mut buf = [0u8; 1024];
    let (len, _) = sock.recv_from(&mut buf).expect("recvfrom UDP");
    let rx = &buf[..len];
    if rx.len() < 48 {
        eprintln!("Response too short ({} bytes).", rx.len());
        return;
    }

    // Extraction des données
    let ref_ts0 = read_u32_le(&rx[16..20]) as f64 / 4_294_967_296.0;
    let ref_ts1 = read_u32_le(&rx[20..24]) as u64;
    let uptime = read_u32_le(&rx[24..28]);
    let ntp_served = read_u32_le(&rx[28..32]);
    let cmd_served = read_u32_le(&rx[32..36]);
    let lock_time = read_u32_le(&rx[36..40]);
    let flags = rx[40];
    let num_sv = rx[41];
    let ser_num = read_u16_le(&rx[42..44]);
    let fw_ver = read_u32_le(&rx[44..48]);
    let unix_secs = (ref_ts1.saturating_sub(time1970)) as i64;

    if cfg.options.show_stats {
        let (y, mo, d, h, mi, s) = unix_to_ymdhms(unix_secs);
        println!("\n===== LeoNTP Statistics ({}) =====", cfg.leontp.ipaddr);
        println!(
            "UTC time       : {:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:.0}",
            y, mo, d, h, mi, s, ref_ts0
        );
        println!("NTP time       : {:.0}", ref_ts1 as f64 + ref_ts0);
        println!("Uptime         : {} s ({:.2} days)", uptime, uptime as f64 / 86400.0);
        println!("NTP requests   : {}", ntp_served);
        println!("Mode 6 requests: {}", cmd_served);
        println!("GPS lock time  : {} s ({:.2} days)", lock_time, lock_time as f64 / 86400.0);
        println!("GPS flags      : {}", flags);
        println!("Active satellites: {}", num_sv);
        println!("Firmware ver.  : {}.{:02}", fw_ver >> 8, fw_ver & 0xFF);
        println!("Serial number  : {}", ser_num);
        println!("==========================================\n");
    }

    // Envoi vers InfluxDB si activé
    if cfg.options.send_to_influxdb {
        let ts_ns = (unix_secs as i128)
            .saturating_mul(1_000_000_000)
            .saturating_add((ref_ts0 * 1_000_000_000.0) as i128);

        let line = format!(
            "Measurements,host=NTP01 Uptime={},Nb_NTP_Requests={},Nb_of_SAT={},GPS_lock_time={},Serial_Number={},Firmware_Version={} {}",
            uptime, ntp_served, num_sv, lock_time, ser_num, fw_ver, ts_ns
        );

        let path = format!(
            "/api/v2/write?org={}&bucket={}&precision=ns",
            url_encode(&cfg.influxdb.org),
            url_encode(&cfg.influxdb.bucket)
        );

        let mut stream = match TcpStream::connect((&cfg.influxdb.host[..], cfg.influxdb.port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Error: failed to connect to InfluxDB {}:{}: {}", cfg.influxdb.host, cfg.influxdb.port, e);
                return;
            }
        };
        stream.set_write_timeout(Some(Duration::from_secs(3))).ok();
        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

        let body = line.as_bytes();
        let req = format!(
            "POST {} HTTP/1.1\r\n\
             Host: {}:{}\r\n\
             Authorization: Token {}\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n",
            path,
            cfg.influxdb.host,
            cfg.influxdb.port,
            cfg.influxdb.token,
            body.len()
        );

        if let Err(e) = stream.write_all(req.as_bytes()).and_then(|_| stream.write_all(body)) {
            eprintln!("❌ HTTP send error: {}", e);
            return;
        }

        let mut resp = String::new();
        if let Err(e) = stream.read_to_string(&mut resp) {
            eprintln!("❌ HTTP response read error: {}", e);
            return;
        }

        if let Some(first_line) = resp.lines().next() {
            if first_line.contains(" 204 ") || first_line.contains(" 200 ") {
                if cfg.options.show_stats {
                    println!("✅ Data successfully sent to InfluxDB.");
                }
            } else {
                eprintln!("❌ InfluxDB error: {}", first_line);
                if let Some(idx) = resp.find("\r\n\r\n") {
                    eprintln!("Detail: {}", &resp[idx + 4..]);
                }
            }
        } else {
            eprintln!("❌ Empty/illegal HTTP response.");
        }
    } else if cfg.options.show_stats {
        println!("ℹ️  Envoi vers InfluxDB désactivé par configuration.");
    }
}
