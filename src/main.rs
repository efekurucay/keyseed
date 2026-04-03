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

const MAX_CONTENT_WIDTH: f32 = 760.0;
const COMPACT_BREAKPOINT: f32 = 680.0;
const REGULAR_EDITOR_HEIGHT: f32 = 170.0;
const COMPACT_EDITOR_HEIGHT: f32 = 120.0;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Hashit")
            .with_inner_size([720.0, 520.0])
            .with_min_inner_size([380.0, 420.0]),
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
                    .fill(Color32::from_rgb(246, 248, 251))
                    .inner_margin(Margin::same(14)),
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
                                ui.add_space(8.0);
                                self.master_key_section(ui, compact);
                                ui.add_space(8.0);
                                self.content_section(ui, compact, editor_height);
                                ui.add_space(8.0);
                                self.actions_section(ui, compact);
                                ui.add_space(8.0);
                                self.status_section(ui);
                            });
                        });
                    });
            });
    }
}

impl HashitApp {
    fn header(&self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Hashit")
                        .size(21.0)
                        .strong()
                        .color(Color32::from_rgb(16, 24, 40)),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "Encrypt and decrypt secrets locally with a single master key.",
                    )
                    .size(11.5)
                    .color(Color32::from_rgb(71, 84, 103)),
                );
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    info_chip(ui, "Local only");
                    info_chip(ui, "AES-256-GCM");
                    info_chip(ui, "PBKDF2-SHA256");
                    info_chip(ui, "hashit:v2");
                });
            });
        });
    }

    fn master_key_section(&mut self, ui: &mut egui::Ui, compact: bool) {
        section_frame().show(ui, |ui| {
            section_title(ui, "Master key", "Only kept in memory for the current session.");
            ui.add_space(8.0);

            if compact {
                ui.vertical(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.master_key)
                            .password(!self.show_master_key)
                            .hint_text("Enter your master key")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(6.0);
                    let toggle_label = if self.show_master_key { "Hide" } else { "Show" };
                    if ui
                        .add(subtle_button(toggle_label).min_size(Vec2::new(ui.available_width(), 30.0)))
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
                        .add(subtle_button(toggle_label).min_size(Vec2::new(72.0, 30.0)))
                        .clicked()
                    {
                        self.show_master_key = !self.show_master_key;
                    }
                });
            }

            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Use a strong, memorable passphrase. Hashit does not store your master key on disk.",
                )
                .size(10.5)
                .color(Color32::from_rgb(102, 112, 133)),
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
                "Plain text",
                "Paste any password, note, or secret you want to protect.",
                &mut self.plain_text,
                plain_copied,
                "Copy text",
                Color32::from_rgb(52, 64, 84),
                editor_height,
            );

            ui.add_space(8.0);

            copy_encrypted = render_text_panel(
                ui,
                "Encrypted output",
                "Store this Hashit payload anywhere. You will need the same master key to decrypt it.",
                &mut self.encrypted_text,
                encrypted_copied,
                "Copy output",
                Color32::from_rgb(12, 74, 110),
                editor_height,
            );
        } else {
            ui.columns(2, |columns| {
                copy_plain = render_text_panel(
                    &mut columns[0],
                    "Plain text",
                    "Paste any password, note, or secret you want to protect.",
                    &mut self.plain_text,
                    plain_copied,
                    "Copy text",
                    Color32::from_rgb(52, 64, 84),
                    editor_height,
                );

                copy_encrypted = render_text_panel(
                    &mut columns[1],
                    "Encrypted output",
                    "Store this Hashit payload anywhere. You will need the same master key to decrypt it.",
                    &mut self.encrypted_text,
                    encrypted_copied,
                    "Copy output",
                    Color32::from_rgb(12, 74, 110),
                    editor_height,
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
        if compact {
            if action_button(ui, "Encrypt", Color32::from_rgb(17, 94, 89), true) {
                self.encrypt_action();
            }
            ui.add_space(6.0);
            if action_button(ui, "Decrypt", Color32::from_rgb(29, 78, 216), true) {
                self.decrypt_action();
            }
            ui.add_space(6.0);
            if action_button(ui, "Clear", Color32::from_rgb(71, 84, 103), false) {
                self.clear_all();
            }
        } else {
            ui.columns(3, |columns| {
                if action_button(&mut columns[0], "Encrypt", Color32::from_rgb(17, 94, 89), true) {
                    self.encrypt_action();
                }
                if action_button(&mut columns[1], "Decrypt", Color32::from_rgb(29, 78, 216), true) {
                    self.decrypt_action();
                }
                if action_button(&mut columns[2], "Clear", Color32::from_rgb(71, 84, 103), false) {
                    self.clear_all();
                }
            });
        }
    }

    fn status_section(&self, ui: &mut egui::Ui) {
        if self.status_message.is_empty() {
            return;
        }

        let (bg, fg, border, label) = if self.is_error {
            (
                Color32::from_rgb(254, 242, 242),
                Color32::from_rgb(185, 28, 28),
                Color32::from_rgb(254, 205, 211),
                "Error",
            )
        } else {
            (
                Color32::from_rgb(240, 253, 250),
                Color32::from_rgb(15, 118, 110),
                Color32::from_rgb(153, 246, 228),
                "Ready",
            )
        };

        Frame::new()
            .fill(bg)
            .stroke(Stroke::new(1.0, border))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(10, 8))
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
                self.set_status("Encryption complete.", false);
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

fn configure_theme(ctx: &egui::Context) {
    ctx.set_zoom_factor(0.88);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 6.0);
    style.spacing.text_edit_width = 200.0;
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(Color32::from_rgb(16, 24, 40));
    style.visuals.window_corner_radius = CornerRadius::same(18);
    style.visuals.panel_fill = Color32::from_rgb(246, 248, 251);
    style.visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
    style.visuals.faint_bg_color = Color32::from_rgb(248, 250, 252);
    style.visuals.code_bg_color = Color32::from_rgb(248, 250, 252);
    style.visuals.selection.bg_fill = Color32::from_rgb(191, 219, 254);
    style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(29, 78, 216));
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(226, 232, 240));
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(255, 255, 255);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(203, 213, 225));
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(248, 250, 252);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(148, 163, 184));
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(241, 245, 249);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(100, 116, 139));
    style.visuals.widgets.open.bg_fill = Color32::from_rgb(248, 250, 252);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(12);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(12);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(12);
    style.visuals.widgets.open.corner_radius = CornerRadius::same(12);
    ctx.set_style(style);
}

