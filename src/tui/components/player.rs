use crate::internal::text::{Size, Text};
use crate::tui::colours::Palette;
use crate::tui::components::{pad, time_from_ms, BlockExt};
use crate::tui::state::player::Image;
use crate::tui::state::{LyricState, State};
use deunicode::deunicode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, BorderType, Borders, LineGauge, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
    Wrap,
};
use ratatui::Frame;
use std::ops::Div;

pub fn draw_player<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    if let Some(playing) = &state.player.playing {
        let [image_area, remaining_area] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(area);

        let [title_area, remaining_area] =
            Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(remaining_area);

        if let Some(image) = &state.player.image {
            let image = Paragraph::new(image.ascii.to_string())
                .block(Block::new().secondary_border())
                .style(Style::new().fg(match playing.is_playing {
                    true => Color::White,
                    false => Palette::Foreground.into(),
                }));
            frame.render_widget(image, image_area);
        }

        let song = playing.item.as_ref().unwrap();
        let song_name: &str = match song.name.is_ascii() {
            true => song.name.as_str(),
            false => &deunicode(&song.name),
        };

        let padding = ((title_area.width / 4) as usize).saturating_sub(song_name.len()) as f32;
        let padding = padding.div(1.5).round().min(5.0) as usize;

        let title = &[&Line::from(pad(song_name, padding))];
        let title = Text::new().size(&Size::Quarter).lines(title);
        frame.render_widget(title, title_area);

        let artist = state.player.get_artist_name().unwrap_or("Unknown");

        let [info_area, progress_bar] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(4)]).areas(remaining_area);

        let info_text = ratatui::text::Text::from(format!("\n「 {artist} 」\n")).centered();
        frame.render_widget(info_text, info_area);

        draw_progress_line(
            frame,
            playing.progress_ms.as_ref().unwrap(),
            &playing.item.as_ref().unwrap().duration_ms,
            playing.is_playing,
            progress_bar,
        );
    }
}

fn draw_progress_line<'a>(
    frame: &'a mut Frame,
    progress: &'a u32,
    duration: &'a u32,
    is_playing: bool,
    area: Rect,
) {
    let time = time_from_ms(progress);
    let progress = *progress as f64;
    let ratio = progress.div(*duration as f64);

    let duration = time_from_ms(duration);

    let [gauge_area, duration_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(6)]).areas(area);

    let gauge = LineGauge::default()
        .filled_style(Style::new().fg(match is_playing {
            true => Palette::Secondary.into(),
            false => Palette::Foreground.into(),
        }))
        .line_set(ratatui::symbols::line::THICK)
        .ratio(ratio)
        .label(time);
    let duration = Line::from(duration).centered();

    frame.render_widget(gauge, gauge_area);
    frame.render_widget(duration, duration_area);
}

pub fn draw_lyrics(
    frame: &mut Frame,
    lyrics: &mut LyricState,
    image_state: Option<&Image>,
    area: Rect,
) {
    if !lyrics.active {
        return;
    }

    let (r, g, b) = image_state.unwrap().colour;

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    lyrics.scrollbar_state = lyrics.scrollbar_state.content_length(lyrics.lyrics.len());

    let paragraph = Paragraph::new(lyrics.lyrics.as_str())
        .fg(Color::Rgb(r, g, b))
        .left_aligned()
        .wrap(Wrap { trim: false })
        .block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(pad("Lyrics", 2))
                .title_style(Style::new().bold())
                .padding(Padding::proportional(1)),
        )
        .scroll((0, 0));

    frame.render_widget(paragraph, area);
    frame.render_stateful_widget(scrollbar, area, &mut lyrics.scrollbar_state);
}
