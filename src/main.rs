use clap::{App, Arg};

fn main() {
    let matches = App::new("LastPass CSV De-Duper")
        .version("1.0")
        .author("John Lyon-Smith")
        .about("Remove duplicate entries from exported LastPass CSV")
        .arg(
            Arg::with_name("CSV_FILE")
                .help("Exported CSV file")
                .required(true)
                .index(1),
        )
        .get_matches();

    println!("Using input file: {}", matches.value_of("INPUT").unwrap());
}
