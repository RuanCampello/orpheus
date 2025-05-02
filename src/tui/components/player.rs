use crate::internal::config::ImageKind;
use crate::internal::image::{window_width_height, Rgb};
use crate::internal::text::{Size, Text as BigText};
use crate::tui::colours::Palette;
use crate::tui::components::{pad, time_from_ms, BlockExt};
use crate::tui::state::player::{AsTrack, LyricState, PlayerImage};
use crate::tui::state::State;
use deunicode::deunicode;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Style};
use ratatui::symbols::scrollbar;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, LineGauge, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
    Wrap,
};
use ratatui::Frame;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Div;
use std::sync::mpsc;
use std::thread;

pub fn draw_player<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let Some(playing) = &state.player.playing else {
        return;
    };

    let [mut image_area, remaining_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(area);

    let [title_area, remaining_area] =
        Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(remaining_area);

    let image_kind = state.get_player_image_kind();

    match &image_kind {
        ImageKind::Image => {
            if let PlayerImage::Image(image) = &mut state.player.image {
                let (width, height) = window_width_height(state.window.width, state.window.height);
                // image_area.width = width as _;
                // image_area.height = height as _;
                //

                frame.render_widget(ratatui_image::Image::new(image), image_area)
            }
        }
        ImageKind::Ascii => {
            if let PlayerImage::Ascii(ascii) = &mut state.player.image {
                let image = Paragraph::new(ascii.ascii.to_string())
                    .block(Block::new().secondary_border())
                    .style(Style::new().fg(match playing.is_playing {
                        true => Color::White,
                        false => Palette::Foreground.into(),
                    }));
                frame.render_widget(image, image_area);
            }
        }
    }

    let song = playing
        .item
        .as_ref()
        .and_then(|item| item.as_track())
        .unwrap();
    let song_name: &str = match song.name.is_ascii() {
        true => song.name.as_str(),
        false => &deunicode(&song.name),
    };

    let padding = ((title_area.width / 4) as usize).saturating_sub(song_name.len()) as f32;
    let padding = padding.div(1.5).round().min(5.0) as usize;

    let title = &[&Line::from(pad(song_name, padding))];
    let title = BigText::new().size(&Size::Quarter).lines(title);
    frame.render_widget(title, title_area);

    let artist = state.player.get_artist_name().unwrap_or("Unknown");

    let [info_area, progress_bar] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(4)]).areas(remaining_area);

    let info_text = ratatui::text::Text::from(format!("\n「 {artist} 」\n")).centered();
    frame.render_widget(info_text, info_area);

    draw_progress_line(
        frame,
        playing.progress_ms.as_ref().unwrap(),
        &playing
            .item
            .as_ref()
            .and_then(|item| item.as_track())
            .unwrap()
            .duration_ms,
        playing.is_playing,
        &state.colour,
        progress_bar,
    );
}

fn draw_progress_line<'a>(
    frame: &'a mut Frame,
    progress: &'a u32,
    duration: &'a u32,
    is_playing: bool,
    colour: &'a Rgb,
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
            true => colour.into(),
            false => Palette::Foreground.into(),
        }))
        .line_set(ratatui::symbols::line::THICK)
        .ratio(ratio)
        .label(time);
    let duration = Line::from(duration).centered();

    frame.render_widget(gauge, gauge_area);
    frame.render_widget(duration, duration_area);
}

fn calculate_styled_text(
    ordered_timestamps: &Option<Vec<u32>>,
    timed_lyrics: &Option<HashMap<u32, String>>,
    current_time: u32,
    colour: &Rgb,
) -> Option<Vec<Line<'static>>> {
    let Some(ordered_timestamps) = ordered_timestamps else {
        return None;
    };
    let Some(timed_lyrics) = timed_lyrics else {
        return None;
    };

    let styled_text = ordered_timestamps
        .iter()
        .enumerate()
        .filter_map(|(i, ts)| {
            timed_lyrics.get(ts).map(|text| {
                let next_ts = ordered_timestamps.get(i + 1);
                let colour: Color = match ts <= &current_time {
                    true => {
                        if let Some(next_ts) = next_ts {
                            match current_time < *next_ts {
                                true => colour.into(),
                                false => Color::White,
                            }
                        } else {
                            colour.into()
                        }
                    }
                    false => Color::Gray,
                };
                let text: Cow<'static, str> = Cow::Owned(text.clone());
                Line::from(Span::styled(text, Style::default().fg(colour)))
            })
        })
        .collect();

    Some(styled_text)
}

pub fn draw_lyrics(
    frame: &mut Frame,
    state: &mut LyricState,
    colour: Rgb,
    progress: u32,
    area: Rect,
) {
    let (sender, receiver) = mpsc::channel();

    let ordered_timestamps = &state.ordered_timestamps;
    let timed_lyrics = &state.timed_lyrics;
    thread::scope(|s| {
        s.spawn(move || {
            if let Some(styled) =
                calculate_styled_text(ordered_timestamps, timed_lyrics, progress, &colour)
            {
                sender.send(styled).unwrap();
            }
        });
    });

    let styled_lyrics = match receiver.recv() {
        Ok(lyrics) => lyrics,
        _ => state
            .lyrics
            .lines()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(colour.into()))))
            .collect(),
    };

    state.scrollbar_state = state.scrollbar_state.content_length(styled_lyrics.len());
    state.area = area;

    let paragraph = Paragraph::new(styled_lyrics)
        .fg(Color::from(colour))
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
        .scroll((state.offset as u16, 0));

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .symbols(scrollbar::VERTICAL)
        .track_symbol(Some("│"))
        .begin_symbol(None)
        .end_symbol(None);

    frame.render_widget(paragraph, area);
    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut state.scrollbar_state,
    );
    state.update_time(&progress);
}
