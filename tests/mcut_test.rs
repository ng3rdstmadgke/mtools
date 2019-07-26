extern crate mtools;

use mtools::mcut;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Cursor;
use std::fs::File;


#[test]
fn test_mcut_1() {
    let mut reader = BufReader::new(File::open("tests/mcut_test/data.csv").unwrap());
    let mut writer = Cursor::new(vec![]);
    let field = String::from("kana,title,field:word,src:0,narrow1:-");
    let delimiter = b',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let field_map = mcut::fmap_from_col_name(&line, delimiter, field);
    let cfg = mcut::Config::new(delimiter, field_map);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected.csv"), actual);
}

#[test]
fn test_mcut_2() {
    let mut reader = BufReader::new(File::open("tests/mcut_test/data.csv").unwrap());
    let mut writer = Cursor::new(vec![]);
    let field = String::from("id,title,narrow1,field:,kana");
    let delimiter = b',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let field_map = mcut::fmap_from_col_name(&line, delimiter, field.clone());
    let cfg = mcut::Config::new(delimiter, field_map);
    mcut::write_header(&mut writer, &cfg);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_2.csv"), actual);
}

#[test]
fn test_mcut_3() {
    let mut reader = BufReader::new(File::open("tests/mcut_test/data.csv").unwrap());
    let mut writer = Cursor::new(vec![]);
    let field = String::from("3,:foo,1,0");
    let delimiter = b',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let field_map = mcut::fmap_from_col_number(&line, delimiter, field);
    let cfg = mcut::Config::new(delimiter, field_map);
    mcut::mcut_line(line.as_bytes(), &mut writer, &cfg);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_3.csv"), actual);
}

#[test]
fn test_mcut_4() {
    let mut reader = BufReader::new(File::open("tests/mcut_test/data.csv").unwrap());
    let mut writer = Cursor::new(vec![]);
    let field = String::from("3,:foo,1,kana,0,piyo:sample,narrow1");
    let delimiter = b',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let field_map = mcut::fmap_from_col_name(&line, delimiter, field.clone());
    let cfg = mcut::Config::new(delimiter, field_map);
    mcut::write_header(&mut writer, &cfg);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_4.csv"), actual);
}

fn read_all(file_name: &str) -> String {
    let mut f = File::open(file_name).ok().unwrap();
    let mut buf: String = String::new();
    f.read_to_string(&mut buf).ok().unwrap();
    buf
}
