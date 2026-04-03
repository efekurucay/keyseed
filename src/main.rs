mod crypto;

use std::time::{Duration, Instant};

use crypto::EncryptedPayload;
use eframe::{
    egui::{
        self, Align, Button, Color32, CornerRadius, Frame, Layout, Margin, RichText, ScrollArea,
        Stroke, TextEdit, TextStyle, Vec2,
    },
    App, NativeOptions,
};

const MAX_CONTENT_WIDTH: f32 = 820.0;
const COMPACT_BREAKPOINT: f32 = 700.0;
const REGULAR_EDITOR_HEIGHT: f32 = 150.0;
const COMPACT_EDITOR_HEIGHT: f32 = 112.0;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Hashit")
            .with_inner_size([820.0, 540.0])
            .with_min_inner_size([420.0, 460.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Hashit",
        native_options,
        Box::new(|cc| {
            configure_theme(&cc.egui_ctx);
            Ok(Box::new(HashitApp::default()))
        }),
    )
}

#[derive(Default)]
struct HashitApp {
    plain_text: String,
    encrypted_text: String,
    master_key: String,
    show_master_key: bool,
    status_message: String,
    is_error: bool,
    copied_field: Option<CopiedState>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CopiedField {
    Plain,
    Encrypted,
}

struct CopiedState {
    field: CopiedField,
    at: Instant,
}

impl App for HashitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_copy_state(ctx);

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(Color32::from_rgb(242, 244, 248))
                    .inner_margin(Margin::same(10)),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let content_width = ui.available_width().min(MAX_CONTENT_WIDTH);

                        ui.horizontal_centered(|ui| {
                            ui.set_width(content_width);

                            let compact = content_width < COMPACT_BREAKPOINT;
                            let editor_height = if compact {
                                COMPACT_EDITOR_HEIGHT
                            } else {
                                REGULAR_EDITOR_HEIGHT
                            };

                            ui.vertical(|ui| {
                                self.header(ui);
                                ui.add_space(12.0);
                                self.master_key_section(ui, compact);
                                ui.add_space(12.0);
                                self.content_section(ui, compact, editor_height);
                                ui.add_space(12.0);
                                self.actions_section(ui, compact);
                                ui.add_space(12.0);
                                self.status_section(ui);
                            });
                        });
                    });
            });
    }
}

