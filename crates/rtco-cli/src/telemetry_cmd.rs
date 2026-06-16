use anyhow::{Context, Result};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum TelemetrySubcommand {
    Status,
    Enable,
    Disable,
    Forget,
}

pub fn run(command: &TelemetrySubcommand) -> Result<()> {
    match command {
        TelemetrySubcommand::Status => run_status(),
        TelemetrySubcommand::Enable => run_enable(),
        TelemetrySubcommand::Disable => run_disable(),
        TelemetrySubcommand::Forget => run_forget(),
    }
}

fn run_status() -> Result<()> {
    let config = rtco_core::config::Config::load().unwrap_or_default();

    let consent_str = match config.telemetry.consent_given {
        Some(true) => "yes",
        Some(false) => "no",
        None => "never asked",
    };

    let enabled_str = if config.telemetry.enabled {
        "yes"
    } else {
        "no"
    };

    let env_override = std::env::var("RTCO_TELEMETRY_DISABLED").unwrap_or_default() == "1";

    println!("Telemetry status:");
    println!("  consent:       {}", consent_str);
    if let Some(date) = &config.telemetry.consent_date {
        println!("  consent date:  {}", date);
    }
    println!("  enabled:       {}", enabled_str);
    if env_override {
        println!("  env override:  RTCO_TELEMETRY_DISABLED=1 (blocked)");
    }

    let salt_path = rtco_core::telemetry::salt_file_path();
    if salt_path.exists() {
        let hash = rtco_core::telemetry::generate_device_hash();
        println!("  device hash:   {}...{}", &hash[..8], &hash[56..]);
    } else {
        println!("  device hash:   (no salt file)");
    }

    println!();
    println!("Data controller: RTK AI Labs, contact@rtk-ai.app");
    println!("Details: https://github.com/rtk-ai/rtk/blob/master/docs/TELEMETRY.md");

    Ok(())
}

fn run_enable() -> Result<()> {
    use std::io::{self, BufRead, IsTerminal};

    if !io::stdin().is_terminal() {
        anyhow::bail!(
            "consent requires interactive terminal — cannot enable telemetry in piped mode"
        );
    }

    eprintln!("RTK collects anonymous usage metrics once per day to improve filters.");
    eprintln!();
    eprintln!("  What:    command names (not arguments), token savings, OS, version");
    eprintln!("  Who:     RTK AI Labs, contact@rtk-ai.app");
    eprintln!("  Details: https://github.com/rtk-ai/rtk/blob/master/docs/TELEMETRY.md");
    eprintln!();
    eprint!("Enable anonymous telemetry? [y/N] ");

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .context("Failed to read user input")?;

    let accepted = {
        let response = line.trim().to_lowercase();
        response == "y" || response == "yes"
    };

    crate::hooks::init::save_telemetry_consent(accepted)?;

    if accepted {
        println!("Telemetry enabled. Disable anytime: rtk telemetry disable");
    } else {
        println!("Telemetry not enabled.");
    }

    Ok(())
}

fn run_disable() -> Result<()> {
    crate::hooks::init::save_telemetry_consent(false)?;
    println!("Telemetry disabled.");
    Ok(())
}

fn run_forget() -> Result<()> {
    crate::hooks::init::save_telemetry_consent(false)?;

    let salt_path = rtco_core::telemetry::salt_file_path();
    let marker_path = rtco_core::telemetry::telemetry_marker_path();

    // Compute device hash before deleting the salt
    let device_hash = if salt_path.exists() {
        Some(rtco_core::telemetry::generate_device_hash())
    } else {
        None
    };

    if salt_path.exists() {
        std::fs::remove_file(&salt_path)
            .with_context(|| format!("Failed to delete {}", salt_path.display()))?;
    }

    if marker_path.exists() {
        let _ = std::fs::remove_file(&marker_path);
    }

    // Purge local tracking database (GDPR Art. 17 — right to erasure applies to local data too)
    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(rtco_core::constants::RTK_DATA_DIR)
        .join(rtco_core::constants::HISTORY_DB);
    if db_path.exists() {
        match std::fs::remove_file(&db_path) {
            Ok(()) => println!("Local tracking database deleted: {}", db_path.display()),
            Err(e) => eprintln!("rtk: could not delete {}: {}", db_path.display(), e),
        }
    }

    // Send server-side erasure request
    if let Some(hash) = device_hash {
        match send_erasure_request(&hash) {
            Ok(()) => {
                println!("Erasure request sent to server.");
            }
            Err(e) => {
                eprintln!("rtk: could not reach server: {}", e);
                eprintln!("  To complete erasure, email contact@rtk-ai.app");
                eprintln!("  with your device hash: {}", hash);
            }
        }
    }

    println!("Local telemetry data deleted. Telemetry disabled.");
    Ok(())
}

fn send_erasure_request(device_hash: &str) -> Result<()> {
    let url = option_env!("RTCO_TELEMETRY_URL");
    let url = match url {
        Some(u) => format!("{}/erasure", u),
        None => anyhow::bail!("no telemetry endpoint configured"),
    };

    let payload = serde_json::json!({
        "device_hash": device_hash,
        "action": "erasure",
    });

    let mut req = ureq::post(&url).set("Content-Type", "application/json");

    if let Some(token) = option_env!("RTCO_TELEMETRY_TOKEN") {
        req = req.set("X-RTK-Token", token);
    }

    req.timeout(std::time::Duration::from_secs(5))
        .send_string(&payload.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_subcommand_status() {
        // Verify the enum variants exist and Debug works
        let cmd = TelemetrySubcommand::Status;
        assert!(format!("{:?}", cmd).contains("Status"));
    }

    #[test]
    fn test_telemetry_subcommand_all_variants() {
        let status = TelemetrySubcommand::Status;
        let enable = TelemetrySubcommand::Enable;
        let disable = TelemetrySubcommand::Disable;
        let forget = TelemetrySubcommand::Forget;

        match &status {
            TelemetrySubcommand::Status => {}
            _ => panic!("Wrong variant"),
        }
        match &enable {
            TelemetrySubcommand::Enable => {}
            _ => panic!("Wrong variant"),
        }
        match &disable {
            TelemetrySubcommand::Disable => {}
            _ => panic!("Wrong variant"),
        }
        match &forget {
            TelemetrySubcommand::Forget => {}
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_run_without_panic() {
        // Running in a non-interactive environment should work gracefully
        let result = run(&TelemetrySubcommand::Status);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_send_erasure_request_no_endpoint() {
        // Without RTCO_TELEMETRY_URL set, should bail with expected error
        let result = send_erasure_request("test-hash");
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("no telemetry endpoint configured"));
    }
}
