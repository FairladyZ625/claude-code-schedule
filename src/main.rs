use anyhow::{Context, Result};
use chrono::{DateTime, Local, Timelike};
use clap::Parser;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

mod logger;
use logger::Logger;

#[derive(Parser, Debug)]
#[command(
    author = "Ian Macalinao <ian@macalinao.com>",
    version,
    about = "Schedule Claude Code to run at a specific time - by Ian Macalinao",
    long_about = "A CLI tool by Ian Macalinao that runs Claude Code at a scheduled time. \
                  The tool will stay running in your terminal and execute the command when the time is reached.\
                  \n\nCreated by Ian Macalinao - https://ianm.com"
)]
struct Args {
    /// Run Claude Code at a specific time (format: HH:MM, default: 06:00)
    #[arg(short, long, value_name = "HH:MM")]
    time: Option<String>,

    /// Message to pass to Claude Code (default: "Continue working on what you were working on previously. If you weren't working on something previously, then come up with a list of tasks to work on based on what is left in the codebase.")
    #[arg(
        short,
        long,
        default_value = "Continue working on what you were working on previously. If you weren't working on something previously, then come up with a list of tasks to work on based on what is left in the codebase."
    )]
    message: String,

    /// Dry run - print what would happen without scheduling
    #[arg(short, long)]
    dry_run: bool,

    /// Query global weather information instead of running Claude Code
    #[arg(short, long)]
    ping_mode: bool,

    /// Directory for storing logs (default: log)
    #[arg(long, default_value = "log")]
    log_dir: String,

    /// Enable continuous loop mode (runs every 5 hours: 7:00, 12:00, 17:00, 22:00, 03:00)
    #[arg(short, long)]
    loop_mode: bool,

    /// Write PID file for daemon management
    #[arg(long)]
    pid_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger
    let logger = Logger::new(&args.log_dir);
    logger.init().context("Failed to initialize logger")?;

    // Write PID file if requested
    if let Some(ref pid_file) = args.pid_file {
        write_pid_file(pid_file)?;
    }

    if args.loop_mode {
        // Loop mode: ignore time parameter and use predefined schedule
        run_loop_mode(&args, &logger).await?;
    } else {
        // Single execution mode
        let target_time = if let Some(ref time_str) = args.time {
            parse_time(time_str)?
        } else {
            // Default to 6:00 AM
            parse_time("06:00")?
        };

        let target_time = if target_time <= Local::now() {
            target_time + chrono::Duration::days(1)
        } else {
            target_time
        };

        run_single_mode(&args, &logger, target_time).await?;
    }

    // Cleanup PID file
    cleanup_pid_file(&args.pid_file);
    Ok(())
}

