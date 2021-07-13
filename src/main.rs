use clap::{App, Arg, ArgMatches};
use csv::{ReaderBuilder, StringRecord};
use ordered_float::OrderedFloat;
use std::fs::File;
use std::io::prelude::*;

use query_ddlog::api::HDDlog;
use query_ddlog::relid2name;
use query_ddlog::typedefs::*;
use query_ddlog::Relations;

use differential_datalog::ddval::{DDValConvert, DDValue};
use differential_datalog::program::RelId;
use differential_datalog::program::Update;
use differential_datalog::DeltaMap;
use differential_datalog::{DDlog, DDlogDynamic};

fn relation_str_to_enum(relation: &str) -> Relations {
    match relation {
        "edge" | "Edge" => Relations::Edge,
        _ => panic!("Unknown input relation: {}", relation),
    }
}

fn parse_tuple_for_relation(tuple: StringRecord, offset: usize, relation: Relations) -> DDValue {
    match relation {
        Relations::Edge => Edge {
            parent: tuple[offset + 0].to_string(),
            child: tuple[offset + 1].to_string(),
            weight: OrderedFloat::from(tuple[offset + 2].parse::<f32>().unwrap()),
        }
            .into_ddvalue(),
        _ => panic!("Unsupported input relation: {:?}", relation),
    }
}

fn get_cli_arg_matches<'a>() -> ArgMatches<'a> {
    App::new("DDlog Benchmark CLI")
        .arg(
            Arg::with_name("input")
                .short("i")
                .help("Specifies a CSV input file for a relation. We expect CSVs to have headers.")
                .takes_value(true)
                .number_of_values(2)
                .value_names(&["relation", "csv"])
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("updates")
                .short("u")
                .help("Specifies a CSV file for updates to the computation. We expect CSVs to have headers.\
                The first column of the CSV should be either -1 or 1, indicating whether to add or remove a tuple.\
                The second column must be the name of the relation, followed by the relation attributes.")
                .takes_value(true)
                .value_names(&["csv"])
                .multiple(true)
                .required(false),
        )
        .get_matches()
}

fn main() -> Result<(), String> {
    let matches = get_cli_arg_matches();
    println!("Instantiating DDlog program...");
    let start = std::time::Instant::now();
    // Returns a handle to the program and initial contents of output relations.
    // Arguments
    // - number of worker threads (you typically want 1 or 2).
    // - Boolean flag that indicates whether DDlog will track the complete snapshot
    //   of output relations.  Should only be used for debugging in order to dump
    //   the contents of output tables using `HDDlog::dump_table()`.  Otherwise,
    //   indexes are the preferred way to achieve this.
    let (hddlog, init_state) = HDDlog::run(1, false)?;

    println!(
        "Instantiating program took {} µs",
        start.elapsed().as_micros()
    );
    dump_delta(&init_state);

    let initial_start = std::time::Instant::now();
    println!("Adding inputs to the dataflow with multiple transactions");
    let inputs: Vec<_> = matches.values_of("input").unwrap().collect();
    for i in (0..inputs.len()).step_by(2) {
        let (input_relation, csv_file) = (relation_str_to_enum(inputs[i]), inputs[i + 1]);
        let file = File::open(csv_file).expect(&*format!("Could not open file {}", csv_file));
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        let max_batch_size = 1000;
        let mut batch = Vec::with_capacity(max_batch_size);

        start_transaction(&hddlog);
        for tuple in rdr.records().flatten() {
            batch.push(Update::Insert {
                relid: input_relation as RelId,
                v: parse_tuple_for_relation(tuple, 0, input_relation),
            });

            if batch.len() >= max_batch_size {
                hddlog.apply_updates(&mut batch.drain(..).into_iter())?;
            }
        }
        hddlog.apply_updates(&mut batch.into_iter())?;
        commit_transaction(&hddlog);
    }
    println!(
        "Initial computation took {} µs to complete",
        initial_start.elapsed().as_micros()
    );

    let updates: Vec<_> = matches
        .values_of("updates")
        .map_or_else(Vec::new, |values| values.collect());
    for file_name in updates {
        let start = std::time::Instant::now();
        println!("Adding updates from file {} to the dataflow", file_name);
        let file = File::open(file_name).expect(&*format!("Could not open file {}", file_name));
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        start_transaction(&hddlog);
        let mut updates = vec![];
        for tuple in rdr.records().flatten() {
            let add_or_remove = tuple.get(0).expect("Empty rows are not valid");
            let relation =
                relation_str_to_enum(tuple.get(1).expect("Each row must contain a relation"));

            if add_or_remove == "1" {
                updates.push(Update::Insert {
                    relid: relation as RelId,
                    v: parse_tuple_for_relation(tuple, 2, relation),
                });
            } else if add_or_remove == "-1" {
                updates.push(Update::DeleteValue {
                    relid: relation as RelId,
                    v: parse_tuple_for_relation(tuple, 2, relation),
                });
            } else {
                panic!(
                    "First column must either be 1 or -1 but was: {:?}",
                    add_or_remove
                )
            }
        }
        hddlog.apply_updates(&mut updates.into_iter())?;
        println!(
            "Finished adding updates after {} µs.",
            start.elapsed().as_micros()
        );
        commit_transaction(&hddlog);
    }

    hddlog.stop().unwrap();
    Ok(())
}

fn start_transaction(hddlog: &HDDlog) {
    println!("Starting transaction...");
    hddlog.transaction_start().unwrap();
}

fn commit_transaction(hddlog: &HDDlog) {
    let start = std::time::Instant::now();
    let delta = hddlog.transaction_commit_dump_changes().unwrap();
    println!(
        "Committing transaction took {} µs",
        start.elapsed().as_micros()
    );
    dump_delta(&delta);
}

fn dump_delta(delta: &DeltaMap<DDValue>) {
    for (rel, changes) in delta.iter() {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(format!("{}.out", relid2name(*rel).unwrap()))
            .expect(&*format!(
                "Could not open file for writing exported relation: {:?}.out",
                relid2name(*rel)
            ));
        for (val, weight) in changes.iter() {
            let _ = writeln!(file, "{:+}, ({})", weight, val);
        }
    }
}
