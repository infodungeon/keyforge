// apps/keyforge-tui/src/main.rs

use clap::Parser;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use keyforge_infra::net::client::ClientConfig;
use keyforge_infra::HiveClient;
use keyforge_model::constants::DEFAULT_HIVE_URL;
use keyforge_protocol::SystemMetrics;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use serde::Deserialize;
use std::io;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const TUI_DOCKER_REFRESH_INTERVAL_SECS: u64 = 5;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = DEFAULT_HIVE_URL)]
    url: String,

    #[arg(long, env = "HIVE_SECRET")]
    secret: Option<String>,
}

/// A single log entry from the Hive server.
#[derive(Deserialize, Default, Clone, Debug)]
pub struct LogEntry {
    /// RFC3339 formatted timestamp.
    pub timestamp: String,
    /// Log level (e.g., "INFO", "WARN", "ERROR").
    pub level: String,
    /// The actual log message content.
    pub message: String,
}

/// The combined response from the system status endpoint.
#[derive(Deserialize, Default, Clone, Debug)]
pub struct SystemStatusResponse {
    /// Aggregate system and cluster metrics.
    pub metrics: SystemMetrics,
    /// A collection of recent important log entries.
    pub logs: Vec<LogEntry>,
}

#[derive(Clone, Default, Debug)]
struct ContainerMetrics {
    name: String,
    status: String,
    ram: String,
    cpu: String,
    is_online: bool,
}

#[derive(Debug)]
struct DockerMonitor {
    state: Arc<Mutex<Vec<ContainerMetrics>>>,
}

impl DockerMonitor {
    fn new() -> Self {
        let state = Arc::new(Mutex::new(Vec::new()));
        let state_clone = state.clone();

        thread::spawn(move || loop {
            let metrics = Self::fetch();
            if let Ok(mut guard) = state_clone.lock() {
                *guard = metrics;
            }
            thread::sleep(Duration::from_secs(TUI_DOCKER_REFRESH_INTERVAL_SECS));
        });

        Self { state }
    }

    fn get(&self) -> Vec<ContainerMetrics> {
        self.state.lock().map(|g| g.clone()).unwrap_or_default()
    }

    fn fetch() -> Vec<ContainerMetrics> {
        let mut containers = Vec::new();

        // 1. Get Status (Uptime) via `docker ps`
        // Format: Names 	 Status
        let ps_output = Command::new("docker")
            .args(["ps", "--format", "{{.Names}}\t{{.Status}}"])
            .output()
            .ok();

        let mut status_map = std::collections::HashMap::new();
        if let Some(out) = ps_output {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                if let Some((name, status)) = line.split_once('\t') {
                    status_map.insert(name.to_string(), status.to_string());
                }
            }
        }

        // 2. Get Stats (RAM/CPU) via `docker stats`
        // Format: Name 	 MemUsage 	 CPUPerc
        let stats_output = Command::new("docker")
            .args([
                "stats",
                "--no-stream",
                "--format",
                "{{.Name}}\t{{.MemUsage}}\t{{.CPUPerc}}",
            ])
            .output()
            .ok();

        if let Some(out) = stats_output {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    // Filter for relevant containers
                    if name.contains("hive")
                        || name.contains("db")
                        || name.contains("postgres")
                        || name.contains("web")
                        || name.contains("apache")
                    {
                        let status = status_map
                            .get(&name)
                            .cloned()
                            .unwrap_or_else(|| "Unknown".to_string());
                        containers.push(ContainerMetrics {
                            name,
                            status,
                            ram: parts[1].to_string(),
                            cpu: parts[2].to_string(),
                            is_online: true,
                        });
                    }
                }
            }
        }

        // Sort for consistency
        containers.sort_by(|a, b| a.name.cmp(&b.name));
        containers
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    run_monitor(args.url, args.secret).await
}

