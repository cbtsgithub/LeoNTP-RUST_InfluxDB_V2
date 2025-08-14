use ini::Ini;
use std::error::Error;
use std::path::PathBuf;
use std::env;

pub struct LeoNTP {
    pub ipaddr: String,
    pub portnum: u16,
}

pub struct InfluxDB {
    pub host: String,
    pub port: u16,
    pub token: String,
    pub bucket: String,
    pub org: String,
}

pub struct Options {
    pub show_stats: bool,
    pub send_to_influxdb: bool,
}

pub struct Config {
    pub leontp: LeoNTP,
    pub influxdb: InfluxDB,
    pub options: Options,
}

impl Config {
    /// Charge le fichier INI depuis un chemin donné
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let ini = Ini::load_from_file(path)?;

        // --- LeoNTP section ---
        let leontp_section = ini.section(Some("LEONTP"))
            .ok_or("Section [LEONTP] missing")?;
        let ipaddr = leontp_section.get("IPADDR")
            .ok_or("IPADDR missing")?
            .to_string();
        let portnum = leontp_section.get("PORTNUM")
            .ok_or("PORTNUM missing")?
            .parse::<u16>()?;

        // --- InfluxDB section ---
        let influx_section = ini.section(Some("INFLUXDB"))
            .ok_or("Section [INFLUXDB] missing")?;
        let host = influx_section.get("HOST")
            .ok_or("HOST missing")?
            .to_string();
        let port = influx_section.get("PORT")
            .ok_or("PORT missing")?
            .parse::<u16>()?;
        let token = influx_section.get("TOKEN")
            .ok_or("TOKEN missing")?
            .to_string();
        let bucket = influx_section.get("BUCKET")
            .ok_or("BUCKET missing")?
            .to_string();
        let org = influx_section.get("ORG")
            .ok_or("ORG missing")?
            .to_string();

        // --- Options section ---
        let options_section = ini.section(Some("OPTIONS"))
            .ok_or("Section [OPTIONS] missing")?;
        let show_stats = options_section
            .get("SHOW_STATS")
            .unwrap_or("false")
            .eq_ignore_ascii_case("true");

        let send_to_influxdb = options_section
            .get("SEND_TO_INFLUXDB")
            .unwrap_or("false")
            .eq_ignore_ascii_case("true");

        Ok(Config {
            leontp: LeoNTP { ipaddr, portnum },
            influxdb: InfluxDB { host, port, token, bucket, org },
            options: Options { show_stats, send_to_influxdb },
        })
    }

    /// Détermine le chemin du fichier INI depuis un argument ou le répertoire de l'exécutable
    pub fn resolve_path_from_args_or_exe(default_filename: &str) -> Result<PathBuf, Box<dyn Error>> {
        let mut args = env::args();
        args.next(); // ignore le nom du programme

        if let Some(custom_path) = args.next() {
            return Ok(PathBuf::from(custom_path));
        }

        let exe_path = env::current_exe()?;
        let exe_dir = exe_path.parent()
            .ok_or("Impossible de déterminer le répertoire de l'exécutable")?;
        Ok(exe_dir.join(default_filename))
    }
}
