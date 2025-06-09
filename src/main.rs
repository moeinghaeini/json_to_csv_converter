//! JSON to CSV Converter
//! 
//! A professional GUI application that converts JSON files to CSV format with advanced features.
//! This application provides a user-friendly interface for converting JSON data to CSV format,
//! with support for customization, preview, and various export options.

use eframe::egui;
use rfd::FileDialog;
use serde_json::Value;
use std::path::PathBuf;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::VecDeque;

/// Maximum number of recent files to keep in history
const MAX_RECENT_FILES: usize = 5;

/// Tracks the progress and status of the conversion process
#[derive(Default)]
struct ConversionProgress {
    /// Current status message
    status: String,
    /// Progress value between 0.0 and 1.0
    progress: f32,
    /// Whether a conversion is currently in progress
    is_converting: bool,
}

/// Application settings and configuration
#[derive(Default, Clone)]
struct Settings {
    /// Whether dark mode is enabled
    dark_mode: bool,
    /// CSV delimiter character
    delimiter: String,
    /// Whether to include headers in the CSV output
    include_headers: bool,
    /// Whether to quote fields in the CSV output
    quote_fields: bool,
    /// Maximum number of rows to show in preview
    max_preview_rows: usize,
}

/// Main application state
struct JsonToCsvApp {
    /// Path to the currently loaded JSON file
    json_path: Option<PathBuf>,
    /// Path to the saved CSV file
    csv_path: Option<PathBuf>,
    /// Current application status message
    status: String,
    /// Content of the loaded JSON file
    json_content: Option<String>,
    /// Generated CSV content
    csv_content: Option<String>,
    /// Preview data for the grid view
    preview_data: Option<Vec<Vec<String>>>,
    /// Progress tracking for conversion
    progress: Arc<Mutex<ConversionProgress>>,
    /// Whether to show the preview panel
    show_preview: bool,
    /// Current error message if any
    error_message: Option<String>,
    /// Application settings
    settings: Settings,
    /// List of recently opened files
    recent_files: VecDeque<PathBuf>,
    /// Whether to show the settings panel
    show_settings: bool,
    /// Current search query for preview
    search_query: String,
    /// Selected columns for export
    selected_columns: Vec<String>,
    /// All available columns from the JSON
    all_columns: Vec<String>,
}

impl Default for JsonToCsvApp {
    fn default() -> Self {
        Self {
            json_path: None,
            csv_path: None,
            status: "Ready".to_string(),
            json_content: None,
            csv_content: None,
            preview_data: None,
            progress: Arc::new(Mutex::new(ConversionProgress::default())),
            show_preview: false,
            error_message: None,
            settings: Settings {
                dark_mode: false,
                delimiter: ",".to_string(),
                include_headers: true,
                quote_fields: true,
                max_preview_rows: 100,
            },
            recent_files: VecDeque::new(),
            show_settings: false,
            search_query: String::new(),
            selected_columns: Vec::new(),
            all_columns: Vec::new(),
        }
    }
}