/// Starts the interactive TUI monitor for the `KeyForge` Hive.
///
/// This function takes over the terminal, connects to the Hive API,
/// and displays real-time metrics, logs, and Docker container status.
async fn run_monitor(
    url: String,
    secret: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Assume assets are on port 3001 if hive is 3000, or just use same base if proxied
    let asset_url = url.replace("3000", "3001");

    let config = ClientConfig {
        api_url: url,
        asset_url,
        secret,
        ..Default::default()
    };
    let client = HiveClient::new(config).map_err(io::Error::other)?;
    let docker = DockerMonitor::new();

    let mut status = SystemStatusResponse::default();
    let mut error_msg = String::new();

    loop {
        // 1. Fetch API Data
        match client.get("sys/status").send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(s) = resp.json::<SystemStatusResponse>().await {
                        status = s;
                        error_msg.clear();
                    } else {
                        error_msg = "Failed to parse status JSON".to_string();
                    }
                } else {
                    error_msg = format!("API Error: {}", resp.status());
                }
            }
            Err(e) => error_msg = format!("Connection Failed: {e}"),
        }

        // 2. Fetch Docker Data (Non-blocking via thread)
        let containers = docker.get();

        // 3. Draw
        terminal.draw(|f| ui(f, &status, &containers, &error_msg))?;

        // 4. Input Handling
        if event::poll(Duration::from_millis(1000))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Restore Terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn ui(
    f: &mut Frame<'_>,
    status: &SystemStatusResponse,
    containers: &[ContainerMetrics],
    err: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Length(3),      // Status Bar
            Constraint::Percentage(40), // Main Stats
            Constraint::Percentage(50), // Logs
            Constraint::Length(3),      // Footer
        ])
        .split(f.area());

    // Header
    let title = Paragraph::new("KEYFORGE HIVE COMMAND CENTER")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Error Bar
    if err.is_empty() {
        let uptime = status.metrics.uptime_secs;
        let status_text = format!("Host Uptime: {uptime}s | Online");
        let status_widget = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status_widget, chunks[1]);
    } else {
        let err_widget = Paragraph::new(err)
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(err_widget, chunks[1]);
    }

    // Main Grid
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Left: Cluster Stats
    let m = &status.metrics;
    let cluster_rows = vec![
        Row::new(vec![
            Cell::from("Active Jobs"),
            Cell::from(m.active_jobs.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Total Results"),
            Cell::from(m.total_results.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Nodes Online"),
            Cell::from(m.nodes_online.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Cluster Ops/Sec"),
            Cell::from(format!("{:.2} M", m.total_ops_per_sec / 1_000_000.0)),
        ]),
    ];

    let cluster_table = Table::new(
        cluster_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title("Cluster State")
            .borders(Borders::ALL),
    )
    .header(Row::new(vec!["Metric", "Value"]).style(Style::default().fg(Color::Yellow)));

    f.render_widget(cluster_table, main_chunks[0]);

    // Right: Container Stats (Replaces Server RAM Gauge)
    let container_rows: Vec<Row<'_>> = containers
        .iter()
        .map(|c| {
            let style = if c.is_online {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(c.status.clone()),
                Cell::from(c.ram.clone()),
                Cell::from(c.cpu.clone()),
            ])
            .style(style)
        })
        .collect();

    let container_table = Table::new(
        container_rows,
        [
            Constraint::Percentage(25), // Name
            Constraint::Percentage(45), // Status
            Constraint::Percentage(15), // RAM
            Constraint::Percentage(15), // CPU
        ],
    )
    .block(
        Block::default()
            .title("Docker Containers")
            .borders(Borders::ALL),
    )
    .header(
        Row::new(vec!["Name", "Status", "RAM", "CPU"]).style(Style::default().fg(Color::Magenta)),
    );

    f.render_widget(container_table, main_chunks[1]);

    // Logs Panel
    let log_rows: Vec<Row<'_>> = status
        .logs
        .iter()
        .rev()
        .map(|l| {
            let color = match l.level.as_str() {
                "ERROR" => Color::Red,
                "WARN" => Color::Yellow,
                _ => Color::White,
            };
            Row::new(vec![
                Cell::from(l.timestamp.clone()),
                Cell::from(l.level.clone()),
                Cell::from(l.message.clone()),
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    let log_table = Table::new(
        log_rows,
        [
            Constraint::Length(25),
            Constraint::Length(8),
            Constraint::Min(10),
        ],
    )
    .block(
        Block::default()
            .title("System Events (WARN/ERROR)")
            .borders(Borders::ALL),
    )
    .header(Row::new(vec!["Time", "Level", "Message"]).style(Style::default().fg(Color::Blue)));

    f.render_widget(log_table, chunks[3]);

    // Footer
    let footer = Paragraph::new("Press 'q' to exit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[4]);
}