async fn run_single_mode(args: &Args, logger: &Logger, target_time: DateTime<Local>) -> Result<()> {

    if args.dry_run {
        println!("Would run at: {}", target_time.format("%Y-%m-%d %H:%M:%S"));
        if args.ping_mode {
            println!("Action: Query global weather information");
        } else {
            println!("Command: {}", build_claude_command(&args.message));
        }
        println!("Log directory: {}", args.log_dir);
        return Ok(());
    }

    println!("Claude Code Schedule by Ian Macalinao");
    println!(
        "Scheduled to run at: {}",
        target_time.format("%Y-%m-%d %H:%M:%S")
    );
    if args.ping_mode {
        println!("Action: Query global weather information");
    } else {
        println!("Command: {}", build_claude_command(&args.message));
    }
    println!("Log directory: {}", args.log_dir);
    println!("Press Ctrl+C to cancel...\n");

    // Set up Ctrl+C handler
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\nCancelled by user");
        std::process::exit(0);
    });

    // Wait until the target time
    loop {
        let now = Local::now();
        if now >= target_time {
            println!("\nRunning scheduled action...");

            if args.ping_mode {
                match run_ping(&args.message) {
                    Ok(response) => {
                        if let Err(e) = logger.log_ping_success_with_response(&response, None) {
                            eprintln!("Warning: Failed to log ping success: {e}");
                        }
                        println!("Ping completed successfully!");
                        println!("Response length: {} characters", response.len());
                    }
                    Err(e) => {
                        if let Err(log_err) = logger.log_ping_error_with_cycle(&e.to_string(), None) {
                            eprintln!("Warning: Failed to log ping error: {log_err}");
                        }
                        return Err(e);
                    }
                }
            } else {
                match run_claude_command(&args.message) {
                    Ok(response) => {
                        if let Err(e) = logger.log_claude_success_with_response(&response, None) {
                            eprintln!("Warning: Failed to log claude success: {e}");
                        }
                        println!("Command completed successfully!");
                        println!("Response length: {} characters", response.len());
                    }
                    Err(e) => {
                        if let Err(log_err) = logger.log_claude_error_with_cycle(&e.to_string(), None) {
                            eprintln!("Warning: Failed to log claude error: {log_err}");
                        }
                        return Err(e);
                    }
                }
            }

            println!("Claude Code Schedule by Ian Macalinao - https://ianm.com");
            break;
        }

        let duration_until = target_time.signed_duration_since(now);
        let hours = duration_until.num_hours();
        let minutes = duration_until.num_minutes() % 60;
        let seconds = duration_until.num_seconds() % 60;

        print!("\rTime remaining: {hours:02}:{minutes:02}:{seconds:02}");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        // Sleep for 1 second
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn run_loop_mode(args: &Args, logger: &Logger) -> Result<()> {
    if args.dry_run {
        println!("Loop mode dry run:");
        println!("Schedule: 7:00, 12:00, 17:00, 22:00, 03:00 (every 5 hours)");
        if args.ping_mode {
            println!("Action: Query global weather information");
        } else {
            println!("Command: {}", build_claude_command(&args.message));
        }
        println!("Log directory: {}", args.log_dir);
        return Ok(());
    }

    println!("Claude Code Schedule by Ian Macalinao - Loop Mode");
    println!("Schedule: 7:00, 12:00, 17:00, 22:00, 03:00 (every 5 hours)");
    if args.ping_mode {
        println!("Action: Query global weather information");
    } else {
        println!("Command: {}", build_claude_command(&args.message));
    }
    println!("Log directory: {}", args.log_dir);
    println!("Press Ctrl+C to stop...\n");

    // Set up Ctrl+C handler for loop mode
    let pid_file_clone = args.pid_file.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\nStopping loop mode...");
        cleanup_pid_file(&pid_file_clone);
        std::process::exit(0);
    });

    let mut cycle_number = 1u32;

    loop {
        let now = Local::now();
        let next_time = get_next_loop_time(now);

        println!("Cycle {cycle_number} - Next execution: {}", next_time.format("%Y-%m-%d %H:%M:%S"));

        // Wait until the next scheduled time
        loop {
            let now = Local::now();
            if now >= next_time {
                break;
            }

            let duration_until = next_time.signed_duration_since(now);
            let hours = duration_until.num_hours();
            let minutes = duration_until.num_minutes() % 60;
            let seconds = duration_until.num_seconds() % 60;

            print!("\rTime until next execution: {hours:02}:{minutes:02}:{seconds:02}");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();

            sleep(Duration::from_secs(1)).await;
        }

        // Log cycle start
        if let Err(e) = logger.log_cycle_start(cycle_number) {
            eprintln!("Warning: Failed to log cycle start: {e}");
        }

        println!("\nExecuting cycle {cycle_number}...");

        // Execute the action
        if args.ping_mode {
            match run_ping(&args.message) {
                Ok(response) => {
                    if let Err(e) = logger.log_ping_success_with_response(&response, Some(cycle_number)) {
                        eprintln!("Warning: Failed to log ping success: {e}");
                    }
                    println!("Cycle {cycle_number} ping completed successfully!");
                    println!("Response length: {} characters", response.len());
                }
                Err(e) => {
                    if let Err(log_err) = logger.log_ping_error_with_cycle(&e.to_string(), Some(cycle_number)) {
                        eprintln!("Warning: Failed to log ping error: {log_err}");
                    }
                    eprintln!("Cycle {cycle_number} ping failed: {e}");
                }
            }
        } else {
            match run_claude_command(&args.message) {
                Ok(response) => {
                    if let Err(e) = logger.log_claude_success_with_response(&response, Some(cycle_number)) {
                        eprintln!("Warning: Failed to log claude success: {e}");
                    }
                    println!("Cycle {cycle_number} command completed successfully!");
                    println!("Response length: {} characters", response.len());
                }
                Err(e) => {
                    if let Err(log_err) = logger.log_claude_error_with_cycle(&e.to_string(), Some(cycle_number)) {
                        eprintln!("Warning: Failed to log claude error: {log_err}");
                    }
                    eprintln!("Cycle {cycle_number} command failed: {e}");
                }
            }
        }

        // Log cycle end
        if let Err(e) = logger.log_cycle_end(cycle_number) {
            eprintln!("Warning: Failed to log cycle end: {e}");
        }

        cycle_number += 1;
        println!("Cycle completed. Waiting for next scheduled time...\n");
    }
}

