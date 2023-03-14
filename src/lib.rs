mod cursor;
mod input;
mod savefiles;

use std::collections::HashMap;
use std::fmt::Display;

use std::time::{Duration, Instant};

use anyhow::Result;
use eframe::CreationContext;
use egui::{CentralPanel, ComboBox, Context, ScrollArea, TopBottomPanel};

use cursor::*;
use input::{is_pressed_load, is_pressed_next, is_pressed_prev, is_pressed_save};
use notify_rust::Notification;
use savefiles::{Savefile, SavefilePath};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Game {
    DarkSoulsIII,
    EldenRing,
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Game::EldenRing => "Elden Ring",
                Game::DarkSoulsIII => "Dark Souls III",
            }
        )
    }
}

struct SelfDestructible<T> {
    val: T,
    instant: Instant,
    lifetime: Duration,
}

impl<T> SelfDestructible<T> {
    fn new(val: T, lifetime: Duration) -> Self {
        Self {
            val,
            lifetime,
            instant: Instant::now(),
        }
    }

    fn get(&self) -> Option<&T> {
        if self.instant.elapsed() < self.lifetime {
            Some(&self.val)
        } else {
            None
        }
    }
}

pub struct App {
    savefile_paths: Cursor<SavefilePath>,
    savefiles: HashMap<Game, Cursor<Savefile>>,
    message: Option<SelfDestructible<String>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            savefile_paths: Cursor::new(SavefilePath::get_all().unwrap()),
            savefiles: Default::default(),
            message: None,
        }
    }
}

impl App {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Default::default()
    }

    fn current_savefile_path(&self) -> &SavefilePath {
        self.savefile_paths.get()
    }

    fn current_savefiles(&self) -> Option<&Cursor<Savefile>> {
        self.savefiles.get(&self.current_savefile_path().game)
    }

    fn current_savefiles_mut(&mut self) -> &mut Cursor<Savefile> {
        self.savefiles
            .entry(self.current_savefile_path().game)
            .or_insert_with(Cursor::default)
    }

    fn next_slot(&mut self) {
        self.current_savefiles_mut().next();
    }

    fn prev_slot(&mut self) {
        self.current_savefiles_mut().prev();
    }

    fn goto_last(&mut self) {
        let s = self.current_savefiles_mut();
        s.goto(s.len() - 1);
    }

    fn save(&mut self) -> Result<()> {
        let new_savefile = Savefile::new(&self.current_savefile_path().path)?;
        self.current_savefiles_mut().push(new_savefile);
        self.message = Some(SelfDestructible::new(
            "Saved".to_string(),
            Duration::from_secs(5),
        ));
        self.goto_last();
        Ok(())
    }

    fn show_notif(idx: usize) -> Result<()> {
        Notification::new()
            .summary("Zombie Run")
            .body(&format!("Loaded savefile #{idx}"))
            .timeout(Duration::from_secs(1))
            .show()?;

        Ok(())
    }

    fn load(&mut self) -> Result<()> {
        if let Some(savefiles) = self.current_savefiles() {
            let idx = savefiles.index();
            if savefiles.len() == 0 {
                return Ok(());
            }
            savefiles.get().load(&self.current_savefile_path().path)?;
            self.message = Some(SelfDestructible::new(
                format!(
                    "Loaded #{:03} ({})",
                    savefiles.index(),
                    self.current_savefile_path().game
                ),
                Duration::from_secs(5),
            ));
            App::show_notif(idx)?;
        }

        Ok(())
    }

    fn load_at(&mut self, idx: usize) -> Result<()> {
        if let Some(savefile) = self
            .current_savefiles()
            .and_then(|savefiles| savefiles.get_at(idx))
        {
            savefile.load(&self.current_savefile_path().path)?;
            self.message = Some(SelfDestructible::new(
                format!("Loaded #{:03} ({})", idx, self.current_savefile_path().game),
                Duration::from_secs(5),
            ));

            App::show_notif(idx)?;
        }
        Ok(())
    }

    fn update_inputs(&mut self) -> Result<()> {
        if is_pressed_next() {
            self.next_slot();
        } else if is_pressed_prev() {
            self.prev_slot();
        } else if is_pressed_save() {
            self.save()?;
        } else if is_pressed_load() {
            self.load()?;
        }

        Ok(())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_inputs().ok();
        ctx.request_repaint_after(Duration::from_millis(120));

        TopBottomPanel::top("savefile_path").show(ctx, |ui| {
            ComboBox::from_label("Savefile")
                .selected_text(self.savefile_paths.get().to_string())
                .show_ui(ui, |ui| {
                    let mut idx = self.savefile_paths.index();
                    for i in 0..self.savefile_paths.len() {
                        let savefile = self.savefile_paths.get_at(i).unwrap();
                        ui.selectable_value(&mut idx, i, savefile.to_string());
                    }
                    self.savefile_paths.goto(idx);
                });
        });

        TopBottomPanel::bottom("buttons").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Add savefile (Shift + N)").clicked() {
                    self.save().ok();
                }
                if ui.button("Load savefile (Shift + L)").clicked() {
                    self.load().ok();
                }

                if let Some(message) = self.message.as_ref() {
                    if let Some(message) = message.get() {
                        ui.label(message);
                    }
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let mut sel_index =
                            self.current_savefiles().map(|s| s.index()).unwrap_or(0);
                        let mut to_remove: Option<usize> = None;
                        let mut to_load: Option<usize> = None;

                        for (idx, savefile) in
                            self.current_savefiles_mut().data().iter().enumerate().rev()
                        {
                            let elapsed = savefile.saved().elapsed().as_secs_f64();
                            let hours = (elapsed / 3600.).floor();
                            let minutes = ((elapsed / 60.) % 60.).floor();
                            let seconds = (elapsed % 60.).floor();
                            let uid = savefile.uid;
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut sel_index,
                                    idx,
                                    format!(
                                "#[{idx:02}] [{uid:03}] - {hours:02}:{minutes:02}:{seconds:02} ago"
                            ),
                                );

                                if ui.button("Load (Shift + L)").clicked() {
                                    to_load = Some(idx);
                                }

                                if ui.button("Remove").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        }

                        if let Some(to_load) = to_load {
                            self.load_at(to_load).ok();
                        }

                        if let Some(to_remove) = to_remove {
                            self.current_savefiles_mut().remove(to_remove);
                        }

                        self.current_savefiles_mut().goto(sel_index);
                    });
                });
        });
    }
}
