use anyhow::Result;

use super::args::TelemetryCommand;

pub(crate) fn run(action: TelemetryCommand) -> Result<()> {
    match action {
        TelemetryCommand::Status { json } => run_status(json),
        TelemetryCommand::Enable => run_enable(),
        TelemetryCommand::Disable => run_disable(),
    }
}

fn run_status(json: bool) -> Result<()> {
    let status = crate::telemetry::status();
    let source = status.opt_out_source.map(|source| source.as_str());

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "enabled": status.enabled,
                "content_sharing_enabled": status.content_sharing_enabled,
                "opt_out_source": source,
                "telemetry_id": status.telemetry_id,
            }))?
        );
        return Ok(());
    }

    println!(
        "Anonymous usage telemetry: {}",
        if status.enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!(
        "Content sharing: {}",
        if status.content_sharing_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    if let Some(source) = source {
        println!("Opt-out source: {source}");
    }
    println!(
        "Anonymous telemetry ID: {}",
        status.telemetry_id.as_deref().unwrap_or("not assigned")
    );
    Ok(())
}

fn run_enable() -> Result<()> {
    crate::telemetry::enforce_disabled_policy();
    println!("Telemetry is permanently disabled in this privacy fork.");
    Ok(())
}

fn run_disable() -> Result<()> {
    crate::telemetry::enforce_disabled_policy();
    println!("Telemetry is permanently disabled in this privacy fork.");
    Ok(())
}