fn parse_time(time_str: &str) -> Result<DateTime<Local>> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid time format. Expected HH:MM");
    }

    let hour: u32 = parts[0].parse().context("Invalid hour")?;
    let minute: u32 = parts[1].parse().context("Invalid minute")?;

    if hour >= 24 || minute >= 60 {
        anyhow::bail!("Invalid time. Hour must be 0-23, minute must be 0-59");
    }

    let now = Local::now();
    now.with_hour(hour)
        .and_then(|t| t.with_minute(minute))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .context("Failed to create target time")
}

fn get_loop_schedule() -> Vec<(u32, u32)> {
    // (hour, minute) pairs for the 5-hour cycle
    vec![(7, 0), (12, 0), (17, 0), (22, 0), (3, 0)]
}

fn get_next_loop_time(now: DateTime<Local>) -> DateTime<Local> {
    let schedule = get_loop_schedule();
    let _current_time = (now.hour(), now.minute());

    // Find the next scheduled time
    for &(hour, minute) in &schedule {
        let target = now
            .with_hour(hour)
            .and_then(|t| t.with_minute(minute))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap();

        if target > now {
            return target;
        }
    }

    // If no time today, get the first time tomorrow
    let tomorrow = now + chrono::Duration::days(1);
    tomorrow
        .with_hour(schedule[0].0)
        .and_then(|t| t.with_minute(schedule[0].1))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .unwrap()
}

fn write_pid_file(pid_file: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let pid = std::process::id();
    let mut file = File::create(pid_file)
        .context("Failed to create PID file")?;
    writeln!(file, "{pid}")
        .context("Failed to write PID to file")?;

    println!("PID file written: {pid_file} (PID: {pid})");
    Ok(())
}

fn cleanup_pid_file(pid_file: &Option<String>) {
    if let Some(path) = pid_file {
        if let Err(e) = std::fs::remove_file(path) {
            eprintln!("Warning: Failed to remove PID file {path}: {e}");
        } else {
            println!("PID file removed: {path}");
        }
    }
}

fn build_claude_command(message: &str) -> String {
    format!(
        "claude --dangerously-skip-permissions \"{}\"",
        message.replace("\"", "\\\"")
    )
}

fn run_claude_command(message: &str) -> Result<String> {
    let output = Command::new("claude")
        .args(["--dangerously-skip-permissions", message])
        .output()
        .context("Failed to execute claude command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Claude command failed with exit code: {:?}\nError: {}", output.status.code(), stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

fn run_ping(_message: &str) -> Result<String> {
    // In ping mode, we use a specific weather query to consume more tokens
    let weather_query = "请搜索今日全球天气信息，告诉我：1) 今天全世界最热的地方及其温度；2) 今天全世界最冷的地方及其温度；3) 这些地方的具体位置和当地时间；4) 简要分析造成这些极端温度的气象原因；5) 提供一些有趣的天气相关事实。请提供详细和准确的信息，包括数据来源。";
    run_claude_command(weather_query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_claude_command() {
        assert_eq!(
            build_claude_command("Hello, world!"),
            "claude --dangerously-skip-permissions \"Hello, world!\""
        );
        assert_eq!(
            build_claude_command("Hello \"world\""),
            "claude --dangerously-skip-permissions \"Hello \\\"world\\\"\""
        );
    }

    #[test]
    fn test_parse_time() {
        let time = parse_time("14:30").unwrap();
        assert_eq!(time.hour(), 14);
        assert_eq!(time.minute(), 30);
    }

    #[test]
    fn test_parse_invalid_time() {
        assert!(parse_time("25:00").is_err());
        assert!(parse_time("12:60").is_err());
        assert!(parse_time("12").is_err());
        assert!(parse_time("12:30:45").is_err());
    }
}
