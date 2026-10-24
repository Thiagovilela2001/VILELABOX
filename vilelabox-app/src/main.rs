//! Primeiro marco visível do VILELABOX em Rust: interface completa
//! inspirada no fluxo de trabalho do rekordbox, com iconografia desenhada
//! para o VILELABOX: nenhum arquivo visual de terceiro é usado no crate.

mod library;

use slint::{Color, ModelRc, SharedString, VecModel};
use std::path::PathBuf;
use std::rc::Rc;

slint::slint! {
    import { ListView, ScrollView } from "std-widgets.slint";

    export struct Bar { pos: float, }
    export struct Segment { start: float, end: float, color: color, label: string, }

    export struct TrackRow {
        id: int,
        title: string,
        artist: string,
        bpm: string,
        key: string,
        duration: string,
        bars_count: int,
        alt: bool,
    }

    component TopMenuButton inherits Rectangle {
        in property <float> s: 1.0;
        in property <string> text;
        height: ((28px * s));
        background: touch.has_hover ? #1a1c24 : transparent;
        border-radius: ((3px * s));
        touch := TouchArea { }
        HorizontalLayout {
            padding_left: ((10px * s));
            padding_right: ((10px * s));
            Text {
                text: root.text;
                color: touch.has_hover ? #ffffff : #c5c9d6;
                font_size: ((12px * s));
                vertical_alignment: center;
            }
        }
    }

    component HeaderPill inherits Rectangle {
        in property <float> s: 1.0;
        in property <string> label;
        in property <color> dot_color: #00a2ff;
        in property <bool> active: false;
        height: ((24px * s));
        background: root.active ? rgb(30, 34, 48) : #141620;
        border_color: root.active ? #00a2ff : #2d3142;
        border_width: ((1px * s));
        border_radius: ((3px * s));
        touch := TouchArea { }
        HorizontalLayout {
            padding_left: ((8px * s));
            padding_right: ((8px * s));
            spacing: ((6px * s));
            Rectangle {
                width: ((6px * s));
                height: ((6px * s));
                background: root.dot_color;
                border_radius: ((3px * s));
                y: (parent.height - self.height) / 2;
            }
            Text {
                text: root.label;
                color: #ffffff;
                font_size: ((11px * s));
                font_weight: 700;
                vertical_alignment: center;
            }
        }
    }

    // ==========================================
    // Componentes do layout (geometria do rekordbox 7.2.8)
    // ==========================================

    component RailButton inherits Rectangle {
        in property <float> s: 1.0;
        in property <string> glyph;
        width: ((24px * s));
        height: ((24px * s));
        border_radius: ((3px * s));
        background: #1b1d24;
        border_width: ((1px * s));
        border_color: #262933;
        Text {
            text: root.glyph;
            color: #9aa1b5;
            font_size: ((11px * s));
            horizontal_alignment: center;
            vertical_alignment: center;
        }
    }

    component IconRail inherits Rectangle {
        in property <float> s: 1.0;
        width: ((36px * s));
        background: #16171c;
        border_color: #24262f;
        border_width: ((1px * s));
        VerticalLayout {
            padding_top: ((8px * s));
            padding_bottom: ((8px * s));
            spacing: ((4px * s));
            alignment: start;
            RailButton { s: root.s; glyph: "↕"; }
            RailButton { s: root.s; glyph: "⤢"; }
            RailButton { s: root.s; glyph: "⊕"; }
            RailButton { s: root.s; glyph: "⊖"; }
            RailButton { s: root.s; glyph: "▤"; }
            RailButton { s: root.s; glyph: "☰"; }
            RailButton { s: root.s; glyph: "♫"; }
            RailButton { s: root.s; glyph: "⊞"; }
            RailButton { s: root.s; glyph: "◫"; }
            RailButton { s: root.s; glyph: "≡"; }
            RailButton { s: root.s; glyph: "⚙"; }
        }
    }

    component MixerStrip inherits Rectangle {
        in property <float> s: 1.0;
        background: #0e1015;
        border_color: #1e2130;
        border_width: ((1px * s));

        HorizontalLayout {
            padding: ((4px * s));
            spacing: ((4px * s));
            alignment: center;

            for lado in 2 : VerticalLayout {
                spacing: ((6px * s));
                alignment: center;
                for i in 4 : VerticalLayout {
                    spacing: ((1px * s));
                    alignment: center;
                    Rectangle {
                        width: ((30px * s));
                        height: ((30px * s));
                        border_radius: ((15px * s));
                        background: #23262f;
                        border_color: #3a3f4d;
                        border_width: ((1px * s));
                        Rectangle { x: ((13px * s)); y: ((3px * s)); width: ((2px * s)); height: ((11px * s)); background: #8fd0ff; }
                    }
                    Text { text: ["TRIM", "HIGH", "MID", "LOW"][i]; color: #8b92a5; font_size: ((6px * s)); font_weight: 700; horizontal_alignment: center; }
                }
                Rectangle {
                    width: ((18px * s));
                    height: ((18px * s));
                    border_radius: ((2px * s));
                    background: #1b1e28;
                    border_color: #3a4152;
                    border_width: ((1px * s));
                    Text { text: "CUE"; color: #8b92a5; font_size: ((6px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; }
                }
            }
        }
    }

    component MixerBar inherits Rectangle {
        in property <float> s: 1.0;
        background: #0f1116;
        border_color: #1e2130;
        border_width: ((1px * s));

        HorizontalLayout {
            padding_left: ((10px * s));
            padding_right: ((10px * s));
            spacing: ((10px * s));
            alignment: center;

            Text { text: "🎧"; font_size: ((12px * s)); color: #8b92a5; }
            for etq in 2 : VerticalLayout {
                spacing: ((1px * s));
                alignment: center;
                Rectangle {
                    width: ((30px * s));
                    height: ((30px * s));
                    border_radius: ((15px * s));
                    background: #23262f;
                    border_color: #3a3f4d;
                    border_width: ((1px * s));
                    Rectangle { x: ((13px * s)); y: ((3px * s)); width: ((2px * s)); height: ((11px * s)); background: #8fd0ff; }
                }
                Text { text: ["MIX", "LEVEL"][etq]; color: #8b92a5; font_size: ((6px * s)); font_weight: 700; horizontal_alignment: center; }
            }

            // faders de canal 1–4 com cap 24x14
            for i in 4 : Rectangle {
                width: ((26px * s));
                height: ((34px * s));
                Rectangle {
                    x: ((10px * s));
                    width: ((6px * s));
                    height: ((34px * s));
                    background: #06070a;
                    border_color: #272b3c;
                    border_width: ((1px * s));
                }
                Rectangle {
                    x: ((1px * s));
                    y: i == 0 ? ((17px * s)) : ((9px * s));
                    width: ((24px * s));
                    height: ((14px * s));
                    border_radius: ((2px * s));
                    background: #d6dae6;
                }
                Text { text: i + 1; y: ((36px * s)); width: ((26px * s)); color: #6d7488; font_size: ((6px * s)); font_weight: 700; horizontal_alignment: center; }
            }

            // crossfader
            Rectangle {
                width: ((150px * s));
                height: ((10px * s));
                background: #06070a;
                border_color: #272b3c;
                border_width: ((1px * s));
                Rectangle { x: ((64px * s)); y: -((2px * s)); width: ((24px * s)); height: ((14px * s)); border_radius: ((2px * s)); background: #d6dae6; }
            }
        }
    }

    component StatusBar inherits Rectangle {
        in property <float> s: 1.0;
        background: #101117;
        border_color: #24262f;
        border_width: ((1px * s));
        HorizontalLayout {
            padding_left: ((12px * s));
            padding_right: ((12px * s));
            spacing: ((10px * s));
            Text { text: "VILELABOX"; color: #7f879b; font_size: ((10px * s)); font_weight: 800; vertical_alignment: center; }
            Rectangle { horizontal_stretch: 1; }
            Text { text: root.status; color: #636b80; font_size: ((10px * s)); vertical_alignment: center; }
        }
        in property <string> status: "pronto";
    }


    // ==========================================
    // DECK — arranjo do rekordbox: cabeçalho (68) / onda (54) / corpo (163)
    // ==========================================
    component Deck inherits Rectangle {
        in property <float> s: 1.0;
        in property <int> numero: 1;
        in property <string> titulo;
        in property <string> artista;
        in property <string> duracao;
        in property <string> bpm;
        in property <string> tom;
        in property <[Segment]> phrases;
        in property <[Bar]> bars;
        in property <[string]> letras: ["A", "B", "C", "D", "E", "F", "G", "H"];
        in property <[string]> nomes: ["INTRO", "VERSE", "CHORUS", "DROP", "BREAK", "BUILD", "OUTRO", "CUE"];
        in property <[color]> cores: [#00a2ff, #00c46a, #ff8a00, #ff2d55, #7b61ff, #ffd400, #00c4b4, #8b92a5];
        background: transparent;

        VerticalLayout {
            spacing: (0px * s);

            // ---------- CABEÇALHO (68) ----------
            Rectangle {
                height: ((68px * s));
                background: #0e1015;
                border_color: #1e2130;
                border_width: ((1px * s));

                HorizontalLayout {
                    padding_left: ((8px * s));
                    padding_right: ((8px * s));
                    spacing: ((8px * s));
                    alignment: center;

                    Rectangle {
                        width: ((22px * s));
                        height: ((22px * s));
                        border_radius: ((3px * s));
                        background: #00a2ff;
                        Text { text: root.numero; color: #05070b; font_size: ((13px * s)); font_weight: 900; horizontal_alignment: center; vertical_alignment: center; }
                    }

                    Rectangle {
                        width: ((30px * s));
                        height: ((30px * s));
                        background: #1a1d27;
                        border_color: #2b3040;
                        border_width: ((1px * s));
                        border_radius: ((3px * s));
                        Text { text: "♪"; color: #666d80; font_size: ((13px * s)); horizontal_alignment: center; vertical_alignment: center; }
                    }

                    VerticalLayout {
                        vertical_stretch: 1;
                        spacing: ((1px * s));
                        alignment: center;
                        Text { text: root.titulo == "" ? "Not Loaded." : root.titulo; color: root.titulo == "" ? #6d7488 : #ffffff; font_size: ((11px * s)); font_weight: 700; overflow: elide; }
                        Text { text: root.artista; color: #78809a; font_size: ((9px * s)); overflow: elide; }
                    }

                    Text { text: root.duracao; color: #aeb6c8; font_size: ((10px * s)); font_weight: 700; }
                    Text { text: "-00:00.0"; color: #6d7488; font_size: ((10px * s)); font_weight: 700; }

                    Rectangle { width: ((62px * s)); height: ((22px * s)); border_radius: ((3px * s)); background: #171a23; border_color: #2b3040; border_width: ((1px * s));
                        Text { text: "KEY SYNC"; color: #8b92a5; font_size: ((8px * s)); font_weight: 700; horizontal_alignment: center; vertical_alignment: center; } }
                    Rectangle { width: ((62px * s)); height: ((22px * s)); border_radius: ((3px * s)); background: #171a23; border_color: #2b3040; border_width: ((1px * s));
                        Text { text: "BEAT SYNC"; color: #8b92a5; font_size: ((8px * s)); font_weight: 700; horizontal_alignment: center; vertical_alignment: center; } }
                    Rectangle { width: ((18px * s)); height: ((18px * s)); border_radius: ((9px * s)); background: #3a4152; border_color: #5a6478; border_width: ((1px * s)); }
                }
            }

            // ---------- FAIXA DA FORMA DE ONDA (54) ----------
            Rectangle {
                height: ((54px * s));
                background: #06070a;
                border_color: #1e2130;
                border_width: ((1px * s));
                clip: true;

                // frases reais do ANLZ (start/end normalizados 0..1)
                for p in root.phrases : Rectangle {
                    x: parent.width * p.start;
                    width: parent.width * (p.end - p.start);
                    height: parent.height;
                    background: p.color.with_alpha(0.20);
                    border_color: p.color.with_alpha(0.55);
                    border_width: ((1px * s));
                    Text { x: ((3px * s)); y: ((2px * s)); text: p.label; color: p.color.with_alpha(0.95); font_size: ((7px * s)); font_weight: 700; }
                }

                // batidas (downbeats)
                for b in root.bars : Rectangle {
                    x: parent.width * b.pos;
                    y: parent.height * 0.42;
                    width: ((1px * s));
                    height: parent.height * 0.58;
                    background: #ffffff;
                    opacity: 0.18;
                }

                // linha central e cursor
                Rectangle { y: parent.height * 0.5 - ((1px * s)); width: parent.width; height: ((1px * s)); background: #22283a; }
                Rectangle { x: parent.width * 0.42; width: ((2px * s)); height: parent.height; background: #ff2d55; }
            }

            // ---------- CORPO (163) ----------
            HorizontalLayout {
                padding: ((5px * s));
                spacing: ((6px * s));
                vertical_stretch: 1;

                // transporte: quadrados 30x30
                VerticalLayout {
                    spacing: ((4px * s));
                    alignment: center;
                    HorizontalLayout {
                        spacing: ((4px * s));
                        alignment: center;
                        Rectangle { width: ((30px * s)); height: ((30px * s)); border_radius: ((15px * s)); background: #1b1e28; border_color: #00a2ff; border_width: ((2px * s));
                            Text { text: "CUE"; color: #00a2ff; font_size: ((8px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; } }
                        Rectangle { width: ((30px * s)); height: ((30px * s)); border_radius: ((15px * s)); background: #1b1e28; border_color: #3a4152; border_width: ((1px * s));
                            Text { text: "▶"; color: #c5c9d6; font_size: ((11px * s)); horizontal_alignment: center; vertical_alignment: center; } }
                        Rectangle { width: ((30px * s)); height: ((30px * s)); border_radius: ((15px * s)); background: #1b1e28; border_color: #3a4152; border_width: ((1px * s));
                            Text { text: "⟳"; color: #c5c9d6; font_size: ((13px * s)); horizontal_alignment: center; vertical_alignment: center; } }
                    }
                    Text { text: "4 BEAT"; color: #6d7488; font_size: ((8px * s)); font_weight: 700; horizontal_alignment: center; }
                }

                // pads A–H: botões 50x26, duas fileiras de quatro
                VerticalLayout {
                    spacing: ((4px * s));
                    alignment: center;
                    HorizontalLayout {
                        spacing: ((4px * s));
                        alignment: center;
                        for i in 4 : Rectangle {
                            width: ((46px * s));
                            height: ((26px * s));
                            border_radius: ((3px * s));
                            background: #14161f;
                            border_color: root.cores[i].with_alpha(0.55);
                            border_width: ((1px * s));
                            VerticalLayout {
                                padding: ((2px * s));
                                spacing: (0px * s);
                                Text { text: root.letras[i] + "  " + root.nomes[i]; color: root.cores[i]; font_size: ((7px * s)); font_weight: 800; horizontal_alignment: center; }
                                Text { text: "--:--"; color: #5c6376; font_size: ((7px * s)); horizontal_alignment: center; }
                            }
                        }
                    }
                    HorizontalLayout {
                        spacing: ((4px * s));
                        alignment: center;
                        for i in 4 : Rectangle {
                            width: ((46px * s));
                            height: ((26px * s));
                            border_radius: ((3px * s));
                            background: #14161f;
                            border_color: root.cores[i + 4].with_alpha(0.55);
                            border_width: ((1px * s));
                            VerticalLayout {
                                padding: ((2px * s));
                                spacing: (0px * s);
                                Text { text: root.letras[i + 4] + "  " + root.nomes[i + 4]; color: root.cores[i + 4]; font_size: ((7px * s)); font_weight: 800; horizontal_alignment: center; }
                                Text { text: "--:--"; color: #5c6376; font_size: ((7px * s)); horizontal_alignment: center; }
                            }
                        }
                    }
                }

                // BPM: caixa 50x26 + leitura grande
                VerticalLayout {
                    spacing: ((3px * s));
                    alignment: center;
                    HorizontalLayout {
                        spacing: ((4px * s));
                        alignment: center;
                        Rectangle { width: ((50px * s)); height: ((26px * s)); border_radius: ((3px * s)); background: #16202b; border_color: #2f6f96; border_width: ((1px * s));
                            Text { text: "×1"; color: #9fd8ff; font_size: ((9px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; } }
                        Rectangle { width: ((44px * s)); height: ((26px * s)); border_radius: ((3px * s)); background: #171a23; border_color: #2b3040; border_width: ((1px * s));
                            Text { text: root.tom; color: #d6dae6; font_size: ((10px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; } }
                    }
                    Text { text: root.bpm; color: #00d0ff; font_size: ((17px * s)); font_weight: 900; horizontal_alignment: center; }
                }

                // SLIP / Q / MT: quadrados 30x30
                VerticalLayout {
                    spacing: ((4px * s));
                    alignment: center;
                    HorizontalLayout {
                        spacing: ((4px * s));
                        alignment: center;
                        Rectangle { width: ((30px * s)); height: ((30px * s)); border_radius: ((3px * s)); background: #1b1e28; border_color: #3a4152; border_width: ((1px * s));
                            Text { text: "SLIP"; color: #8b92a5; font_size: ((7px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; } }
                        Rectangle { width: ((30px * s)); height: ((30px * s)); border_radius: ((3px * s)); background: #1b1e28; border_color: #3a4152; border_width: ((1px * s));
                            Text { text: "Q"; color: #8b92a5; font_size: ((9px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; } }
                    }
                    Rectangle { width: ((30px * s)); height: ((26px * s)); border_radius: ((3px * s)); background: #1b1e28; border_color: #3a4152; border_width: ((1px * s));
                        Text { text: "MT"; color: #8b92a5; font_size: ((8px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; } }
                }

                // pitch: trilho com cap 24x14 e passos 14x15
                VerticalLayout {
                    spacing: ((4px * s));
                    alignment: center;
                    Rectangle { width: ((46px * s)); height: ((18px * s)); border_radius: ((2px * s)); background: #171a23; border_color: #2b3040; border_width: ((1px * s));
                        Text { text: "±6%"; color: #8b92a5; font_size: ((8px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; } }
                    Rectangle {
                        width: ((8px * s));
                        height: ((92px * s));
                        background: #06070a;
                        border_color: #272b3c;
                        border_width: ((1px * s));
                        border_radius: ((4px * s));
                        Rectangle { y: parent.height * 0.5 - ((1px * s)); x: -((9px * s)); width: ((26px * s)); height: ((2px * s)); background: #00a2ff; }
                        Image { source: @image-url("../assets/ui/ch_fader.svg"); y: parent.height * 0.5 - ((7px * s)); x: -((9px * s)); width: ((24px * s)); height: ((14px * s)); }
                    }
                    Text { text: "0.0%"; color: #00a2ff; font_size: ((9px * s)); font_weight: 800; horizontal_alignment: center; }
                }
            }
        }
    }

    export component AppWindow inherits Window {
        title: "VILELABOX — Biblioteca e Decks";
        preferred_width: ((1180px * s));
        preferred_height: ((660px * s));
        background: #090a0d;

        // Escala global: 1.0 equivale à captura de referência 1917x1078
        in-out property <float> s: 1.0;   // definido pelo Rust a partir do tamanho da janela

        in property <[TrackRow]> track_rows;
        in property <int> selected_row: 0;
        in property <string> status: "2661 faixas carregadas";
        in property <string> sel_title: "Nenhuma faixa selecionada";
        in property <string> sel_artist: "Selecione uma faixa da biblioteca abaixo";
        in property <string> sel_bpm: "128.00";
        in property <string> sel_key: "8A";
        in property <string> sel_duration: "03:45";
        in property <string> sel_info: "Linha do tempo: compassos e frases do ANLZ";
        in property <[Bar]> bars;
        in property <[Segment]> phrases;

        callback track_selected(int);

        VerticalLayout {
            spacing: ((0px * s));

            // ==========================================
            // 1. BARRA DE MENUS
            // ==========================================
            Rectangle {
                height: ((28px * s));
                background: #0c0d12;
                border_color: #1a1c26;
                border_width: ((1px * s));

                HorizontalLayout {
                    padding_left: ((14px * s));
                    padding_right: ((14px * s));
                    spacing: ((12px * s));

                    // Marca VILELABOX — mesma caixa de 102x14 da barra original
                    Rectangle {
                        width: ((102px * s));
                        height: ((14px * s));
                        HorizontalLayout {
                            alignment: start;
                            spacing: ((6px * s));
                            Image {
                                source: @image-url("../assets/ui/mark.svg");
                                width: ((12px * s));
                                height: ((12px * s));
                                vertical_alignment: center;
                            }
                            Text {
                                text: "VILELABOX";
                                color: #ffffff;
                                font_size: ((13px * s));
                                font_weight: 800;
                                letter_spacing: ((1.4px * s));
                                vertical_alignment: center;
                            }
                        }
                    }

                    Rectangle { width: ((1px * s)); height: ((18px * s)); background: #252836; y: (parent.height - self.height) / 2; }

                    TopMenuButton { s: root.s; text: "Ficheiro"; }
                    TopMenuButton { s: root.s; text: "Editar"; }
                    TopMenuButton { s: root.s; text: "Ver"; }
                    TopMenuButton { s: root.s; text: "Faixa"; }
                    TopMenuButton { s: root.s; text: "Lista de reprodução"; }
                    TopMenuButton { s: root.s; text: "Exportar"; }
                    TopMenuButton { s: root.s; text: "Ajuda"; }

                    Rectangle { horizontal_stretch: 1; }

                    // Assinatura própria no canto direito — mesma caixa de 73x10
                    Rectangle {
                        width: ((73px * s));
                        height: ((12px * s));
                        background: #ff0000;
                        HorizontalLayout {
                            alignment: end;
                            spacing: ((4px * s));
                            Text {
                                text: "vilelabox";
                                color: #ffffff;
                                font_size: ((8px * s));
                                vertical_alignment: center;
                            }
                            Image {
                                source: @image-url("../assets/ui/mark.svg");
                                width: ((10px * s));
                                height: ((10px * s));
                                vertical_alignment: center;
                            }
                        }
                    }
                }
            }

            // ==========================================
            // 2. PERFORMANCE / GLOBAL CONTROL TOOLBAR
            // ==========================================
            Rectangle {
                height: ((36px * s));
                background: #111218;
                border_color: rgb(30, 32, 44);
                border_width: ((1px * s));

                HorizontalLayout {
                    padding_left: ((14px * s));
                    padding_right: ((14px * s));
                    spacing: ((10px * s));

                    HeaderPill { s: root.s; label: "PERFORMANCE  ▾"; dot_color: #00a2ff; active: true; }

                    Image {
                        source: @image-url("../assets/ui/transport_bar.svg");
                        width: ((90px * s));
                        height: ((16px * s));
                        vertical_alignment: center;
                    }

                    Rectangle { width: ((1px * s)); height: ((16px * s)); background: #222533; y: (parent.height - self.height) / 2; }

                    HeaderPill { s: root.s; label: "FX"; dot_color: #ff9800; active: false; }
                    HeaderPill { s: root.s; label: "SAMPLER"; dot_color: #9c27b0; active: false; }
                    HeaderPill { s: root.s; label: "MIXER"; dot_color: #00bcd4; active: false; }

                    // Pro DJ Link status
                    Rectangle {
                        height: ((24px * s));
                        background: #102419;
                        border_color: #10b981;
                        border_width: ((1px * s));
                        border_radius: ((3px * s));
                        HorizontalLayout {
                            padding_left: ((8px * s));
                            padding_right: ((8px * s));
                            spacing: ((6px * s));
                            Rectangle { width: ((6px * s)); height: ((6px * s)); background: #10b981; border_radius: ((3px * s)); y: (parent.height - self.height) / 2; }
                            Text { text: "LINK ATIVO"; color: #10b981; font_size: ((10px * s)); font_weight: 800; vertical_alignment: center; }
                        }
                    }

                    Rectangle { horizontal_stretch: 1; }

                    // Master output icon
                    Image {
                        source: @image-url("../assets/ui/master_out.svg");
                        width: ((22px * s));
                        height: ((22px * s));
                        vertical_alignment: center;
                    }

                    Rectangle { width: ((1px * s)); height: ((16px * s)); background: #222533; y: (parent.height - self.height) / 2; }

                    Text { text: "PAD    MIDI    Free    ⚙    10:24"; color: #727992; font_size: ((11px * s)); vertical_alignment: center; }
                }
            }

            HorizontalLayout {
                vertical_stretch: 1;
                spacing: ((0px * s));

                IconRail { s: root.s; height: 100%; }

                VerticalLayout {
                    spacing: ((0px * s));

            // ==========================================
            // 3. DECKS 1 e 2 + FAIXA DO MIXER  (339 = 285 + 54)
            // ==========================================
            Rectangle {
                height: ((339px * s));
                background: #0b0c10;
                border_color: #1a1c27;
                border_width: ((1px * s));

                VerticalLayout {
                    spacing: (0px * s);

                    HorizontalLayout {
                        spacing: (0px * s);
                        vertical_stretch: 1;

                        Deck {
                            s: root.s;
                            numero: 1;
                            width: (853px * s);
                            clip: true;
                            titulo: root.sel_title;
                            artista: root.sel_artist;
                            duracao: root.sel_duration;
                            bpm: root.sel_bpm;
                            tom: root.sel_key;
                            phrases: root.phrases;
                            bars: root.bars;
                        }

                        MixerStrip { s: root.s; width: (133px * s); clip: true; }

                        Deck {
                            s: root.s;
                            numero: 2;
                            width: (887px * s);
                            clip: true;
                            titulo: root.sel_title;
                            artista: root.sel_artist;
                            duracao: root.sel_duration;
                            bpm: root.sel_bpm;
                            tom: root.sel_key;
                            phrases: root.phrases;
                            bars: root.bars;
                        }
                    }

                    // barra do mixer: 555..1119 na captura
                    HorizontalLayout {
                        spacing: (0px * s);
                        height: ((54px * s));
                        Rectangle { width: 29%; height: ((54px * s)); }
                        MixerBar { s: root.s; width: 29.4%; height: ((54px * s)); }
                        Rectangle { horizontal_stretch: 1; height: ((54px * s)); }
                    }
                }
            }

            // ==========================================
            // 4. BROWSER & COLLECTION SECTION
            // ==========================================
            Rectangle {
                vertical_stretch: 1;
                min-height: ((0px * s));
                preferred-height: ((0px * s));
                clip: true;
                background: #0a0b0e;

                VerticalLayout {
                    spacing: ((0px * s));
                    min-height: ((0px * s));

                    // Browser Toolbar (Search & status)
                    Rectangle {
                        height: ((38px * s));
                        background: #12141c;
                        border_color: #1f2230;
                        border_width: ((1px * s));

                        HorizontalLayout {
                            padding_left: ((12px * s));
                            padding_right: ((12px * s));
                            spacing: ((12px * s));

                            Text { text: "ÁRVORE DE NAVEGAÇÃO"; color: #62697d; font_size: ((11px * s)); font_weight: 800; vertical_alignment: center; }

                            Rectangle { width: ((1px * s)); height: ((16px * s)); background: #262a3c; y: (parent.height - self.height) / 2; }

                            // Search bar
                            Rectangle {
                                width: ((360px * s));
                                height: ((26px * s));
                                background: #0a0b0f;
                                border_color: #2d3247;
                                border_width: ((1px * s));
                                border_radius: ((3px * s));
                                y: (parent.height - self.height) / 2;

                                HorizontalLayout {
                                    padding_left: ((8px * s));
                                    padding_right: ((8px * s));
                                    spacing: ((6px * s));
                                    Image {
                                        source: @image-url("../assets/ui/search.svg");
                                        width: ((14px * s));
                                        height: ((14px * s));
                                        vertical_alignment: center;
                                    }
                                    Text {
                                        text: "Procurar em Coleção / Análises...";
                                        color: #626982;
                                        font_size: ((11px * s));
                                        vertical_alignment: center;
                                    }
                                }
                            }

                            Rectangle { horizontal_stretch: 1; }

                            // Status badge
                            Text {
                                text: root.status;
                                color: #00a2ff;
                                font_size: ((11px * s));
                                font_weight: 700;
                                vertical_alignment: center;
                            }
                        }
                    }

                    // Split view: Left Navigation Sidebar & Right Track Grid
                    HorizontalLayout {
                        spacing: ((0px * s));
                        vertical_stretch: 1;

                        // LEFT: NAVIGATION TREE SIDEBAR
                        Rectangle {
                            width: ((250px * s));
                            background: #0d0f15;
                            border_color: #1a1c27;
                            border_width: ((1px * s));

                            VerticalLayout {
                                padding: ((8px * s));
                                spacing: ((2px * s));
                                alignment: start;

                                // Coleção (Selected)
                                Rectangle {
                                    height: ((28px * s));
                                    background: rgb(0, 78, 138);
                                    border_radius: ((3px * s));
                                    HorizontalLayout {
                                        padding_left: ((8px * s));
                                        spacing: ((8px * s));
                                        Image { source: @image-url("../assets/ui/tree_track.svg"); width: ((16px * s)); height: ((16px * s)); vertical_alignment: center; }
                                        Text { text: "Coleção de Faixas"; color: #ffffff; font_size: ((12px * s)); font_weight: 800; vertical_alignment: center; }
                                    }
                                }

                                // Listas de reprodução
                                Rectangle {
                                    height: ((26px * s));
                                    HorizontalLayout {
                                        padding_left: ((8px * s));
                                        spacing: ((8px * s));
                                        Image { source: @image-url("../assets/ui/tree_minus.svg"); width: ((10px * s)); height: ((10px * s)); vertical_alignment: center; }
                                        Image { source: @image-url("../assets/ui/tree_folder.svg"); width: ((16px * s)); height: ((16px * s)); vertical_alignment: center; }
                                        Text { text: "Listas de Reprodução"; color: #a4abbf; font_size: ((12px * s)); vertical_alignment: center; }
                                    }
                                }
                                Rectangle {
                                    height: ((22px * s));
                                    HorizontalLayout {
                                        padding_left: ((36px * s));
                                        spacing: ((6px * s));
                                        Text { text: "★ Favoritas do Set"; color: rgb(136, 144, 176); font_size: ((11px * s)); vertical_alignment: center; }
                                    }
                                }
                                Rectangle {
                                    height: ((22px * s));
                                    HorizontalLayout {
                                        padding_left: ((36px * s));
                                        spacing: ((6px * s));
                                        Text { text: "⚡ Drum & Bass / Jungle"; color: rgb(136, 144, 176); font_size: ((11px * s)); vertical_alignment: center; }
                                    }
                                }
                                Rectangle {
                                    height: ((22px * s));
                                    HorizontalLayout {
                                        padding_left: ((36px * s));
                                        spacing: ((6px * s));
                                        Text { text: "🔥 Peak Time Techno"; color: rgb(136, 144, 176); font_size: ((11px * s)); vertical_alignment: center; }
                                    }
                                }

                                // Dispositivos (USB/SD)
                                Rectangle {
                                    height: ((26px * s));
                                    HorizontalLayout {
                                        padding_left: ((8px * s));
                                        spacing: ((8px * s));
                                        Image { source: @image-url("../assets/ui/tree_device.svg"); width: ((16px * s)); height: ((16px * s)); vertical_alignment: center; }
                                        Text { text: "Dispositivos USB / SD"; color: #a4abbf; font_size: ((12px * s)); vertical_alignment: center; }
                                    }
                                }
                                Rectangle {
                                    height: ((22px * s));
                                    HorizontalLayout {
                                        padding_left: ((36px * s));
                                        spacing: ((6px * s));
                                        Text { text: "💾 USB (FAT32)"; color: rgb(136, 144, 176); font_size: ((11px * s)); vertical_alignment: center; }
                                    }
                                }

                                // Histórico
                                Rectangle {
                                    height: ((26px * s));
                                    HorizontalLayout {
                                        padding_left: ((8px * s));
                                        spacing: ((8px * s));
                                        Image { source: @image-url("../assets/ui/tree_playlist.svg"); width: ((16px * s)); height: ((16px * s)); vertical_alignment: center; }
                                        Text { text: "Histórico de Sessões"; color: #a4abbf; font_size: ((12px * s)); vertical_alignment: center; }
                                    }
                                }
                            }
                        }

                        // RIGHT: TRACK TABLE LIST
                        VerticalLayout {
                            horizontal_stretch: 1;
                            spacing: ((0px * s));

                            // Table Header Row
                            Rectangle {
                                height: ((26px * s));
                                background: #131522;
                                border_color: rgb(30, 33, 45);
                                border_width: ((1px * s));

                                HorizontalLayout {
                                    padding_left: ((10px * s));
                                    padding_right: ((10px * s));
                                    spacing: ((8px * s));

                                    Text { width: ((36px * s)); text: "#"; color: #5f667a; font_size: ((10px * s)); font_weight: 800; vertical_alignment: center; }
                                    Text { horizontal_stretch: 3; text: "TÍTULO DA FAIXA"; color: #5f667a; font_size: ((10px * s)); font_weight: 800; vertical_alignment: center; }
                                    Text { horizontal_stretch: 2; text: "ARTISTA"; color: #5f667a; font_size: ((10px * s)); font_weight: 800; vertical_alignment: center; }
                                    Text { width: ((75px * s)); text: "BPM"; color: #5f667a; font_size: ((10px * s)); font_weight: 800; horizontal_alignment: right; vertical_alignment: center; }
                                    Text { width: ((45px * s)); text: "TOM"; color: #5f667a; font_size: ((10px * s)); font_weight: 800; horizontal_alignment: center; vertical_alignment: center; }
                                    Text { width: ((60px * s)); text: "TEMPO"; color: #5f667a; font_size: ((10px * s)); font_weight: 800; horizontal_alignment: right; vertical_alignment: center; }
                                    Text { width: ((85px * s)); text: "COMPASSOS"; color: #5f667a; font_size: ((10px * s)); font_weight: 800; horizontal_alignment: right; vertical_alignment: center; }
                                }
                            }

                            // Virtualized Track Rows
                            ListView {
                                vertical_stretch: 1;
                                min-height: ((0px * s));
                                for row[idx] in root.track_rows: Rectangle {
                                    height: ((28px * s));
                                    background: idx == root.selected_row ? rgb(0, 78, 138) : (row.alt ? #12141c : #0c0d13);

                                    // Left selection neon blue indicator
                                    if idx == root.selected_row: Rectangle {
                                        x: ((0px * s));
                                        width: ((3px * s));
                                        height: parent.height;
                                        background: #00a2ff;
                                    }

                                    HorizontalLayout {
                                        padding_left: ((10px * s));
                                        padding_right: ((10px * s));
                                        spacing: ((8px * s));

                                        Text {
                                            width: ((36px * s));
                                            text: row.id;
                                            color: idx == root.selected_row ? #ffffff : #585f73;
                                            font_size: ((11px * s));
                                            vertical_alignment: center;
                                        }
                                        Text {
                                            horizontal_stretch: 3;
                                            text: row.title;
                                            color: idx == root.selected_row ? #ffffff : #e1e4ed;
                                            font_size: ((12px * s));
                                            font_weight: idx == root.selected_row ? 700 : 400;
                                            overflow: elide;
                                            vertical_alignment: center;
                                        }
                                        Text {
                                            horizontal_stretch: 2;
                                            text: row.artist;
                                            color: idx == root.selected_row ? #cfe4ff : #82899d;
                                            font_size: ((11px * s));
                                            overflow: elide;
                                            vertical_alignment: center;
                                        }
                                        Text {
                                            width: ((75px * s));
                                            text: row.bpm;
                                            color: rgb(0, 229, 255);
                                            font_size: ((11px * s));
                                            font_weight: 700;
                                            horizontal_alignment: right;
                                            vertical_alignment: center;
                                        }
                                        Text {
                                            width: ((45px * s));
                                            text: row.key;
                                            color: #ff9800;
                                            font_size: ((11px * s));
                                            font_weight: 700;
                                            horizontal_alignment: center;
                                            vertical_alignment: center;
                                        }
                                        Text {
                                            width: ((60px * s));
                                            text: row.duration;
                                            color: #9aa1b5;
                                            font_size: ((11px * s));
                                            horizontal_alignment: right;
                                            vertical_alignment: center;
                                        }
                                        Text {
                                            width: ((85px * s));
                                            text: row.bars_count > 0 ? row.bars_count + " comp." : "—";
                                            color: #636b80;
                                            font_size: ((10px * s));
                                            horizontal_alignment: right;
                                            vertical_alignment: center;
                                        }
                                    }

                                    TouchArea {
                                        clicked => {
                                            root.track_selected(idx);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
                }
            }

            StatusBar { s: root.s; height: ((43px * s)); }

        }
    }
}

/// Pasta onde o rekordbox grava as análises (`.DAT`/`.EXT`). É um caminho de
/// dados de terceiros lido em modo somente leitura — interoperabilidade, não
/// marca: o nome aparece porque é onde os arquivos estão.
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

fn split_artist_title(raw: &str) -> (String, String) {
    let name = raw
        .strip_suffix(".mp3")
        .or_else(|| raw.strip_suffix(".flac"))
        .or_else(|| raw.strip_suffix(".wav"))
        .or_else(|| raw.strip_suffix(".aiff"))
        .or_else(|| raw.strip_suffix(".m4a"))
        .or_else(|| raw.strip_suffix(".MP3"))
        .or_else(|| raw.strip_suffix(".FLAC"))
        .unwrap_or(raw);

    let clean = if let Some(rest) = name.strip_prefix(|c: char| c.is_ascii_digit()) {
        let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        let rest = rest.trim_start_matches(['.', '-', ' ']);
        rest.trim()
    } else {
        name
    };

    if let Some((artist, title)) = clean.split_once(" - ") {
        (artist.trim().to_string(), title.trim().to_string())
    } else {
        ("Vilelabox / Coleção".to_string(), clean.trim().to_string())
    }
}

fn format_duration(grid: Option<&vilelabox_anlz::Grid>) -> String {
    let Some(grid) = grid else {
        return "—".into();
    };
    let Some(last) = grid.beats.last() else {
        return "—".into();
    };
    let beat_ms = if grid.bpm > 0.0 {
        60_000.0 / grid.bpm
    } else {
        0.0
    };
    let total_ms = last.time_ms as f64 + beat_ms;
    let mins = (total_ms / 60000.0) as u32;
    let secs = ((total_ms % 60000.0) / 1000.0) as u32;
    format!("{:02}:{:02}", mins, secs)
}

fn main() -> Result<(), slint::PlatformError> {
    let dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_dir);
    let tracks = Rc::new(library::scan(&dir));

    let ui = AppWindow::new()?;

    // Fator de escala único: 1.0 = geometria de referência (1917x1078).
    // VLBX_ESCALA permite reduzir o desenho inteiro (ex.: 0.62 para caber em 1180px).
    {
        let s: f32 = std::env::var("VLBX_ESCALA")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
            .unwrap_or(1.0)
            .clamp(0.35, 1.0);
        ui.set_s(s);
        eprintln!("escala {s} — geometria de referência 1917x1078");
    }

    let rows: Vec<TrackRow> = tracks
        .iter()
        .enumerate()
        .map(|(idx, t)| {
            let (artist, title) = split_artist_title(&t.title);
            let bpm = t
                .grid
                .as_ref()
                .map_or("—".into(), |g| format!("{:.2}", g.bpm));
            let duration = format_duration(t.grid.as_ref());
            let bars_count = t.grid.as_ref().map_or(0, |g| {
                g.beats.iter().filter(|b| b.beat_number == 1).count() as i32
            });

            TrackRow {
                id: (idx + 1) as i32,
                title: SharedString::from(title),
                artist: SharedString::from(artist),
                bpm: SharedString::from(bpm),
                key: SharedString::from("8A"),
                duration: SharedString::from(duration),
                bars_count,
                alt: idx % 2 == 1,
            }
        })
        .collect();

    ui.set_track_rows(ModelRc::new(VecModel::from(rows)));
    ui.set_status(SharedString::from(format!(
        "{} faixas em {}",
        tracks.len(),
        dir.display()
    )));

    // Load first track by default if available
    if let Some(first_track) = tracks.first() {
        let (artist, title) = split_artist_title(&first_track.title);
        let bpm = first_track
            .grid
            .as_ref()
            .map_or("—".into(), |g| format!("{:.2}", g.bpm));
        let duration = format_duration(first_track.grid.as_ref());
        let view = library::timeline(first_track);

        ui.set_selected_row(0);
        ui.set_sel_title(SharedString::from(title));
        ui.set_sel_artist(SharedString::from(artist));
        ui.set_sel_bpm(SharedString::from(bpm));
        ui.set_sel_duration(SharedString::from(duration));
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
    }

    let weak = ui.as_weak();
    let lib = tracks.clone();
    ui.on_track_selected(move |index| {
        let (Some(ui), Some(track)) = (
            weak.upgrade(),
            usize::try_from(index).ok().and_then(|i| lib.get(i)),
        ) else {
            return;
        };

        let (artist, title) = split_artist_title(&track.title);
        let bpm = track
            .grid
            .as_ref()
            .map_or("—".into(), |g| format!("{:.2}", g.bpm));
        let duration = format_duration(track.grid.as_ref());
        let view = library::timeline(track);

        ui.set_selected_row(index);
        ui.set_sel_title(SharedString::from(title));
        ui.set_sel_artist(SharedString::from(artist));
        ui.set_sel_bpm(SharedString::from(bpm));
        ui.set_sel_duration(SharedString::from(duration));
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
