//! Cheat panel: pure model (CheatPanel), menu content and "add" window.
//!
//! UI: the `Triches` menu lists one checkbox per cheat (the menu stays open on a click),
//! then "Ajouter une triche...", "Supprimer" and "Tout desactiver".

use eframe::egui;
use puce16_core::cheat::{parse_par, CheatEntry, ParCode, ParseError};

/// Splits a text on spaces, commas and line breaks and parses every code.
/// Errors are in French and name the faulty code (shown live in the add window).
pub fn check_codes(text: &str) -> Result<Vec<ParCode>, String> {
    let mut codes = Vec::new();
    for seg in text.split(|c: char| c.is_whitespace() || c == ',') {
        if seg.is_empty() {
            continue;
        }
        match parse_par(seg) {
            Ok(c) => codes.push(c),
            Err(e) => {
                let why = match e {
                    ParseError::Length(n) => format!("{} chiffres au lieu de 8", n),
                    ParseError::NotHex(c) => format!("caractere '{}' non hexadecimal", c),
                    ParseError::LooksLikeGameGenie => {
                        "code Game Genie non gere (Pro Action Replay brut seulement)".to_string()
                    }
                };
                return Err(format!("code '{}': {}", seg, why));
            }
        }
    }
    if codes.is_empty() {
        return Err("saisir au moins un code".to_string());
    }
    Ok(codes)
}

/// Pure cheat panel model (no egui).
pub struct CheatPanel {
    entries: Vec<CheatEntry>,
    pub add_open: bool,
    pub codes_input: String,
    pub name_input: String,
    /// Give the keyboard focus to the codes field on the next frame of the add window.
    focus_codes: bool,
}

impl CheatPanel {
    /// Create a new panel from loaded cheat entries.
    pub fn new(entries: Vec<CheatEntry>) -> CheatPanel {
        CheatPanel {
            entries,
            add_open: false,
            codes_input: String::new(),
            name_input: String::new(),
            focus_codes: false,
        }
    }

    /// Return a slice of the cheat entries.
    pub fn entries(&self) -> &[CheatEntry] {
        &self.entries
    }

    /// Open the add window (empty fields, focus on the codes field).
    pub fn open_add(&mut self) {
        self.add_open = true;
        self.focus_codes = true;
        self.codes_input.clear();
        self.name_input.clear();
    }

    /// Add a new cheat entry (always enabled). An empty name is replaced by the first code.
    pub fn add(&mut self, name: &str, codes_text: &str) -> Result<(), String> {
        let codes = check_codes(codes_text)?;
        let mut name = name.trim().to_string();
        if name.contains('=') || name.contains('\n') || name.contains('\r') {
            return Err("nom invalide".to_string());
        }
        if name.is_empty() {
            name = codes[0].to_string();
        }
        self.entries.push(CheatEntry { name, enabled: true, codes });
        Ok(())
    }

    /// Toggle the enabled state of entry at index i. No effect if out of range.
    pub fn toggle(&mut self, i: usize) {
        if let Some(entry) = self.entries.get_mut(i) {
            entry.enabled = !entry.enabled;
        }
    }

    /// Remove the entry at index i. No effect if out of range.
    pub fn remove(&mut self, i: usize) {
        if i < self.entries.len() {
            self.entries.remove(i);
        }
    }

    /// Disable every entry (they stay in the list).
    pub fn disable_all(&mut self) {
        for e in &mut self.entries {
            e.enabled = false;
        }
    }

    /// Return all active (enabled) codes in order across all entries.
    pub fn active_codes(&self) -> Vec<ParCode> {
        let mut result = Vec::new();
        for entry in &self.entries {
            if entry.enabled {
                result.extend_from_slice(&entry.codes);
            }
        }
        result
    }

    /// Content of the `Triches` menu. Returns true if the list changed.
    pub fn menu_ui(&mut self, ui: &mut egui::Ui, rom_loaded: bool) -> bool {
        let mut changed = false;
        ui.add_enabled_ui(rom_loaded, |ui| {
            if self.entries.is_empty() {
                ui.add_enabled(false, egui::Label::new("Aucune triche pour ce jeu"));
            } else {
                let mut toggles = Vec::new();
                for (i, entry) in self.entries.iter().enumerate() {
                    let mut on = entry.enabled;
                    let codes: Vec<String> = entry.codes.iter().map(|c| c.to_string()).collect();
                    if ui
                        .checkbox(&mut on, entry.name.as_str())
                        .on_hover_text(codes.join(" "))
                        .changed()
                    {
                        toggles.push(i);
                    }
                }
                for i in toggles {
                    self.toggle(i);
                    changed = true;
                }
            }
            ui.separator();
            if ui.button("Ajouter une triche...").clicked() {
                self.open_add();
                ui.close();
            }
            ui.add_enabled_ui(!self.entries.is_empty(), |ui| {
                ui.menu_button("Supprimer", |ui| {
                    let mut del = None;
                    for (i, entry) in self.entries.iter().enumerate() {
                        if ui.button(entry.name.as_str()).clicked() {
                            del = Some(i);
                        }
                    }
                    if let Some(i) = del {
                        self.remove(i);
                        changed = true;
                        ui.close();
                    }
                });
                if ui.button("Tout desactiver").clicked() {
                    self.disable_all();
                    changed = true;
                }
            });
        });
        changed
    }

    /// Small "add a cheat" window. Returns true if a cheat was added this frame.
    pub fn add_window(&mut self, ctx: &egui::Context) -> bool {
        if !self.add_open {
            return false;
        }
        let mut open = true;
        let mut submit = false;
        let mut cancel = false;
        egui::Window::new("Ajouter une triche")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Code(s) Pro Action Replay (8 chiffres, ex: 7E0DBE05). Plusieurs codes: separes par espace, virgule ou ligne.");
                let r = ui.add(
                    egui::TextEdit::multiline(&mut self.codes_input)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                if self.focus_codes {
                    r.request_focus();
                    self.focus_codes = false;
                }
                let check = check_codes(&self.codes_input);
                if !self.codes_input.trim().is_empty() {
                    if let Err(e) = &check {
                        ui.colored_label(egui::Color32::from_rgb(220, 60, 60), e.as_str());
                    }
                }
                ui.horizontal(|ui| {
                    ui.label("Nom (facultatif):");
                    ui.text_edit_singleline(&mut self.name_input);
                });
                ui.horizontal(|ui| {
                    if ui.add_enabled(check.is_ok(), egui::Button::new("Ajouter")).clicked() {
                        submit = true;
                    }
                    if ui.button("Annuler").clicked() {
                        cancel = true;
                    }
                });
                let (enter, esc) = ui.input(|i| {
                    (
                        i.key_pressed(egui::Key::Enter) && !i.modifiers.shift,
                        i.key_pressed(egui::Key::Escape),
                    )
                });
                if enter && check.is_ok() {
                    submit = true;
                }
                if esc {
                    cancel = true;
                }
            });
        let mut added = false;
        if submit {
            let (name, codes) = (self.name_input.clone(), self.codes_input.clone());
            if self.add(&name, &codes).is_ok() {
                added = true;
                self.add_open = false;
            }
        }
        if cancel || !open {
            self.add_open = false;
        }
        added
    }
}
