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
    let field_map = mcut::get_field_map_2(&line, delimiter, field);
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
    let field_map = mcut::get_field_map_2(&line, delimiter, field);
    let cfg = mcut::Config::new(delimiter, field_map);
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
    let field_map = mcut::get_field_map_1(field);
    let cfg = mcut::Config::new(delimiter, field_map);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_3.csv"), actual);
}

fn read_all(file_name: &str) -> String {
    let mut f = File::open(file_name).ok().unwrap();
    let mut buf: String = String::new();
    f.read_to_string(&mut buf).ok().unwrap();
    buf
}
