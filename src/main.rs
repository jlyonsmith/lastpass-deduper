use clap::{App, Arg};
use csv::StringRecord;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let result = real_main();

    if let Err(ref err) = result {
        eprintln!("error: {}", err);
    }

    result
}

fn real_main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("LastPass CSV De-Duper")
        .version("1.0.0-20120712.0")
        .author("John Lyon-Smith")
        .about("Display duplicate entries from exported LastPass CSV")
        .arg(
            Arg::with_name("CSV_FILE")
                .help("Unprocessed LastPass CSV file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .help("Processed CSV file in LastPass format")
                .long("output")
                .short("o")
                .takes_value(true)
                .value_name("CSV_FILE")
                .required(false),
        )
        .get_matches();

    let mut csv_reader = File::open(Path::new(matches.value_of("CSV_FILE").unwrap())).unwrap();
    let mut csv_writer: Box<dyn Write> = match matches.value_of("output") {
        Some(f) => Box::new(File::create(Path::new(f))?),
        None => Box::new(std::io::stdout()),
    };

    process_csv(&mut csv_reader, &mut csv_writer)
}

trait HeaderHelpers {
    fn column_of(self: &Self, name: &str) -> usize;
}

impl HeaderHelpers for StringRecord {
    fn column_of(self: &Self, name: &str) -> usize {
        self.iter().position(|f| f == name).unwrap()
    }
}

fn process_csv(
    csv_reader: &mut dyn Read,
    csv_writer: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(csv_reader);
    let mut writer = csv::Writer::from_writer(csv_writer);
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
                    eprintln!("'{}' at line {} matches line {}", name, line, other_line);
                } else {
                    eprintln!(
                        "'{}' at line {} is different from record at line {}",
                        name, line, other_line
                    );
                }
            } else {
                writer.write_record(record.iter())?;
                map.insert(name.to_string(), record);
            }
        }
    }
    Ok(())
}
