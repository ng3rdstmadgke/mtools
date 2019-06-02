extern crate mtools;

use mtools::util;
use mtools::mcut;
use std::env;
use std::env::Args;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::BufReader;
use std::fs::File;

fn main() {
    let options = parse_args(env::args());
    let mut reader: BufReader<Box<Read>> = if let Some(file) = options.get("file") {
        BufReader::new(Box::new(File::open(file).ok().unwrap()))
    } else {
        BufReader::new(Box::new(io::stdin()))
    };
    let mut writer = BufWriter::new(io::stdout());

    let delimiter: char = if let Some(d) = options.get("-d") {
        let chars: Vec<char> = d.chars().collect();
        if chars.len() > 1 {
            panic!("-d に指定できる文字は1文字です: {}", d);
        }
        chars[0]
    } else {
        '\t'
    };

    match (options.get("-f"), options.get("-F")) {
        (Some(f), None) => {
            // ヘッダなし
            let (field_map, default_map) = mcut::get_field_map_1(f.clone());
            let cfg = mcut::Config::new(delimiter, field_map, default_map);
            mcut::ecut(&mut reader, &mut writer, cfg);
        }
        (None, Some(f)) => {
            // ヘッダあり
            let line = (&mut reader).lines().next().unwrap().ok().unwrap();
            let (field_map, default_map) = mcut::get_field_map_2(&line, delimiter, f.clone());
            let cfg = mcut::Config::new(delimiter, field_map, default_map);
            if options.get("--no-header").is_none() {
                let header: Vec<&str> = f.split(',')
                    .map(|e| e.splitn(2, ':').next().unwrap())
                    .collect();
                let header = format!("{}\n", util::join(delimiter, &header));
                writer.write(header.as_bytes()).ok();
            }
            mcut::ecut(&mut reader, &mut writer, cfg);
        }
        (_, _) => panic!("-f, -F どちらか一方を指定してください。"),
    };

}

fn parse_args(mut args: Args) -> HashMap<String, String> {
    let mut options = HashMap::new();
    let mut key: Option<String> = None;
    let _script = args.next().unwrap();
    for arg in args {
        if let Some(k) = key {
            options.insert(k.clone(), arg);
            key = None;
        } else {
            if arg == "-f" {
                key = Some(arg);
            } else if arg == "-F" {
                key = Some(arg);
            } else if arg == "-d" {
                key = Some(arg);
            } else if arg == "--no-header" {
                options.insert("--no-header".to_string(), arg);
            } else if options.get("file") == None {
                options.insert("file".to_string(), arg);
            } else {
                panic!("不明なオプション: {}", arg);
            }
        }
    }
    options
}

