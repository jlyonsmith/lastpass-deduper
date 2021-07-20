use clap::{App, Arg};
use console::{set_colors_enabled, style, Term};
use csv::StringRecord;
use dialoguer::{Input, Select};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let result = run();

    if let Err(ref err) = result {
        eprintln!("error: {}", err);
    }

    result
}

fn run() -> Result<(), Box<dyn Error>> {
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

    // Force colors even if stdout is redirected
    set_colors_enabled(true);

    // Read values and process
    let mut csv_reader = File::open(Path::new(matches.value_of("CSV_FILE").unwrap()))?;
    let map = process_csv(&mut csv_reader)?;

    // Write all values out to stdout or new file
    let csv_writer: Box<dyn Write> = match matches.value_of("output") {
        Some(f) => Box::new(File::create(Path::new(f))?),
        None => Box::new(std::io::stdout()),
    };
    let mut writer = csv::Writer::from_writer(csv_writer);

    for (_, value) in map.iter() {
        writer.write_record(value)?;
    }

    Ok(())
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
) -> Result<HashMap<String, csv::StringRecord>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(csv_reader);
    let mut map: HashMap<String, csv::StringRecord> = HashMap::new();
    let headers = reader.headers()?.clone();
    let name_pos = headers.column_of("name");
    let eterm = Term::stderr();

    for result in reader.records() {
        let record = result?;

        if let Some(name) = record.get(name_pos) {
            if let Some(other_record) = map.get(name) {
                // Record is a duplicate
                let line = record.position().map_or(1, |pos| pos.line() + 1);
                let other_line = other_record.position().map_or(1, |pos| pos.line() + 1);
                enum Action {
                    Merge,
                    DropOne,
                    DropBoth,
                    Split,
                }
                let mut action = Action::DropOne;
                let mut differences = 0;
                let mut new_record: Vec<String> = vec![];

                // See if the all the fields are the same
                for (pos, other_field) in other_record.iter().enumerate() {
                    if pos == name_pos {
                        new_record.push(other_field.to_string());
                        continue;
                    }

                    let field = record.get(pos).unwrap();

                    if field == other_field {
                        new_record.push(other_field.to_string());
                        continue;
                    }

                    // There is at least one difference

                    if differences == 0 {
                        eterm.write_line(
                            &style(format!(
                                "'{}' at line {} is different from record at line {}",
                                name, line, other_line
                            ))
                            .yellow()
                            .to_string(),
                        )?;
                        eprintln!("{}", record.iter().collect::<Vec<&str>>().join(","));
                        eprintln!("{}", other_record.iter().collect::<Vec<&str>>().join(","));

                        let selection = Select::new()
                            .item("Merge")
                            .item("Drop Both")
                            .item("Split")
                            .default(0)
                            .interact()?;

                        if selection == 1 {
                            action = Action::DropBoth;
                            break;
                        } else if selection == 2 {
                            action = Action::Split;
                            break;
                        } else {
                            action = Action::Merge;
                        }
                    }

                    differences += 1;

                    let selection = Select::new()
                        .with_prompt(format!("Which '{}'?", headers.get(pos).unwrap()))
                        .item(field)
                        .item(other_field)
                        .interact()?;

                    new_record.push(match selection {
                        0 => field.to_string(),
                        1 => other_field.to_string(),
                        _ => panic!(),
                    });
                }

                match action {
                    Action::DropOne => {
                        eterm.write_line(
                            &style(format!(
                                "'{}' at line {} matches line {}, dropping one",
                                name, line, other_line
                            ))
                            .yellow()
                            .to_string(),
                        )?;
                        ()
                    }
                    Action::DropBoth => (), // Just don't add either to the map
                    Action::Merge => {
                        map.insert(name.to_string(), StringRecord::from(new_record));
                        ()
                    }
                    Action::Split => {
                        let new_name: String = Input::new()
                            .with_prompt("Name for second record:")
                            .with_initial_text(name.to_string() + &" New".to_string())
                            .interact_text()?;

                        new_record = other_record
                            .iter()
                            .enumerate()
                            .map(|(pos, value)| {
                                if pos == name_pos {
                                    new_name.clone()
                                } else {
                                    value.to_string()
                                }
                            })
                            .collect();
                        map.insert(new_name, StringRecord::from(new_record));
                        ()
                    }
                }
            } else {
                map.insert(name.to_string(), record);
            }
        }
    }

    Ok(map)
}
