use ini::Ini;
use std::error::Error;

pub struct Config {
    pub leontp: LeoNTPSection,
    pub influxdb: InfluxDBSection,
    pub options: OptionsSection,
}

pub struct LeoNTPSection {
    pub ipaddr: String,
    pub portnum: u16,
}

pub struct InfluxDBSection {
    pub host: String,
    pub port: u16,
    pub token: String,
    pub bucket: String,
    pub org: String,
}

pub struct OptionsSection {
    pub show_stats: bool,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let conf = Ini::load_from_file(path)?;

        let leontp = conf.section(Some("LEONTP"))
            .ok_or("Section [LEONTP] missing")?;
        let influxdb = conf.section(Some("INFLUXDB"))
            .ok_or("Section [INFLUXDB] missing")?;
        let options = conf.section(Some("OPTIONS"))
            .ok_or("Section [OPTIONS] missing")?;

        Ok(Config {
            leontp: LeoNTPSection {
                ipaddr: leontp.get("IPADDR").unwrap().to_string(),
                portnum: leontp.get("PORTNUM").unwrap().parse()?,
            },
            influxdb: InfluxDBSection {
                host: influxdb.get("HOST").unwrap().to_string(),
                port: influxdb.get("PORT").unwrap().parse()?,
                token: influxdb.get("TOKEN").unwrap().to_string(),
                bucket: influxdb.get("BUCKET").unwrap().to_string(),
                org: influxdb.get("ORG").unwrap().to_string(),
            },
            options: OptionsSection {
                show_stats: options.get("SHOW_STATS").unwrap().parse::<bool>()?,
            }
        })
    }
}
