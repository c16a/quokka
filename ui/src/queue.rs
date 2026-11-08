//! The queue panel: the tracks that play after the current one. Rows can be dragged to reorder
//! and removed. These changes only touch the queue, never the files on disk.

use crate::i18n;
use eframe::egui;
use quokka_core::{MediaItem, PlaylistSummary};

use crate::format;
use crate::icons;
use crate::playback::{MenuRequest, Playback};
use crate::squircle::{self, Padding, SquircleButton};

/// Height of a queue row's content.
const ROW_HEIGHT: f32 = 52.0;

/// State kept between frames: the queue index of the row being dragged, if any.
#[derive(Default)]
pub struct QueueView {
    dragging: Option<usize>,
}

/// What the user asked for this frame. Applied after the rows are drawn.
#[derive(Default)]
struct Edits {
    clear: bool,
    remove: Option<usize>,
    jump: Option<usize>,
    /// The queue index of a row whose drag ended this frame.
    dropped: Option<usize>,
}

/// Draw the queue panel: a header with Clear, then the upcoming tracks.
pub fn show(ui: &mut egui::Ui, view: &mut QueueView, playback: &mut Playback) {
    let mut edits = Edits::default();
    let index = playback.index();
    let queue = playback.queue();
    let end = queue.len();
    let first = (index + 1).min(end);
    let danger = ui.visuals().error_fg_color;

    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.vertical(|ui| {
            ui.heading(i18n::up_next_2());
            ui.weak(match end - first {
                0 => i18n::nothing_queued().to_owned(),
                n => i18n::track_count(n),
            });
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.add_space(14.0);
            let clear = ui
                .add_enabled_ui(first < end, |ui| {
                    let label = egui::RichText::new(i18n::clear()).color(danger);
                    SquircleButton::new(egui::Label::new(label).extend())
                        .padding(squircle::TEXT_PADDING)
                        .fill(egui::Color32::TRANSPARENT)
                        .show(ui)
                })
                .inner;
            if clear.clicked() {
                edits.clear = true;
            }
        });
    });
    ui.add_space(8.0);

    if first == end {
        view.dragging = None;
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.weak(i18n::tracks_you_queue_will_appear_here());
        });
        return;
    }

    let mut rows: Vec<(usize, egui::Rect)> = Vec::with_capacity(end - first);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, item) in queue.iter().enumerate().skip(first) {
                // Reserved now, filled in once we know whether the row is hovered.
                let background = ui.painter().add(egui::Shape::Noop);
                let mut grip_rect = egui::Rect::NOTHING;
                let mut close_rect = egui::Rect::NOTHING;
                let frame = egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (grip, _) = ui.allocate_exact_size(
                                egui::vec2(14.0, ROW_HEIGHT - 12.0),
                                egui::Sense::hover(),
                            );
                            grip_rect = grip;
                            ui.add_space(6.0);
                            ui.vertical(|ui| {
                                // Leave room on the right for the duration and the remove button.
                                ui.set_max_width((ui.available_width() - 80.0).max(0.0));
                                ui.add(egui::Label::new(format::title(item)).truncate());
                                if let Some(artist) =
                                    item.artist.as_deref().or(item.album_artist.as_deref())
                                {
                                    ui.add(
                                        egui::Label::new(egui::RichText::new(artist).weak())
                                            .truncate(),
                                    );
                                }
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let (close, _) = ui.allocate_exact_size(
                                        egui::vec2(18.0, 18.0),
                                        egui::Sense::hover(),
                                    );
                                    close_rect = close;
                                    ui.add_space(6.0);
                                    ui.weak(format::duration(item.duration_ms));
                                },
                            );
                        });
                    });

                let row = frame.response.rect;
                let hovered = ui.rect_contains_pointer(row);
                // Registered after the row, so the grip and remove button sit on top of it.
                let row_response = ui.interact(
                    row,
                    ui.scope_id().with(("queue_row", i)),
                    egui::Sense::click(),
                );
                let grip = ui.interact(
                    grip_rect,
                    ui.scope_id().with(("queue_grip", i)),
                    egui::Sense::drag(),
                );
                let close = ui.interact(
                    close_rect,
                    ui.scope_id().with(("queue_close", i)),
                    egui::Sense::click(),
                );
                if hovered {
                    ui.painter().set(
                        background,
                        egui::Shape::rect_filled(row, 6.0, ui.visuals().faint_bg_color),
                    );
                    draw_grip(ui.painter(), grip_rect, ui.visuals().weak_text_color());
                    let close_color = if close.hovered() {
                        ui.visuals().strong_text_color()
                    } else {
                        ui.visuals().weak_text_color()
                    };
                    draw_close(ui.painter(), close_rect, close_color);
                }
                if grip.hovered() || view.dragging.is_some() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }

                if grip.drag_started() {
                    view.dragging = Some(i);
                }
                if grip.drag_stopped() {
                    view.dragging = None;
                    edits.dropped = Some(i);
                }
                if close.clicked() {
                    edits.remove = Some(i);
                } else if row_response.double_clicked() && !grip.hovered() {
                    edits.jump = Some(i);
                }
                rows.push((i, row));
            }
        });

    // Where a dragged row would land: before the row under the pointer, or at the end.
    let pointer = ui.ctx().input(|i| i.pointer.interact_pos());
    let before = pointer.map(|p| insertion_point(&rows, p.y, end));
    if let (Some(_), Some(before)) = (view.dragging, before) {
        let color = ui.visuals().selection.bg_fill;
        draw_drop_line(ui.painter(), &rows, before, color);
    }

    if edits.clear {
        playback.clear_upcoming();
    } else if let Some(i) = edits.remove {
        playback.remove_at(i);
    } else if let Some(i) = edits.jump {
        playback.jump_to(i);
    } else if let (Some(from), Some(before)) = (edits.dropped, before) {
        playback.move_upcoming(from, before);
    }
}