impl JsonToCsvApp {
    /// Creates a new instance of the application
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    /// Opens a file dialog to select a JSON file and loads its contents
    fn select_json_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file() 
        {
            self.json_path = Some(path.clone());
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.json_content = Some(content);
                    self.status = "JSON file loaded successfully".to_string();
                    self.error_message = None;
                    self.preview_data = None;
                    
                    // Add to recent files
                    if !self.recent_files.contains(&path) {
                        if self.recent_files.len() >= MAX_RECENT_FILES {
                            self.recent_files.pop_back();
                        }
                        self.recent_files.push_front(path);
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to read JSON file: {}", e));
                    self.status = "Error loading file".to_string();
                }
            }
        }
    }

    /// Converts the loaded JSON content to CSV format
    /// This function runs the conversion in a separate thread to keep the UI responsive
    fn convert_to_csv(&mut self) {
        let json_content = match &self.json_content {
            Some(content) => content.clone(),
            None => {
                self.error_message = Some("No JSON content loaded".to_string());
                return;
            }
        };

        let progress = Arc::clone(&self.progress);
        let mut progress_guard = progress.lock().unwrap();
        progress_guard.is_converting = true;
        progress_guard.progress = 0.0;
        progress_guard.status = "Starting conversion...".to_string();
        drop(progress_guard);

        let settings = self.settings.clone();
        let selected_columns = self.selected_columns.clone();

        thread::spawn(move || {
            let mut progress_guard = progress.lock().unwrap();
            progress_guard.progress = 0.2;
            progress_guard.status = "Parsing JSON...".to_string();
            drop(progress_guard);

            let json_value: Value = match serde_json::from_str(&json_content) {
                Ok(value) => value,
                Err(e) => {
                    let mut progress_guard = progress.lock().unwrap();
                    progress_guard.status = format!("JSON parsing error: {}", e);
                    progress_guard.is_converting = false;
                    return;
                }
            };

            let mut progress_guard = progress.lock().unwrap();
            progress_guard.progress = 0.4;
            progress_guard.status = "Converting to CSV...".to_string();
            drop(progress_guard);

            // Configure CSV writer with user settings
            let mut csv_writer = csv::WriterBuilder::new()
                .delimiter(settings.delimiter.as_bytes()[0])
                .quote_style(if settings.quote_fields {
                    csv::QuoteStyle::Necessary
                } else {
                    csv::QuoteStyle::Never
                })
                .from_writer(vec![]);

            let mut preview_data = Vec::new();

            match json_value {
                Value::Array(arr) => {
                    if let Some(first) = arr.first() {
                        if let Value::Object(obj) = first {
                            // Get headers based on selection or all columns
                            let headers: Vec<String> = if selected_columns.is_empty() {
                                obj.keys().cloned().collect()
                            } else {
                                selected_columns
                            };

                            // Write headers if enabled
                            if settings.include_headers {
                                csv_writer.write_record(&headers).unwrap();
                                preview_data.push(headers.clone());
                            }

                            // Write data rows
                            for (i, item) in arr.iter().enumerate() {
                                if let Value::Object(obj) = item {
                                    let values: Vec<String> = headers.iter()
                                        .map(|key| obj.get(key)
                                            .map(|v| v.to_string())
                                            .unwrap_or_default())
                                        .collect();
                                    csv_writer.write_record(&values).unwrap();
                                    if i < settings.max_preview_rows {
                                        preview_data.push(values);
                                    }
                                }

                                // Update progress
                                let mut progress_guard = progress.lock().unwrap();
                                progress_guard.progress = 0.4 + (i as f32 / arr.len() as f32) * 0.5;
                                drop(progress_guard);
                            }
                        }
                    }
                }
                Value::Object(obj) => {
                    // Handle single object case
                    let headers: Vec<String> = if selected_columns.is_empty() {
                        obj.keys().cloned().collect()
                    } else {
                        selected_columns
                    };

                    if settings.include_headers {
                        csv_writer.write_record(&headers).unwrap();
                        preview_data.push(headers.clone());
                    }

                    let values: Vec<String> = headers.iter()
                        .map(|key| obj.get(key)
                            .map(|v| v.to_string())
                            .unwrap_or_default())
                        .collect();
                    csv_writer.write_record(&values).unwrap();
                    preview_data.push(values);
                }
                _ => {
                    let mut progress_guard = progress.lock().unwrap();
                    progress_guard.status = "Unsupported JSON structure".to_string();
                    progress_guard.is_converting = false;
                    return;
                }
            }

            let mut progress_guard = progress.lock().unwrap();
            progress_guard.progress = 0.9;
            progress_guard.status = "Finalizing...".to_string();
            drop(progress_guard);

            match String::from_utf8(csv_writer.into_inner().unwrap()) {
                Ok(csv_data) => {
                    let mut progress_guard = progress.lock().unwrap();
                    progress_guard.progress = 1.0;
                    progress_guard.status = "Conversion completed successfully".to_string();
                    progress_guard.is_converting = false;
                }
                Err(e) => {
                    let mut progress_guard = progress.lock().unwrap();
                    progress_guard.status = format!("CSV generation error: {}", e);
                    progress_guard.is_converting = false;
                }
            }
        });
    }

    /// Saves the converted CSV content to a file
    fn save_csv_file(&mut self) {
        if let Some(content) = &self.csv_content {
            if let Some(path) = FileDialog::new()
                .add_filter("CSV", &["csv"])
                .save_file() 
            {
                match std::fs::write(&path, content) {
                    Ok(_) => {
                        self.csv_path = Some(path);
                        self.status = "CSV file saved successfully".to_string();
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to save CSV file: {}", e));
                        self.status = "Error saving file".to_string();
                    }
                }
            }
        }
    }

    /// Displays the settings panel with all configuration options
    fn show_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.add_space(10.0);

        // Theme toggle
        if ui.checkbox(&mut self.settings.dark_mode, "Dark Mode").changed() {
            // Apply theme change
            if self.settings.dark_mode {
                ui.ctx().set_visuals(egui::Visuals::dark());
            } else {
                ui.ctx().set_visuals(egui::Visuals::light());
            }
        }

        ui.add_space(10.0);

        // CSV Settings
        ui.heading("CSV Settings");
        ui.add_space(5.0);

        // Delimiter selection
        ui.horizontal(|ui| {
            ui.label("Delimiter:");
            egui::ComboBox::from_label("")
                .selected_text(&self.settings.delimiter)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.delimiter, ",".to_string(), "Comma (,)");
                    ui.selectable_value(&mut self.settings.delimiter, ";".to_string(), "Semicolon (;)");
                    ui.selectable_value(&mut self.settings.delimiter, "\t".to_string(), "Tab");
                });
        });

        ui.checkbox(&mut self.settings.include_headers, "Include Headers");
        ui.checkbox(&mut self.settings.quote_fields, "Quote Fields");
        
        ui.add_space(10.0);
        ui.add(egui::Slider::new(&mut self.settings.max_preview_rows, 10..=1000)
            .text("Max Preview Rows"));

        // Column Selection
        if !self.all_columns.is_empty() {
            ui.add_space(10.0);
            ui.heading("Column Selection");
            ui.add_space(5.0);

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for column in &self.all_columns {
                        let mut is_selected = self.selected_columns.contains(column);
                        if ui.checkbox(&mut is_selected, column).changed() {
                            if is_selected {
                                self.selected_columns.push(column.clone());
                            } else {
                                self.selected_columns.retain(|c| c != column);
                            }
                        }
                    }
                });
        }
    }

    /// Displays the recent files panel
    fn show_recent_files(&mut self, ui: &mut egui::Ui) {
        if !self.recent_files.is_empty() {
            ui.heading("Recent Files");
            ui.add_space(5.0);

            for path in &self.recent_files {
                if ui.button(path.display().to_string()).clicked() {
                    self.json_path = Some(path.clone());
                    if let Ok(content) = std::fs::read_to_string(path) {
                        self.json_content = Some(content);
                        self.status = "JSON file loaded successfully".to_string();
                        self.error_message = None;
                        self.preview_data = None;
                    }
                }
            }
        }
    }
}

