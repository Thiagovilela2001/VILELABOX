//! Primeiro marco visível do VILELABOX em Rust: biblioteca lida das análises
//! ANLZ e linha do tempo de compassos e frases da faixa escolhida. Sem áudio.
//! Só leitura: nada é gravado na pasta de análises.

mod library;

use slint::{Color, ModelRc, SharedString, StandardListViewItem, VecModel};
use std::path::PathBuf;
use std::rc::Rc;

slint::slint! {
    import { StandardListView } from "std-widgets.slint";

    export struct Bar { pos: float, }
    export struct Segment { start: float, end: float, color: color, label: string, }

    export component AppWindow inherits Window {
        title: "VILELABOX";
        preferred-width: 1200px;
        preferred-height: 680px;
        background: #16181c;

        in property <[StandardListViewItem]> tracks;
        in property <string> status;
        in property <string> sel-title;
        in property <string> sel-info;
        in property <[Bar]> bars;
        in property <[Segment]> phrases;
        callback selected(int);

        HorizontalLayout {
            padding: 8px;
            spacing: 8px;

            VerticalLayout {
                width: 420px;
                spacing: 4px;
                Text { text: root.status; color: #9aa0a6; font-size: 12px; }
                StandardListView {
                    model: root.tracks;
                    current-item-changed(i) => { root.selected(i); }
                }
            }

            VerticalLayout {
                spacing: 8px;
                alignment: start;
                Text { text: root.sel-title; color: #e8eaed; font-size: 18px; wrap: word-wrap; }
                Text { text: root.sel-info; color: #9aa0a6; font-size: 13px; }

                Rectangle {
                    height: 120px;
                    background: #0d0e10;
                    border-color: #2a2d33;
                    border-width: 1px;
                    clip: true;

                    for p in root.phrases: Rectangle {
                        x: parent.width * p.start;
                        width: parent.width * (p.end - p.start);
                        height: parent.height;
                        background: p.color;
                        Text {
                            x: 4px;
                            y: 4px;
                            text: p.label;
                            color: #ffffff;
                            font-size: 10px;
                        }
                    }

                    for b in root.bars: Rectangle {
                        x: parent.width * b.pos;
                        y: parent.height * 0.6;
                        width: 1px;
                        height: parent.height * 0.4;
                        background: #c8c8c8;
                    }
                }

                Text {
                    text: "Linha do tempo: frases coloridas; traços = compassos (downbeats).";
                    color: #5f6368;
                    font-size: 11px;
                }
            }
        }
    }
}

fn default_dir() -> PathBuf {
    let appdata = std::env::var_os("APPDATA").unwrap_or_default();
    PathBuf::from(appdata).join(r"Pioneer\rekordbox\share\PIONEER\USBANLZ")
}

fn phrase_color(kind: &str) -> Color {
    let rgb = match kind {
        "INTRO" => (0x2b, 0x5d, 0xa8),
        "UP" | "VERSE" => (0x2e, 0x8b, 0x57),
        "CHORUS" => (0xd9, 0x7a, 0x1e),
        "DOWN" | "BRIDGE" => (0x7b, 0x4f, 0xa8),
        "OUTRO" => (0x55, 0x5a, 0x63),
        _ => (0x3c, 0x40, 0x46),
    };
    Color::from_rgb_u8(rgb.0, rgb.1, rgb.2)
}

fn main() -> Result<(), slint::PlatformError> {
    let dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_dir);
    let tracks = Rc::new(library::scan(&dir));

    let ui = AppWindow::new()?;
    let items: Vec<StandardListViewItem> = tracks
        .iter()
        .map(|t| {
            let bpm = t
                .grid
                .as_ref()
                .map_or("—".into(), |g| format!("{:.2}", g.bpm));
            StandardListViewItem::from(SharedString::from(format!("{}   ·   {} BPM", t.title, bpm)))
        })
        .collect();
    ui.set_tracks(ModelRc::new(VecModel::from(items)));
    ui.set_status(SharedString::from(format!(
        "{} faixas em {}",
        tracks.len(),
        dir.display()
    )));

    let weak = ui.as_weak();
    let lib = tracks.clone();
    ui.on_selected(move |index| {
        let (Some(ui), Some(track)) = (
            weak.upgrade(),
            usize::try_from(index).ok().and_then(|i| lib.get(i)),
        ) else {
            return;
        };
        let view = library::timeline(track);
        ui.set_sel_title(SharedString::from(track.title.as_str()));
        ui.set_sel_info(SharedString::from(view.info.as_str()));
        let bars: Vec<Bar> = view.bars.iter().map(|&pos| Bar { pos }).collect();
        let phrases: Vec<Segment> = view
            .phrases
            .iter()
            .map(|p| Segment {
                start: p.start,
                end: p.end,
                color: phrase_color(&p.kind),
                label: SharedString::from(p.kind.as_str()),
            })
            .collect();
        ui.set_bars(ModelRc::new(VecModel::from(bars)));
        ui.set_phrases(ModelRc::new(VecModel::from(phrases)));
    });

    ui.run()
}
