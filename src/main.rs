// The main todos thingie:
// Audio playback
// This is a TUI ok im gonna use ratatui since my focus is the audio here not re-inventing the TUI
// Made for asym games, so sections of each song that you can easily loop (like bookmarks kinda)
// Also thing for DoD, harken themes have calm, then transition, then enraged, would be nice to have a button to transition to enraged (and back)

// Resources?
// https://www.nerdfonts.com/cheat-sheet - Icon cheat sheet, useful for... well, finding icons to use for stuff

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use rand::Rng;
use ratatui::{layout::{Alignment, Constraint, Layout, Margin, Rect}, style::{Color, Style, Stylize}, text::{Line, Span, Text}, widgets::{block::Title, Block, Borders, Paragraph, Sparkline, Wrap}, Frame};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut state = AppState::new();
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|frame| draw_frame(frame, &mut state)).expect("Failed to draw frame"); // TODO: error handling :3
        handle_events(&mut state)?; // TODO: key events arent the only thing that should trigger a redraw, later though i havent made audio stuff yet

        if state.exit { break; }
    }

    ratatui::restore();

    Ok(())
}

fn draw_frame(frame: &mut Frame, state: &mut AppState) {
    use Constraint::{Length, Min, Ratio};

    let mut rng = rand::rng();

    let top_layout = Layout::vertical([Length(1), Min(3), Length(1)]); // idk im just testing i need to figure out a layout first i guess
    let [title_area, main_area, status_area] = top_layout.areas(frame.area());
    let main_layout = Layout::horizontal([Min(3), Ratio(2, 3)]);
    let [left_area, right_area] = main_layout.areas(main_area);

    let title_block = Block::new()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title("Title")
        .title_alignment(Alignment::Center);

    let status_block = Block::new()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .title("Press q to quit (i love vim keybinds)")
        .title_alignment(Alignment::Center);

    let songs_list_block = Block::bordered()
        .title("Songs List");

    let temp_songs_list_message = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("This will be a list of songs, in a "),
            Span::styled("tree", Style::new().italic()),
            Span::raw(" structure.")
        ]),
        Line::from(vec![
            Span::raw("It will have "),
            Span::styled("collapsible arrows", Style::new().italic()),
            Span::styled("    ", Style::new().gray().dim()),
            Span::raw(" beside each folder.")
        ]),
    ]).wrap(Wrap { trim: false });

    // let song_info_block = Block::bordered()
    //     .title("Song Info");

    let song_info_layout = Layout::vertical(vec![Min(2), Length(1), Length(3)]);
    let [waveform_area, _, song_info_area] = song_info_layout.areas(right_area.inner(Margin::new(1, 1)));
    
    // yup custom block widget so i can have nice connecting horizontal line separator :3
    // dist_from_bottom should always be equal to the height of the song_info_area just update it whenever you update that ok?
    let song_info_block = connected_block_widget("Song Info", right_area, 3);

    // Sparklines will panic in debug if value is too high because of the line `*value * u64::from(spark_area.height) * 8 / max_height`
    // value * height * 8 = u64::MAX // max_height removed because overflow happens before it can even divide
    // value = u64::MAX / (height * 8) // maximum our value can be to not overflow
    // maximum for a height of 75 is 30,744,573,456,182,586 (30 quadrillion) i dont think we need to worry about it when thinking normally
    // to cause an overflow you would need height to be above 1,518,500,249 (i dont think you need or can do a 1.5 billion character tall sparkline)
    // let max_sparkline_height = (u64::MAX / 8) / u64::from(waveform_area.height);

    let max_sparkline_height = u64::from(waveform_area.height) * 8;
    let temp_waveform_data: Vec<u64> = (0..waveform_area.width).map(|_| rng.random_range(0..=max_sparkline_height)).collect();

    let waveform_graph = Sparkline::default()
        .max(max_sparkline_height)
        .data(temp_waveform_data);

    // let mut line = String::new();
    // line.push('├');
    // line.push_str(&"─".repeat(song_info_line_area.width.saturating_sub(2) as usize));
    // line.push('┤');
    //
    // let song_info_line = Text::styled(line, Style::default().fg(Color::Gray));

    let temp_song_info_message = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("This will store song info, at the top is the "),
            Span::styled("audio visualizer", Style::new().gray()),
            Span::raw(", then the name, the bar™, and whatever else, like bookmarked looping points?"),
        ]),
        Line::raw(format!("Sparkline height: {}, max sparkline height: {max_sparkline_height}", waveform_area.height)),
    ]).wrap(Wrap { trim: false });

    frame.render_widget(title_block, title_area);
    frame.render_widget(status_block, status_area);

    let songs_list_area = songs_list_block.inner(left_area);
    frame.render_widget(songs_list_block, left_area);
    frame.render_widget(temp_songs_list_message, songs_list_area);

    frame.render_widget(song_info_block, right_area);
    frame.render_widget(waveform_graph, waveform_area);
    // frame.render_widget(song_info_line, song_info_line_area);
    frame.render_widget(temp_song_info_message, song_info_area);
}

// i aint makin a custom widget when i can cheap out with Text
fn connected_block_widget<'a>(title: impl Into<Line<'a>>, area: Rect, dist_from_bottom: usize) -> Text<'a> {
    let title: Line = title.into();
    let width = area.width as usize;
    let height = area.height as usize;

    let mut lines = Vec::new();

    lines.push(Line::raw(format!("┌{}{}┐", title, "─".repeat(width.saturating_sub(2 + title.width())))));

    for _ in 0..height.saturating_sub(dist_from_bottom + 3) {
        lines.push(Line::raw(format!("│{}│", " ".repeat(width.saturating_sub(2)))));
    }

    lines.push(Line::raw(format!("├{}┤", "─".repeat(width.saturating_sub(2)))));

    for _ in 0..dist_from_bottom {
        lines.push(Line::raw(format!("│{}│", " ".repeat(width.saturating_sub(2)))));
    }

    lines.push(Line::raw(format!("└{}┘", "─".repeat(width.saturating_sub(2)))));

    Text::from(lines).style(Style::default().fg(Color::Gray))
}

/// blocks until we get events, because then we need to draw, i think?
fn handle_events(state: &mut AppState) -> Result<()> {
    match event::read()? {
        Event::FocusGained => {},
        Event::FocusLost => {},
        Event::Key(key) => {
            match key.code {
                KeyCode::Char(c) => match c {
                    'q' => state.exit = true, // TODO: when menus get added q wont just close the program
                    'w' => {}, // just so lsp dont complain about 1 pattern
                    _ => {},
                },
                KeyCode::Enter => {}, // same here, anti-lsp complaint :3
                _ => {},
            }
        },
        Event::Mouse(_mouse) => {},
        Event::Paste(_text) => {},
        Event::Resize(_width, _height) => {},
    }

    Ok(())
}

// TODO: move this somewhere idk no way this is staying in main.rs
#[derive(Clone, Debug, Default)]
struct AppState {
    exit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