impl HashitApp {
    fn header(&self, ui: &mut egui::Ui) {
        hero_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                traffic_lights(ui);
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Hashit")
                        .size(15.0)
                        .strong()
                        .color(Color32::from_rgb(24, 28, 37)),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Deterministic local encryption")
                        .size(10.5)
                        .color(Color32::from_rgb(104, 111, 124)),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    feature_pill(ui, "hashit:v2");
                    feature_pill(ui, "PBKDF2-SHA256");
                    feature_pill(ui, "AES-256-GCM-SIV");
                });
            });
        });
    }

    fn master_key_section(&mut self, ui: &mut egui::Ui, compact: bool) {
        section_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Master key")
                            .size(13.5)
                            .strong()
                            .color(Color32::from_rgb(28, 32, 41)),
                    );
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new(
                            "Used only for the current session. Nothing is saved to disk.",
                        )
                        .size(10.5)
                        .color(Color32::from_rgb(108, 115, 128)),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    soft_chip(
                        ui,
                        if self.show_master_key { "Visible" } else { "Hidden" },
                        Color32::from_rgb(245, 247, 250),
                        Color32::from_rgb(94, 103, 120),
                        Color32::from_rgb(223, 227, 235),
                    );
                });
            });

            ui.add_space(12.0);

            if compact {
                ui.vertical(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.master_key)
                            .password(!self.show_master_key)
                            .hint_text("Enter your master key")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(8.0);
                    let toggle_label = if self.show_master_key {
                        "Hide master key"
                    } else {
                        "Show master key"
                    };
                    if ui
                        .add(secondary_button(toggle_label).min_size(Vec2::new(ui.available_width(), 30.0)))
                        .clicked()
                    {
                        self.show_master_key = !self.show_master_key;
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.master_key)
                            .password(!self.show_master_key)
                            .hint_text("Enter your master key")
                            .desired_width(f32::INFINITY),
                    );

                    let toggle_label = if self.show_master_key { "Hide" } else { "Show" };
                    if ui
                        .add(secondary_button(toggle_label).min_size(Vec2::new(78.0, 30.0)))
                        .clicked()
                    {
                        self.show_master_key = !self.show_master_key;
                    }
                });
            }

            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Tip: keep labels stable, like efe.facebook or efe.instagram.",
                )
                .size(9.8)
                .color(Color32::from_rgb(112, 118, 130)),
            );
        });
    }

    fn content_section(&mut self, ui: &mut egui::Ui, compact: bool, editor_height: f32) {
        let plain_copied = self
            .copied_field
            .as_ref()
            .is_some_and(|state| state.field == CopiedField::Plain);
        let encrypted_copied = self
            .copied_field
            .as_ref()
            .is_some_and(|state| state.field == CopiedField::Encrypted);

        let mut copy_plain = false;
        let mut copy_encrypted = false;

        if compact {
            copy_plain = render_text_panel(
                ui,
                "Input",
                "Type a stable label like efe.facebook. With the same master key, Encrypt always gives the same result.",
                &mut self.plain_text,
                plain_copied,
                "Copy input",
                editor_height,
                PanelTone::Input,
            );

            ui.add_space(12.0);

            copy_encrypted = render_text_panel(
                ui,
                "Output",
                "Your deterministic Hashit payload appears here. You can copy it, store it, and decrypt it later with the same master key.",
                &mut self.encrypted_text,
                encrypted_copied,
                "Copy output",
                editor_height,
                PanelTone::Output,
            );
        } else {
            ui.columns(2, |columns| {
                copy_plain = render_text_panel(
                    &mut columns[0],
                    "Input",
                    "Type a stable label like efe.facebook. With the same master key, Encrypt always gives the same result.",
                    &mut self.plain_text,
                    plain_copied,
                    "Copy input",
                    editor_height,
                    PanelTone::Input,
                );

                copy_encrypted = render_text_panel(
                    &mut columns[1],
                    "Output",
                    "Your deterministic Hashit payload appears here. You can copy it, store it, and decrypt it later with the same master key.",
                    &mut self.encrypted_text,
                    encrypted_copied,
                    "Copy output",
                    editor_height,
                    PanelTone::Output,
                );
            });
        }

        if copy_plain {
            self.copy_to_clipboard(ui.ctx(), CopiedField::Plain, self.plain_text.clone());
        }

        if copy_encrypted {
            self.copy_to_clipboard(
                ui.ctx(),
                CopiedField::Encrypted,
                self.encrypted_text.clone(),
            );
        }
    }

    fn actions_section(&mut self, ui: &mut egui::Ui, compact: bool) {
        section_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Actions")
                            .size(13.5)
                            .strong()
                            .color(Color32::from_rgb(28, 32, 41)),
                    );
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new(
                            "Encrypt to generate the deterministic payload. Decrypt to restore the original label or secret.",
                        )
                        .size(10.5)
                        .color(Color32::from_rgb(108, 115, 128)),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    soft_chip(
                        ui,
                        "Deterministic",
                        Color32::from_rgb(238, 245, 255),
                        Color32::from_rgb(41, 98, 255),
                        Color32::from_rgb(202, 220, 255),
                    );
                });
            });

            ui.add_space(14.0);

            if compact {
                if primary_button(ui, "Encrypt") {
                    self.encrypt_action();
                }
                ui.add_space(8.0);
                if secondary_action_button(ui, "Decrypt") {
                    self.decrypt_action();
                }
                ui.add_space(8.0);
                if tertiary_action_button(ui, "Clear") {
                    self.clear_all();
                }
            } else {
                ui.columns(3, |columns| {
                    if primary_button(&mut columns[0], "Encrypt") {
                        self.encrypt_action();
                    }
                    if secondary_action_button(&mut columns[1], "Decrypt") {
                        self.decrypt_action();
                    }
                    if tertiary_action_button(&mut columns[2], "Clear") {
                        self.clear_all();
                    }
                });
            }
        });
    }

    fn status_section(&self, ui: &mut egui::Ui) {
        if self.status_message.is_empty() {
            return;
        }

        let (bg, fg, border, label) = if self.is_error {
            (
                Color32::from_rgb(255, 243, 243),
                Color32::from_rgb(188, 44, 44),
                Color32::from_rgb(245, 204, 204),
                "Problem",
            )
        } else {
            (
                Color32::from_rgb(241, 248, 244),
                Color32::from_rgb(42, 120, 75),
                Color32::from_rgb(202, 229, 213),
                "Ready",
            )
        };

        Frame::new()
            .fill(bg)
            .stroke(Stroke::new(1.0, border))
            .corner_radius(CornerRadius::same(16))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(label).strong().color(fg));
                    ui.label(RichText::new("•").color(border));
                    ui.label(RichText::new(&self.status_message).color(fg));
                });
            });
    }

    fn encrypt_action(&mut self) {
        match EncryptedPayload::encrypt(&self.plain_text, &self.master_key) {
            Ok(encrypted) => {
                self.encrypted_text = encrypted;
                self.set_status("Deterministic encryption complete.", false);
            }
            Err(error) => self.set_status(&error.to_string(), true),
        }
    }

    fn decrypt_action(&mut self) {
        match EncryptedPayload::decrypt(self.encrypted_text.trim(), &self.master_key) {
            Ok(plain) => {
                self.plain_text = plain;
                self.set_status("Decryption complete.", false);
            }
            Err(error) => self.set_status(&error.to_string(), true),
        }
    }

    fn clear_all(&mut self) {
        self.plain_text.clear();
        self.encrypted_text.clear();
        self.master_key.clear();
        self.copied_field = None;
        self.set_status("All fields were cleared.", false);
    }

    fn copy_to_clipboard(&mut self, ctx: &egui::Context, field: CopiedField, value: String) {
        ctx.copy_text(value);
        self.copied_field = Some(CopiedState {
            field,
            at: Instant::now(),
        });
        self.set_status("Copied to clipboard.", false);
    }

    fn set_status(&mut self, message: &str, is_error: bool) {
        self.status_message = message.to_owned();
        self.is_error = is_error;
    }

    fn refresh_copy_state(&mut self, ctx: &egui::Context) {
        if let Some(state) = &self.copied_field {
            if state.at.elapsed() >= Duration::from_millis(1800) {
                self.copied_field = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
    }
}

#[derive(Clone, Copy)]
enum PanelTone {
    Input,
    Output,
}

fn configure_theme(ctx: &egui::Context) {
    ctx.set_zoom_factor(0.86);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 6.0);
    style.spacing.text_edit_width = 180.0;
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(Color32::from_rgb(29, 33, 42));
    style.visuals.window_corner_radius = CornerRadius::same(24);
    style.visuals.panel_fill = Color32::from_rgb(242, 244, 248);
    style.visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
    style.visuals.faint_bg_color = Color32::from_rgb(247, 248, 251);
    style.visuals.code_bg_color = Color32::from_rgb(247, 248, 251);
    style.visuals.selection.bg_fill = Color32::from_rgb(210, 224, 255);
    style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(41, 98, 255));
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(228, 231, 237));
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(255, 255, 255);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 224, 232));
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(250, 251, 253);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(190, 197, 210));
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(244, 246, 249);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(165, 174, 190));
    style.visuals.widgets.open.bg_fill = Color32::from_rgb(248, 249, 252);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.open.corner_radius = CornerRadius::same(16);
    ctx.set_style(style);
}

