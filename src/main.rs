use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};
use std::{
    error::Error,
    io,
    process::Command,
    time::Duration,
};

struct App {
    volume: u8,
    is_muted: bool,
}

impl App {
    fn get_volume_system() -> u8 {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("wpctl")
                .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
                .output()
                .unwrap();

            let stdout = String::from_utf8_lossy(&output.stdout);
            let str_vol = stdout.split_whitespace().nth(1).unwrap();
            let v: f32 = str_vol.parse().unwrap();
            let volume: u8 = (v * 100.0) as u8;

            return volume;
        }
    }
    
    fn new() -> Self {
        Self {
            volume: Self::get_volume_system(),
            is_muted: false,
        }
    }

    fn increment_volume(&mut self) {
        if self.volume <= 95 {
            self.volume += 5;
        } else {
            self.volume = 100;
        }
        self.apply_system_volume();
    }

    fn decrement_volume(&mut self) {
        if self.volume >= 5 {
            self.volume -= 5;
        } else {
            self.volume = 0;
        }
        self.apply_system_volume();
    }

    fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    fn apply_system_volume(&self) {
        #[cfg(target_os = "linux")]
        {
            let vol_f32 = self.volume as f32 / 100.0;
            let _ = Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{:.2}", vol_f32)])
                .spawn();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Execution error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        app.increment_volume();
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        app.decrement_volume();
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        app.toggle_mute();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(1),
        ])
        .split(f.area());

    let title = Paragraph::new(" Volume Control ")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(title, chunks[0]);

    let gauge_label = if app.is_muted {
        "MUTED".to_string()
    } else {
        format!("{}%", app.volume)
    };

    let gauge_color = if app.is_muted {
        Color::Red
    } else if app.volume > 80 {
        Color::LightRed
    } else {
        Color::Green
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Volume "))
        .gauge_style(Style::default().fg(gauge_color).bg(Color::Black))
        .percent(if app.is_muted { 0 } else { app.volume as u16 })
        .label(gauge_label);
    f.render_widget(gauge, chunks[1]);

    let help_text = Paragraph::new(
        " [W] ou [↑] Up | [S] ou [↓] Down | [M] Mute | [Q] Exit "
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(help_text, chunks[2]);
}