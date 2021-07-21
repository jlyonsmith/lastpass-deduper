# LastPass De-Duper

A simple Rust program to help with de-duping your LastPass export file ready for import into another password manager.

## Background

I recently switched from LastPass to the excellent and open source [Bitwarden](https://bitwarden.com/) password manager.  In doing so I wanted to clean up the 1300+ passwards I've amassed over the last decade. I wrote this little Rust program to help.  You feed it the exported CSV file and it will write a cleaned-up version in the same format. The tool works as follows:

- If the tool finds a straight duplicate where all fields are the same, it removes the duplicate entry.
- If there is a duplicate in `name` only, the tool will prompt to merge, split or delete both entries.
  - If you choose to merge, the tool will go through each field that differs and prompt for you to pick one.

I was able to process my 1300+ entries down to around 800 in under an hour.

*Note that LastPass does not export secure notes and other non-site related records, so you'll have to do those manually.* 🙁

## Installation

1. [Install Rust](https://rustup.rs/) (or `brew rustup-init` if you use [Homebrew](https://brew.sh/))
2. `rustup toolchain install stable`
3. Clone this repo.
4. [Export your LastPass CSV](https://support.logmeininc.com/lastpass).
5. Run the tool with something like `cargo run -- lastpass_export.csv --output clean_lastpass_export.csv`