fn render_text_panel(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    value: &mut String,
    copied: bool,
    copy_label: &str,
    editor_height: f32,
    tone: PanelTone,
) -> bool {
    let mut copy_clicked = false;

    let (chip_fill, chip_text, chip_border, chip_label) = match tone {
        PanelTone::Input => (
            Color32::from_rgb(243, 245, 248),
            Color32::from_rgb(86, 96, 112),
            Color32::from_rgb(224, 228, 235),
            "Source",
        ),
        PanelTone::Output => (
            Color32::from_rgb(238, 245, 255),
            Color32::from_rgb(41, 98, 255),
            Color32::from_rgb(202, 220, 255),
            "Deterministic",
        ),
    };

    section_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(title)
                        .size(15.0)
                        .strong()
                        .color(Color32::from_rgb(28, 32, 41)),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(description)
                        .size(11.5)
                        .color(Color32::from_rgb(108, 115, 128)),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                soft_chip(ui, chip_label, chip_fill, chip_text, chip_border);
            });
        });

        ui.add_space(12.0);
        ui.add_sized(
            [ui.available_width(), editor_height],
            TextEdit::multiline(value)
                .desired_width(f32::INFINITY)
                .font(TextStyle::Monospace)
                .frame(true),
        );
        ui.add_space(10.0);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = if copied { "Copied" } else { copy_label };
            if ui
                .add_enabled(
                    !value.is_empty(),
                    secondary_button(label).min_size(Vec2::new(108.0, 32.0)),
                )
                .clicked()
            {
                copy_clicked = true;
            }
        });
    });

    copy_clicked
}

