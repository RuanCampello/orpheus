use crate::internal::text::{Size, Text};
use crate::tui::colours::Palette;
use crate::tui::components::{pad, time_from_ms, BlockExt};
use crate::tui::state::{LyricState, State};
use deunicode::deunicode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text as UIText};
use ratatui::widgets::{Block, LineGauge, Padding, Paragraph, Wrap};
use ratatui::Frame;
use std::ops::Div;

const PLAY_ICON: [&str; 7] = [
    "██████████  ",
    "█████  ███████",
    "██████    ██████",
    "██████      ████",
    "██████    ██████",
    "█████  ███████",
    "██████████",
];

const PAUSE_ICON: [&str; 7] = [
    "██████████  ",
    "████  ██  ████",
    "█████  ██  █████",
    "█████  ██  █████",
    "█████  ██  █████",
    "████  ██  ████",
    "██████████",
];

pub fn draw_player<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let [image_area, remaining_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(area);

    if let Some(image) = &state.player.image {
        let image = Paragraph::new(image.ascii.to_string())
            .block(Block::new().secondary_border())
            .style(Style::new().fg(Color::White));
        frame.render_widget(image, image_area);
    }

    let [title_area, remaining_area] =
        Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(remaining_area);

    if let Some(playing) = &state.player.playing {
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
        let pause_icon = get_text_for_icon(&PAUSE_ICON);
        let play_icon = get_text_for_icon(&PLAY_ICON);

        #[rustfmt::skip]
        let status_icon = if playing.is_playing { pause_icon } else { play_icon };
        let [info_area, progress_bar] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(4)]).areas(remaining_area);

        let info_text =
            ratatui::text::Text::from(format!("\n「 {artist} 」\n\n {status_icon}")).centered();

        frame.render_widget(info_text, info_area);

        draw_progress_line(
            frame,
            playing.progress_ms.as_ref().unwrap(),
            &playing.item.as_ref().unwrap().duration_ms,
            progress_bar,
        );
    }
}

fn draw_progress_line<'a>(frame: &'a mut Frame, progress: &'a u32, duration: &'a u32, area: Rect) {
    let time = time_from_ms(progress);
    let progress = *progress as f64;
    let ratio = progress.div(*duration as f64);

    let duration = time_from_ms(duration);

    let [gauge_area, duration_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(6)]).areas(area);

    let gauge = LineGauge::default()
        .filled_style(Style::new().fg(Palette::Secondary.into()))
        .ratio(ratio)
        .label(time);
    let duration = Line::from(duration).centered();

    frame.render_widget(gauge, gauge_area);
    frame.render_widget(duration, duration_area);
}

pub fn draw_lyrics(frame: &mut Frame, lyrics: &LyricState, area: Rect) {
    if !lyrics.active {
        return;
    }

    let paragraph = Paragraph::new(lyrics.lyrics.as_str())
        .left_aligned()
        .wrap(Wrap { trim: false })
        .block(
            Block::new()
                .secondary_border()
                .title(pad("Lyrics", 2))
                .padding(Padding::proportional(1)),
        );

    frame.render_widget(paragraph, area)
}

fn get_text_for_icon<'a>(icon: &'a [&str]) -> UIText<'a> {
    UIText::from(icon.iter().map(|s| Line::from(*s)).collect::<Vec<Line>>())
}
