use anyhow::Result;
use rustyvim::tui::TUI;

fn main() -> Result<()> {
    let mut tui = TUI::new()?;
    tui.event_loop()?;
    Ok(())
}