fn primary_button(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add_sized(
        [ui.available_width(), 38.0],
        Button::new(RichText::new(label).size(12.5).strong().color(Color32::WHITE))
            .fill(Color32::from_rgb(34, 99, 255))
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(14)),
    )
    .clicked()
}

fn secondary_action_button(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add_sized(
        [ui.available_width(), 38.0],
        Button::new(
            RichText::new(label)
                .size(12.5)
                .strong()
                .color(Color32::from_rgb(46, 56, 72)),
        )
        .fill(Color32::from_rgb(248, 249, 252))
        .stroke(Stroke::new(1.0, Color32::from_rgb(220, 224, 232)))
        .corner_radius(CornerRadius::same(14)),
    )
    .clicked()
}

fn tertiary_action_button(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add_sized(
        [ui.available_width(), 38.0],
        Button::new(
            RichText::new(label)
                .size(12.5)
                .strong()
                .color(Color32::from_rgb(120, 123, 132)),
        )
        .fill(Color32::from_rgb(244, 246, 249))
        .stroke(Stroke::new(1.0, Color32::from_rgb(228, 231, 237)))
        .corner_radius(CornerRadius::same(14)),
    )
    .clicked()
}

fn secondary_button(label: &str) -> Button<'_> {
    Button::new(
        RichText::new(label)
            .size(11.5)
            .strong()
            .color(Color32::from_rgb(52, 61, 76)),
    )
    .fill(Color32::from_rgb(248, 249, 252))
    .stroke(Stroke::new(1.0, Color32::from_rgb(220, 224, 232)))
    .corner_radius(CornerRadius::same(12))
}

fn feature_pill(ui: &mut egui::Ui, text: &str) {
    soft_chip(
        ui,
        text,
        Color32::from_rgb(250, 251, 253),
        Color32::from_rgb(87, 95, 109),
        Color32::from_rgb(225, 229, 236),
    );
}

fn soft_chip(
    ui: &mut egui::Ui,
    text: &str,
    fill: Color32,
    text_color: Color32,
    border: Color32,
) {
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(99))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(10.5).color(text_color));
        });
}

fn traffic_lights(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("●").size(12.0).color(Color32::from_rgb(255, 95, 86)));
        ui.label(RichText::new("●").size(12.0).color(Color32::from_rgb(255, 189, 46)));
        ui.label(RichText::new("●").size(12.0).color(Color32::from_rgb(39, 201, 63)));
    });
}

fn hero_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_rgb(252, 252, 253))
        .stroke(Stroke::new(1.0, Color32::from_rgb(225, 229, 236)))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::symmetric(12, 8))
}

fn section_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_rgb(252, 252, 253))
        .stroke(Stroke::new(1.0, Color32::from_rgb(228, 231, 237)))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(12))
}
