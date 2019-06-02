extern crate mtools;

use mtools::mcut;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Cursor;
use std::fs::File;


fn main() {
    let file = File::open("tests/mcut_test/data.tsv").ok().unwrap();
    let mut reader: BufReader<File> = BufReader::new(file);
    let mut writer = Cursor::new(vec![]);
    let field = String::from("kana,title,field:word,src:0,narrow1:-");
    let delimiter = ',';
    let line = (&mut reader).lines().next().unwrap().ok().unwrap();
    let (field_map, default_map) = mcut::get_field_map_2(&line, delimiter, field);
    let cfg = mcut::Config::new(delimiter, field_map, default_map);
    mcut::ecut(&mut reader, &mut writer, cfg);
    assert_eq!(&read_all("tests/mcut_test/expected.tsv"), writer.get_ref());
}

fn read_all(file_name: &str) -> Vec<u8> {
    let mut f = File::open(file_name).ok().unwrap();
    let mut buf: Vec<u8> = Vec::new();
    f.read_to_end(&mut buf).ok().unwrap();
    buf
}
