use clap::{App, Arg};
use csv::StringRecord;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

fn main() {
    let matches = App::new("LastPass CSV De-Duper")
        .version("1.0")
        .author("John Lyon-Smith")
        .about("Display duplicate entries from exported LastPass CSV")
        .arg(
            Arg::with_name("CSV_FILE")
                .help("Exported CSV file")
                .required(true)
                .index(1),
        )
        .get_matches();

    if let Err(err) = find_duplicates(Path::new(matches.value_of("CSV_FILE").unwrap())) {
        println!("error: {}", err);
    }
}

trait HeaderHelpers {
    fn column_of(self: &Self, name: &str) -> usize;
}

impl HeaderHelpers for StringRecord {
    fn column_of(self: &Self, name: &str) -> usize {
        self.iter().position(|f| f == name).unwrap()
    }
}

fn find_duplicates(csv_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(csv_path)?;
    let mut map: HashMap<String, csv::StringRecord> = HashMap::new();
    let url_pos: usize;
    let name_pos: usize;
    let password_pos: usize;
    let username_pos: usize;

    {
        let headers = reader.headers()?;

        url_pos = headers.column_of("url");
        name_pos = headers.column_of("name");
        password_pos = headers.column_of("password");
        username_pos = headers.column_of("username");
    }

    for result in reader.records() {
        let record = result?;

        if let Some(name) = record.get(name_pos) {
            if let Some(other_record) = map.get(name) {
                // Record is a duplicate, check in what way
                let line = record.position().map_or(1, |pos| pos.line() + 1);
                let other_line = other_record.position().map_or(1, |pos| pos.line() + 1);

                // Is other important stuff is the same?
                if other_record.get(username_pos) == record.get(username_pos)
                    && other_record.get(password_pos) == record.get(password_pos)
                    && other_record.get(url_pos) == record.get(url_pos)
                {
                    // TODO: Write duplicate to stderr
                    // TODO: Write non-duplicate to stdout
                    println!("'{}' at line {} matches line {}", name, line, other_line);
                } else {
                    println!(
                        "'{}' at line {} is different from record at line {}",
                        name, line, other_line
                    );
                }
            } else {
                // Record is unique, capture it and write it out
                map.insert(name.to_string(), record);
            }
        }
    }
    Ok(())
}
