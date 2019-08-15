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
    let cfg = mcut::Config::parse_field_as_name(line.clone(), delimiter, field);
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
    let cfg = mcut::Config::parse_field_as_name(line.clone(), delimiter, field);
    cfg.write_header(&mut writer);
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
    let cfg = mcut::Config::parse_field_as_number(line.clone(), delimiter, field);
    cfg.write_first_line(&mut writer);
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
    let cfg = mcut::Config::parse_field_as_name(line.clone(), delimiter, field);
    cfg.write_header(&mut writer);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_4.csv"), actual);
}

#[test]
fn test_mcut_5() {
    // number, 範囲指定
    let mut reader = BufReader::new(File::open("tests/mcut_test/data.csv").unwrap());
    let mut writer = Cursor::new(vec![]);
    let field = String::from("..,..1,2..,1..2");
    let delimiter = b',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let cfg = mcut::Config::parse_field_as_number(line.clone(), delimiter, field);
    cfg.write_first_line(&mut writer);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_5.csv"), actual);
}

#[test]
fn test_mcut_6() {
    // number, 範囲指定, デフォルト値
    let mut reader = BufReader::new(File::open("tests/mcut_test/data.csv").unwrap());
    let mut writer = Cursor::new(vec![]);
    let field = String::from("..:def1,..1:def2,2..:def3,1..2:def4");
    let delimiter = b',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let cfg = mcut::Config::parse_field_as_number(line.clone(), delimiter, field);
    cfg.write_first_line(&mut writer);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_6.csv"), actual);
}

#[test]
fn test_mcut_7() {
    // name, 範囲指定
    let mut reader = BufReader::new(File::open("tests/mcut_test/data.csv").unwrap());
    let mut writer = Cursor::new(vec![]);
    let field = String::from("..,..title,title..kana,kana..,..,..1,1..2,2..");
    let delimiter = b',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let cfg = mcut::Config::parse_field_as_name(line.clone(), delimiter, field);
    cfg.write_header(&mut writer);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_7.csv"), actual);
}

#[test]
fn test_mcut_8() {
    // name, 範囲指定, デフォルト値
    let mut reader = BufReader::new(File::open("tests/mcut_test/data.csv").unwrap());
    let mut writer = Cursor::new(vec![]);
    let field = String::from("..:def1,..title:def2,title..kana:def3,kana..:def4,..:def5,..1:def6,1..2:def7,2..:def8");
    let delimiter = b',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let cfg = mcut::Config::parse_field_as_name(line.clone(), delimiter, field);
    cfg.write_header(&mut writer);
    mcut::mcut(&mut reader, &mut writer, cfg);
    let actual: String = String::from_utf8(writer.get_ref().to_vec()).unwrap();
    assert_eq!(read_all("tests/mcut_test/expected_8.csv"), actual);
}

fn read_all(file_name: &str) -> String {
    let mut f = File::open(file_name).ok().unwrap();
    let mut buf: String = String::new();
    f.read_to_string(&mut buf).ok().unwrap();
    buf
}
