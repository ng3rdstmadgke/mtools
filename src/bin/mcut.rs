extern crate mtools;

use mtools::mcut;
use std::env;
use std::env::Args;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::BufReader;
use std::fs::File;
use std::process;

fn main() {
    let options = parse_args(env::args());
    let mut reader: BufReader<Box<Read>> = if let Some(file) = options.get("file") {
        BufReader::new(Box::new(File::open(file).ok().unwrap()))
    } else {
        BufReader::new(Box::new(io::stdin()))
    };
    let mut writer = BufWriter::new(io::stdout());

    let delimiter: u8 = if let Some(d) = options.get("-d") {
        let bytes: &[u8] = d.as_bytes();
        if bytes.len() > 1 {
            eprintln!("-d に指定できる文字はシングルバイト文字1文字のみです: {}", d);
            process::exit(1);
        }
        bytes[0]
    } else {
        b'\t'
    };

    if let Some(Ok(line)) = (&mut reader).lines().next() {
        // カラム名とindexの対応表を作成
        if let Some(fields) = options.get("-f") {
            // -f オプション: ヘッダを考慮しない
            let cfg = mcut::Config::parse_field_as_number(line.clone(), delimiter, fields.clone());
            // 1行目を出力する
            cfg.write_first_line(&mut writer);
            mcut::mcut(&mut reader, &mut writer, cfg);
        } else if let Some(fields) = options.get("-F") {
            // -F オプション: ヘッダを考慮する
            let cfg = mcut::Config::parse_field_as_name(line.clone(), delimiter, fields.clone());
            if let None = options.get("--no-header") {
                // --no-headerオプションが指定されていなければ1行目を出力する
                cfg.write_header(&mut writer);
            }
            mcut::mcut(&mut reader, &mut writer, cfg);
        } else {
            eprintln!("-f と -f 少なくともどちらか一方を指定してください。");
            process::exit(1);
        };
    } else {
        std::process::exit(0);
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
            if arg == "-h" || arg == "--help" {
                help();
            } else if arg == "-f" {
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
                eprintln!("不明なオプション: {}", arg);
                process::exit(1);
            }
        }
    }
    options
}

fn help() {
    eprintln!("{}", include_str!("../resources/mcut.txt"));
    process::exit(1);
}
