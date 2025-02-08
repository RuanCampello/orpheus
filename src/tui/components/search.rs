use crate::tui::colours::Palette;
use crate::tui::components::{pad, time_from_ms, BlockExt, ToRow};
use crate::tui::state::State;
use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Row, Table};
use ratatui::Frame;
use rspotify::model::album::SimplifiedAlbum;
use rspotify::model::artist::FullArtist;
use rspotify::model::track::FullTrack;

impl<'a> ToRow<'a> for FullTrack {
    fn to_row(&self, _idx: usize) -> Row<'a> {
        let time = time_from_ms(&self.duration_ms);

        Row::new(vec![
            self.name.to_string(),
            self.artists
                .first()
                .map(|artist| &artist.name)
                .unwrap_or(&String::new())
                .to_string(),
            self.album.name.to_string(),
            time,
        ])
    }
}

impl<'a> ToRow<'a> for SimplifiedAlbum {
    fn to_row(&self, _idx: usize) -> Row<'a> {
        let artists = self
            .artists
            .iter()
            .take(3)
            .map(|artist| artist.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        Row::new(vec![self.name.to_string(), artists])
    }
}

impl<'a> ToRow<'a> for FullArtist {
    fn to_row(&self, _idx: usize) -> Row<'a> {
        Row::new(vec![self.name.to_string()])
    }
}

pub fn draw_search_input<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let input = Block::new()
        .title(pad("What do you wanna listen?", 2))
        .title_alignment(Alignment::Center)
        .bordered_section(state.search_state.active);
    let input = Paragraph::new(state.search_state.input.as_str()).block(input);

    if state.search_state.active {
        frame.set_cursor_position(Position::new(
            area.x + state.search_state.cursor_position.saturating_add(1) as u16,
            area.y + 1,
        ));
    }
    frame.render_widget(input, area);
}

pub fn draw_search_results(frame: &mut Frame, state: &mut State, area: Rect) {
    let [songs_area, lower_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [albums_area, artists_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(lower_area);

    if let Some(songs) = &mut state.search_state.results.songs.data {
        const WIDTHS: &[Constraint] = &[
            Constraint::Percentage(40),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Length(5),
        ];
        const HEADERS: &[&str; 4] = &["Title", "Artist", "Album", "Time"];

        let songs_table = draw_results_table(
            &songs.items,
            "Songs",
            WIDTHS,
            state.search_state.results.songs.table_state.active,
            HEADERS,
            None,
        );

        frame.render_stateful_widget(
            songs_table,
            songs_area,
            &mut state.search_state.results.songs.table_state.state,
        );
    }

    if let Some(albums) = &mut state.search_state.results.albums.data {
        const WIDTHS: &[Constraint] = &[Constraint::Length(40), Constraint::Length(25)];
        const HEADERS: &[&str; 2] = &["Title", "Artist"];
    
        let albums_table = draw_results_table(
            &albums.items,
            "Albums",
            WIDTHS,
            state.search_state.results.albums.table_state.active,
            HEADERS,
            None,
        );
    
        frame.render_stateful_widget(
            albums_table,
            albums_area,
            &mut state.search_state.results.albums.table_state.state,
        );
    }

    if let Some(artists) = &mut state.search_state.results.artists.data {
        const HEADERS: &[&str; 1] = &["Name"];
        const WIDTHS: &[Constraint] = &[Constraint::Length(50)];

        let artists_table = draw_results_table(
            &artists.items,
            "Artists",
            WIDTHS,
            state.search_state.results.artists.table_state.active,
            HEADERS,
            None,
        );

        frame.render_stateful_widget(
            artists_table,
            artists_area,
            &mut state.search_state.results.artists.table_state.state,
        );
    }
}

pub(super) fn draw_results_table<'a, T: ToRow<'a> + 'a>(
    items: &[T],
    title: &'a str,
    widths: &[Constraint],
    active: bool,
    headers: &'a [&'a str],
    offset: Option<u32>,
) -> Table<'a> {
    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| item.to_row(idx + 1 + offset.unwrap_or(0) as usize))
        .collect();

    Table::new(rows, widths)
        .row_highlight_style(match active {
            true => Style::new().bg(Palette::Secondary.into()).bold(),
            false => Style::new().bg(Palette::Foreground.into()).bold(),
        })
        .header(Row::new(headers.to_vec()))
        .column_spacing(2)
        .block(
            Block::bordered()
                .bordered_section(active)
                .border_type(BorderType::Rounded)
                .title(pad(title, 1))
                .padding(Padding::proportional(1)),
        )
}
