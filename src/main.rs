use bytesize::ByteSize;
use clap::Parser;
use clashctl_core::ClashBuilder;
use serde_json::json;
use std::fs::read_to_string;
use std::time::Instant;
use std::{thread::sleep, time::Duration};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Clash API port
    #[arg(short, long)]
    #[arg(default_value = "9090")]
    port: usize,
    /// Update interval in seconds
    #[arg(short, long)]
    #[arg(default_value = "1")]
    interval: u64,
}

fn format_byte(value: u64) -> String {
    ByteSize(value)
        .to_string_as(false)
        .to_lowercase()
        .replace(" ", "")
}

fn print(text: String, tooltip: String) {
    let data = json!({
    "alt": "",
    "class": "",
    "percentage": 0,
    "text": text,
    "tooltip": tooltip,
    });

    println!("{data}");
}

fn main() {
    let args = Cli::parse();
    let nekoray_config_dir = dirs::config_dir().unwrap().join("nekoray/config/");
    let url = format!("http://127.0.0.1:{}", args.port);
    let clash = ClashBuilder::new(url).unwrap().build();
    let mut max_up = 0;
    let mut max_down = 0;
    let mut last = Instant::now();

    loop {
        if let Ok(traffics) = clash.get_traffic() {
            let nekobox_config: serde_json::Value = serde_json::from_str(
                read_to_string(nekoray_config_dir.join("groups/nekobox.json"))
                    .expect("Failed to read nekobox.json")
                    .as_str(),
            )
            .unwrap();

            let remember_id = nekobox_config
                .get("remember_id")
                .expect("remember_id is missing from nekobox.json")
                .as_u64()
                .unwrap();

            let nekoray_profile: serde_json::Value = serde_json::from_str(
                read_to_string(nekoray_config_dir.join(format!("profiles/{remember_id}.json")))
                    .expect("Failed to read the profile")
                    .as_str(),
            )
            .unwrap();

            let profile_name = nekoray_profile
                .get("bean")
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap();

            for traffic in traffics {
                match traffic {
                    Ok(traffic) => {
                        let now: Instant = Instant::now();

                        if traffic.up > max_up {
                            max_up = traffic.up
                        }

                        if traffic.down > max_down {
                            max_down = traffic.down
                        }

                        if now.duration_since(last).as_secs() < args.interval {
                            continue;
                        } else {
                            last = now
                        }

                        let current_upload = format_byte(traffic.up) + "/s";
                        let current_download = format_byte(traffic.down) + "/s";
                        let max_upload = format_byte(max_up) + "/s";
                        let max_download = format_byte(max_down) + "/s";
                        let mut total_upload = String::new();
                        let mut total_download = String::new();
                        let mut connection_count = String::new();

                        if let Ok(conn) = clash.get_connections() {
                            connection_count = conn.connections.len().to_string();
                            total_upload = format_byte(conn.upload_total);
                            total_download = format_byte(conn.download_total);
                        }

                        let speed_text = format!("<span foreground='#89b4fa'> {current_download}</span> <span foreground='#f38ba8'> {current_upload}</span>");
                        let max_speed_text = format!("Max: <span foreground='#89b4fa'> {max_download}</span> <span foreground='#f38ba8'> {max_upload}</span>");
                        let total_text = format!("Total: <span foreground='#89b4fa'> {total_download}</span> <span foreground='#f38ba8'> {total_upload}</span>");

                        print(
                            format!("{profile_name}"),
                            format!(
                                " {connection_count} {speed_text}\n{max_speed_text}\n{total_text}"
                            ),
                        )
                    }

                    Err(_) => break,
                }
            }

            print(String::new(), String::new())
        }

        sleep(Duration::from_secs(5))
    }
}
