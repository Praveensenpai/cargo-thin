use std::sync::mpsc;

use crate::domain::cleaner::{clean_project, CleanReport};
use crate::domain::project::{ProjectInfo, SortOrder};

#[derive(Debug)]
pub enum CleanEvent {
    Progress {
        current: usize,
        total: usize,
        name: String,
    },
    Done(Vec<CleanReport>),
    Error(String),
}

#[derive(Debug)]
pub enum AppMode {
    Browsing,
    ConfirmModal,
    Cleaning {
        rx: mpsc::Receiver<CleanEvent>,
        current: usize,
        total: usize,
        current_name: String,
        tick: u8,
    },
    SummaryModal(Vec<CleanReport>),
    HelpModal,
}

pub struct App {
    pub projects: Vec<ProjectInfo>,
    pub selected_index: usize,
    pub sort_order: SortOrder,
    pub keep_both: bool,
    pub mode: AppMode,
    pub status_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(projects: Vec<ProjectInfo>, keep_both: bool) -> Self {
        let mut app = Self {
            projects,
            selected_index: 0,
            sort_order: SortOrder::TargetSizeDesc,
            keep_both,
            mode: AppMode::Browsing,
            status_message: None,
            should_quit: false,
        };
        app.apply_sort();
        app
    }

    pub fn selected_project(&self) -> Option<&ProjectInfo> {
        self.projects.get(self.selected_index)
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.projects.is_empty() && self.selected_index + 1 < self.projects.len() {
            self.selected_index += 1;
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(project) = self.projects.get_mut(self.selected_index) {
            project.selected = !project.selected;
        }
    }

    pub fn toggle_all(&mut self) {
        let any_unselected = self.projects.iter().any(|p| !p.selected);
        for p in &mut self.projects {
            p.selected = any_unselected;
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_order = self.sort_order.next();
        self.apply_sort();
    }

    fn apply_sort(&mut self) {
        let keep_both = self.keep_both;
        match self.sort_order {
            SortOrder::TargetSizeDesc => {
                self.projects
                    .sort_by_key(|a| std::cmp::Reverse(a.total_target_bytes));
            }
            SortOrder::ReclaimableDesc => {
                self.projects
                    .sort_by_key(|a| std::cmp::Reverse(a.reclaimable_bytes(keep_both)));
            }
            SortOrder::NameAsc => {
                self.projects
                    .sort_by(|a, b| a.package_name.cmp(&b.package_name));
            }
        }
        if self.selected_index >= self.projects.len() && !self.projects.is_empty() {
            self.selected_index = self.projects.len() - 1;
        }
    }

    pub fn toggle_keep_both(&mut self) {
        self.keep_both = !self.keep_both;
    }

    pub fn total_target_bytes(&self) -> u64 {
        self.projects.iter().map(|p| p.total_target_bytes).sum()
    }

    pub fn total_reclaimable_bytes(&self) -> u64 {
        self.projects
            .iter()
            .map(|p| p.reclaimable_bytes(self.keep_both))
            .sum()
    }

    pub fn selected_count(&self) -> usize {
        self.projects.iter().filter(|p| p.selected).count()
    }

    pub fn selected_reclaimable_bytes(&self) -> u64 {
        self.projects
            .iter()
            .filter(|p| p.selected)
            .map(|p| p.reclaimable_bytes(self.keep_both))
            .sum()
    }

    pub fn start_clean(&mut self) {
        let keep_both = self.keep_both;
        let total = self.selected_count();
        let projects: Vec<ProjectInfo> = self
            .projects
            .iter()
            .filter(|p| p.selected)
            .cloned()
            .collect();

        let (tx, rx) = mpsc::channel::<CleanEvent>();
        std::thread::spawn(move || run_clean_worker(projects, keep_both, total, tx));

        self.mode = AppMode::Cleaning {
            rx,
            current: 0,
            total,
            current_name: String::new(),
            tick: 0,
        };
    }

    pub fn poll_clean(&mut self) {
        let AppMode::Cleaning {
            ref rx,
            ref mut current,
            ref mut total,
            ref mut current_name,
            ref mut tick,
        } = self.mode
        else {
            return;
        };

        *tick = tick.wrapping_add(1);
        let (new_mode, errors) = drain_clean_events(rx, current, total, current_name);

        if !errors.is_empty() {
            self.status_message = Some(errors.join(" | "));
        }

        if let Some(mode) = new_mode {
            self.finish_cleaning(mode);
        }
    }

    fn finish_cleaning(&mut self, next_mode: AppMode) {
        if let AppMode::SummaryModal(ref reports) = next_mode {
            self.update_project_sizes(reports);
        }
        self.mode = next_mode;
        self.apply_sort();
    }

    fn update_project_sizes(&mut self, reports: &[CleanReport]) {
        for r in reports {
            if let Some(p) = self
                .projects
                .iter_mut()
                .find(|p| p.package_name == r.package_name)
            {
                p.total_target_bytes = r.bytes_after;
                p.selected = false;
            }
        }
    }
}

fn run_clean_worker(
    projects: Vec<ProjectInfo>,
    keep_both: bool,
    total: usize,
    tx: mpsc::Sender<CleanEvent>,
) {
    let mut reports = Vec::new();
    for (idx, project) in projects.iter().enumerate() {
        let _ = tx.send(CleanEvent::Progress {
            current: idx + 1,
            total,
            name: project.package_name.clone(),
        });
        match clean_project(project, keep_both) {
            Ok(report) => reports.push(report),
            Err(e) => {
                let _ = tx.send(CleanEvent::Error(format!(
                    "Failed cleaning {}: {e}",
                    project.package_name
                )));
            }
        }
    }
    let _ = tx.send(CleanEvent::Done(reports));
}

fn drain_clean_events(
    rx: &mpsc::Receiver<CleanEvent>,
    current: &mut usize,
    total: &mut usize,
    current_name: &mut String,
) -> (Option<AppMode>, Vec<String>) {
    let mut errors = Vec::new();
    while let Ok(event) = rx.try_recv() {
        match event {
            CleanEvent::Progress {
                current: c,
                total: t,
                name,
            } => {
                *current = c;
                *total = t;
                *current_name = name;
            }
            CleanEvent::Error(msg) => errors.push(msg),
            CleanEvent::Done(reports) => return (Some(AppMode::SummaryModal(reports)), errors),
        }
    }
    (None, errors)
}