impl eframe::App for JsonToCsvApp {
    /// Main update function that handles the UI rendering and user interactions
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Main content
                ui.vertical(|ui| {
                    ui.heading("JSON to CSV Converter");
                    ui.add_space(20.0);

                    // File selection
                    if ui.button("Select JSON File").clicked() {
                        self.select_json_file();
                    }

                    if let Some(path) = &self.json_path {
                        ui.label(format!("Selected JSON file: {}", path.display()));
                    }

                    ui.add_space(10.0);

                    // Conversion button and progress
                    let progress = self.progress.lock().unwrap();
                    let is_converting = progress.is_converting;
                    let progress_value = progress.progress;
                    let status = progress.status.clone();
                    drop(progress);

                    if !is_converting {
                        if ui.button("Convert to CSV").clicked() {
                            self.convert_to_csv();
                        }
                    }

                    // Progress bar
                    if is_converting {
                        ui.add_space(10.0);
                        let progress_bar = egui::ProgressBar::new(progress_value)
                            .show_percentage()
                            .animate(true);
                        ui.add(progress_bar);
                        ui.label(&status);
                    }

                    // Preview controls
                    if let Some(_content) = &self.csv_content {
                        ui.add_space(10.0);
                        if ui.button("Save CSV File").clicked() {
                            self.save_csv_file();
                        }

                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.show_preview, "Show Preview");
                            if self.show_preview {
                                ui.text_edit_singleline(&mut self.search_query);
                                if ui.button("🔍").clicked() {
                                    // TODO: Implement search functionality
                                }
                            }
                        });
                    }

                    // Preview window
                    if self.show_preview {
                        if let Some(preview_data) = &self.preview_data {
                            ui.add_space(10.0);
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    egui::Grid::new("preview_grid")
                                        .striped(true)
                                        .show(ui, |ui| {
                                            for row in preview_data {
                                                for cell in row {
                                                    ui.label(cell);
                                                }
                                                ui.end_row();
                                            }
                                        });
                                });
                        }
                    }

                    // Error message
                    if let Some(error) = &self.error_message {
                        ui.add_space(10.0);
                        ui.colored_label(egui::Color32::RED, error);
                    }

                    ui.add_space(20.0);
                    ui.label(format!("Status: {}", self.status));
                });

                // Settings panel
                if self.show_settings {
                    ui.separator();
                    ui.vertical(|ui| {
                        self.show_settings_panel(ui);
                    });
                }
            });

            // Bottom panel for recent files
            egui::TopBottomPanel::bottom("recent_files").show(ctx, |ui| {
                self.show_recent_files(ui);
            });

            // Settings toggle in the top bar
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.show_settings, "⚙️ Settings");
                });
            });
        });
    }
}

/// Application entry point
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_title("JSON to CSV Converter"),
        ..Default::default()
    };
    
    eframe::run_native(
        "JSON to CSV Converter",
        options,
        Box::new(|cc| Box::new(JsonToCsvApp::new(cc))),
    )
}