fn render_text_panel(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    value: &mut String,
    copied: bool,
    copy_label: &str,
    accent: Color32,
    editor_height: f32,
) -> bool {
    let mut copy_clicked = false;

    section_frame().show(ui, |ui| {
        section_title(ui, title, description);
        ui.add_space(6.0);
        ui.add_sized(
            [ui.available_width(), editor_height],
            TextEdit::multiline(value)
                .desired_width(f32::INFINITY)
                .font(TextStyle::Monospace)
                .frame(true),
        );
        ui.add_space(8.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = if copied { "Copied" } else { copy_label };
            if ui
                .add_enabled(
                    !value.is_empty(),
                    Button::new(RichText::new(label).size(11.5).strong().color(Color32::WHITE))
                        .fill(accent)
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(8))
                        .min_size(Vec2::new(92.0, 28.0)),
                )
                .clicked()
            {
                copy_clicked = true;
            }
        });
    });

    copy_clicked
}

fn action_button(ui: &mut egui::Ui, label: &str, color: Color32, strong: bool) -> bool {
    let text = if strong {
        RichText::new(label).size(12.0).strong().color(Color32::WHITE)
    } else {
        RichText::new(label).size(12.0).strong().color(Color32::from_rgb(248, 250, 252))
    };

    ui.add_sized(
        [ui.available_width(), 32.0],
        Button::new(text)
            .fill(color)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(10)),
    )
    .clicked()
}

fn info_chip(ui: &mut egui::Ui, text: &str) {
    Frame::new()
        .fill(Color32::from_rgb(248, 250, 252))
        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
        .corner_radius(CornerRadius::same(24))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(10.0)
                    .color(Color32::from_rgb(71, 84, 103)),
            );
        });
}

fn subtle_button(label: &str) -> Button<'_> {
    Button::new(RichText::new(label).size(11.5).strong().color(Color32::from_rgb(52, 64, 84)))
        .fill(Color32::from_rgb(248, 250, 252))
        .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
        .corner_radius(CornerRadius::same(8))
}

fn card_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_rgb(255, 255, 255))
        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(12))
}

fn section_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_rgb(255, 255, 255))
        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(12))
}

fn section_title(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.label(
        RichText::new(title)
            .size(13.0)
            .strong()
            .color(Color32::from_rgb(16, 24, 40)),
    );
    ui.add_space(4.0);
    ui.label(
        RichText::new(description)
            .size(10.5)
            .color(Color32::from_rgb(102, 112, 133)),
    );
}
