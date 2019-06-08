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
            panic!("-d に指定できる文字はシングルバイト文字1文字のみです: {}", d);
        }
        bytes[0]
    } else {
        b'\t'
    };

    match (options.get("-f"), options.get("-F")) {
        (Some(f), None) => {
            // ヘッダなし
            let field_map = mcut::get_field_map_1(f.clone());
            let cfg = mcut::Config::new(delimiter, field_map);
            mcut::mcut(&mut reader, &mut writer, cfg);
        }
        (None, Some(f)) => {
            // ヘッダあり
            let line = (&mut reader).lines().next();
            match line {
                Some(Ok(line)) => {
                    let field_map = mcut::get_field_map_2(&line, delimiter, f.clone());
                    let cfg = mcut::Config::new(delimiter, field_map);
                    if options.get("--no-header").is_none() {
                        let header: Vec<&str> = f.split(',')
                            .map(|e| e.splitn(2, ':').next().unwrap())
                            .collect();
                        let header = format!("{}\n", util::join(char::from(delimiter), &header));
                        writer.write(header.as_bytes()).ok();
                    }
                    mcut::mcut(&mut reader, &mut writer, cfg);
                }
                _ => {
                    std::process::exit(0);
                }
            }
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
            } else if arg == "-h" {
                usage();
            } else if options.get("file") == None {
                options.insert("file".to_string(), arg);
            } else {
                panic!("不明なオプション: {}", arg);
            }
        }
    }
    options
}

fn usage() {
    let usage = r#"
    [ usage ]
    mcut [ options ] [FILE]

    [ options ]
        -f: 出力するカラムを0から始まる数字で指定する(カンマ区切り)
            「カラム番号:任意の文字列」を指定すると指定したカラムに固定値を出力できる
            例) -f 0,3,:foo

        -F: 1行目をヘッダとみなし、出力するカラムをカラム名で指定する(カンマ区切り)
            「カラム名:任意の文字列」を指定すると指定したカラムに固定値を出力できる
            例) -F title,id,narrow1:foo

        -d: デリミタ(デフォルト値はタブ)

        --no-header: -F 利用時にヘッダを出力しない

    [ example ]
        cat sample.tsv | mcut -d , -F title,id,number1:0,narrow1: --no-header
    "#;
    eprintln!("{}", usage);
    process::exit(1);
}
