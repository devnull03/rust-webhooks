use chrono::{DateTime, Datelike};
use lopdf::{Document, Object};
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{info, error};

use crate::notion::structs::Page;

fn load_pdf<P: AsRef<Path>>(path: P) -> Result<Document, lopdf::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Document::load_from(reader)
}

pub fn create_sasi_timesheet(data: TimesheetData) -> Result<Vec<u8>, String> {
    let template_path = "templates/sasi.pdf";
    let mut output_buffer: Vec<u8> = Vec::new();

    let field_identifiers = (
        "Month Day",
        "Start Time",
        "Finish Time",
        "Hours to be Paid",
        "Total hours",
    );

    match load_pdf(template_path) {
        Ok(mut doc) => {
            info!("Loaded PDF with {} page(s)", doc.get_pages().len());

            let field_refs = {
                let catalog = doc.catalog().unwrap();
                let acroform_ref = catalog.get(b"AcroForm").unwrap().as_reference().unwrap();
                let acroform = doc.get_dictionary(acroform_ref).unwrap();

                if let Ok(Object::Array(fields)) = acroform.get(b"Fields") {
                    info!("Found {} form fields", fields.len());

                    fields
                        .iter()
                        .map(|field_ref| field_ref.as_reference().unwrap())
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            };

            let mut processed_entries = 0;

            for field_ref in field_refs.iter() {
                if let Ok(field_dict) = doc.get_dictionary_mut(*field_ref) {
                    if let Ok(Object::String(name_bytes, _)) = field_dict.get(b"T") {
                        let field_name = String::from_utf8_lossy(name_bytes.as_slice());
                        info!("Processing form field: {}", field_name);

                        if field_name.starts_with(field_identifiers.4) {
                            field_dict.set(b"V", data.total_hours.to_string());
                            break;
                        }

                        if processed_entries >= data.entries.len() {
                            continue;
                        }

                        let mut value = String::new();

                        match field_name {
                            _ if field_name.starts_with(field_identifiers.0) => {
                                if field_name.ends_with("_2") {
                                    value = data.entries[processed_entries].day.to_string();
                                } else {
                                    value = data.entries[processed_entries].month.to_string();
                                }
                            }
                            _ if field_name.starts_with(field_identifiers.1) => {
                                value = data.entries[processed_entries].start.clone();
                            }
                            _ if field_name.starts_with(field_identifiers.2) => {
                                value = data.entries[processed_entries].end.clone();
                            }
                            _ if field_name.starts_with(field_identifiers.3) => {
                                value = data.entries[processed_entries].paid_hours.to_string();
                                processed_entries += 1
                            }

                            std::borrow::Cow::Borrowed(_) => {}
                            std::borrow::Cow::Owned(_) => {}
                        }

                        if !value.is_empty() {
                            field_dict.set(b"V", value);
                        }
                    }
                }
            }

            match doc.save_to(&mut output_buffer) {
                Ok(_) => info!(
                    "Successfully converted PDF to bytes, size: {} bytes",
                    output_buffer.len()
                ),
                Err(e) => error!("Failed to convert PDF to bytes: {}", e),
            }

            Ok(output_buffer)
        }
        Err(e) => {
            error!("Failed to load PDF: {}", e);
            Err(format!("Failed to load PDF: {:?}", e))
        }
    }
}

pub struct TimesheetData {
    pub entries: Vec<TimesheetEntry>,
    pub total_hours: f64,
}

pub struct TimesheetEntry {
    month: u32,
    day: u32,
    start: String,
    end: String,
    paid_hours: f64,
}

impl TryFrom<Page> for TimesheetEntry {
    type Error = String;

    fn try_from(page: Page) -> Result<Self, Self::Error> {
        let start_date = DateTime::parse_from_str(
            &page.properties.start_and_end.date.start,
            "%Y-%m-%dT%H:%M:%S.%fZ",
        )
        .map_err(|e| e.to_string())?;

        let month = start_date.month();
        let day = start_date.day();

        let start = start_date.format("%H:%M").to_string();

        let end = page
            .properties
            .start_and_end
            .date
            .end
            .as_ref()
            .ok_or("Missing end time")?;
        let end_date =
            DateTime::parse_from_str(end, "%Y-%m-%dT%H:%M:%S.%fZ").map_err(|e| e.to_string())?;
        let end = end_date.format("%H:%M").to_string();

        let paid_hours = page
            .properties
            .billable_hours
            .formula
            .number
            .ok_or("Missing Hours property")?;

        Ok(TimesheetEntry {
            month,
            day,
            start,
            end,
            paid_hours,
        })
    }
}

impl TryFrom<Vec<Page>> for TimesheetData {
    type Error = String;

    fn try_from(pages: Vec<Page>) -> Result<Self, Self::Error> {
        if pages.len() > 16 {
            return Err("Exceeds max entry length 16".to_string());
        }

        let mut entries = Vec::new();
        let mut total_hours: f64 = 0.0;

        for page in pages {
            let entry = TimesheetEntry::try_from(page)?;
            total_hours += entry.paid_hours;
            entries.push(entry);
        }

        Ok(TimesheetData {
            entries,
            total_hours: total_hours.into(),
        })
    }
}

impl TryFrom<Vec<TimesheetEntry>> for TimesheetData {
    type Error = String;

    fn try_from(entries: Vec<TimesheetEntry>) -> Result<Self, Self::Error> {
        if entries.len() > 16 {
            return Err("Exceeds max entry length 16".to_string());
        }

        let mut total_hours = 0.0;

        for entry in &entries {
            total_hours += entry.paid_hours;
        }

        Ok(TimesheetData {
            entries,
            total_hours,
        })
    }
}
