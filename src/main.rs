// The main todos thingie:
// Audio playback
// This is a TUI ok im gonna use ratatui since my focus is the audio here not re-inventing the TUI
// Made for asym games, so sections of each song that you can easily loop (like bookmarks kinda)
// Also thing for DoD, harken themes have calm, then transition, then enraged, would be nice to have a button to transition to enraged (and back)

// Resources?
// https://www.nerdfonts.com/cheat-sheet - Icon cheat sheet, useful for... well, finding icons to use for stuff

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{layout::{Alignment, Constraint, Layout}, style::{Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph, Wrap}, Frame};

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

    let song_info_block = Block::bordered()
        .title("Song Info");

    let temp_song_info_message = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("This will store song info, at the top is the "),
            Span::styled("audio graph thingie", Style::new().gray()),
            Span::raw(" you know what i mean, then the name, the bar™, and whatever else, like bookmarked looping points?"),
        ]),
    ]).wrap(Wrap { trim: false });

    frame.render_widget(title_block, title_area);
    frame.render_widget(status_block, status_area);

    let songs_list_area = songs_list_block.inner(left_area);
    frame.render_widget(songs_list_block, left_area);
    frame.render_widget(temp_songs_list_message, songs_list_area);

    let song_info_area = song_info_block.inner(right_area);
    frame.render_widget(song_info_block, right_area);
    frame.render_widget(temp_song_info_message, song_info_area);
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
