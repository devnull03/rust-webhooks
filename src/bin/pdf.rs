use lopdf::{Document, Object, StringFormat};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn load_pdf<P: AsRef<Path>>(path: P) -> Result<Document, lopdf::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Document::load_from(reader)
}

fn main() {
    let path = "templates/sasi.pdf";

    match load_pdf(path) {
        Ok(doc) => {
            println!("Loaded PDF with {} page(s)", doc.get_pages().len());
            
            let catalog = doc.catalog().unwrap();
            let acroform_ref = catalog.get(b"AcroForm").unwrap().as_reference().unwrap();
            let acroform = doc.get_dictionary(acroform_ref).unwrap();
            
            if let Ok(Object::Array(fields)) = acroform.get(b"Fields") {
                println!("Found {} form fields", fields.len());
                
                for field_ref in fields.iter() {
                    if let Ok(field_dict) = doc.get_dictionary(field_ref.as_reference().unwrap()) {

                        if let Ok(Object::String(name_bytes, _)) = field_dict.get(b"T") {
                            let field_name = String::from_utf8_lossy(name_bytes);
                            println!("Field name: {}", field_name);                  
                            
                            // TODO: edit the thingy

                        }
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to load PDF: {}", e);
        }
    }
}