/// The queue index a dragged row would be inserted before, given the pointer's height.
fn insertion_point(rows: &[(usize, egui::Rect)], y: f32, end: usize) -> usize {
    rows.iter()
        .find(|(_, r)| y < r.center().y)
        .map_or(end, |(i, _)| *i)
}

/// A line where a dragged row would be inserted.
fn draw_drop_line(
    painter: &egui::Painter,
    rows: &[(usize, egui::Rect)],
    before: usize,
    color: egui::Color32,
) {
    let Some((_, first)) = rows.first() else {
        return;
    };
    let y = match rows.iter().find(|(i, _)| *i == before) {
        Some((_, r)) => r.top(),
        None => rows.last().map_or(first.top(), |(_, r)| r.bottom()),
    };
    let stroke = egui::Stroke::new(2.0, color);
    painter.line_segment(
        [
            egui::pos2(first.left() + 12.0, y),
            egui::pos2(first.right() - 12.0, y),
        ],
        stroke,
    );
}

/// Six dots in two columns: the drag handle.
fn draw_grip(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    icons::paint(
        painter,
        rect.center(),
        icons::DOTS_SIX_VERTICAL,
        16.0,
        color,
    );
}

/// A cross: removes the row from the queue.
fn draw_close(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    icons::paint(painter, rect.center(), icons::X, 14.0, color);
}

/// The right-click menu for a track: play it next, or add it to the end of the queue.
pub fn track_menu(
    ui: &mut egui::Ui,
    queue: &mut Vec<MenuRequest>,
    track: &MediaItem,
    playlists: &[PlaylistSummary],
) {
    ui.set_min_width(180.0);
    ui.spacing_mut().item_spacing.y = 4.0;
    ui.spacing_mut().button_padding = egui::vec2(14.0, 6.0);
    if menu_row(ui, i18n::play_next()) {
        queue.push(MenuRequest::PlayNext(track.clone()));
        ui.close();
    }
    if menu_row(ui, i18n::add_to_queue()) {
        queue.push(MenuRequest::AddToQueue(track.clone()));
        ui.close();
    }
    ui.menu_button(i18n::add_to_playlist(), |ui| {
        if playlists.is_empty() {
            ui.weak(i18n::no_playlists_yet());
        }
        for playlist in playlists {
            if squircle::text_button(ui, playlist.name.as_str()).clicked() {
                queue.push(MenuRequest::AddToPlaylist {
                    playlist: playlist.id,
                    media_id: track.id,
                });
                ui.close();
            }
        }
    });
    if menu_row(ui, i18n::track_info()) {
        queue.push(MenuRequest::ShowInfo(track.clone()));
        ui.close();
    }
}

/// One full-width entry in a menu, with its label inset from the left edge. Returns true when
/// it was clicked.
fn menu_row(ui: &mut egui::Ui, label: &str) -> bool {
    let width = ui.available_width();
    let text = label.to_owned();
    // The label fills the row, less the padding either side, so the squircle spans the full width.
    let content =
        move |ui: &mut egui::Ui| ui.add_sized([width - 2.0 * 14.0, 18.0], egui::Label::new(text));
    SquircleButton::new(content)
        .padding(Padding::symmetric(14.0, 6.0))
        .show(ui)
        .clicked()
}
