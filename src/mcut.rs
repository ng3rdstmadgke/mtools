extern crate memchr;

use std::io::prelude::*;
use std::io::BufReader;

pub fn get_field_map_1(field: String) -> Vec<(usize, Option<String>)> {
    let field_map: Vec<(usize, Option<String>)> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                (0, Some(split[1].to_string()))
            } else {
                if let Some(f) = e.trim().parse::<usize>().ok() {
                    (f, None)
                } else {
                    panic!("不明なフィールド: {}", e);
                }
            }
        }).collect();
    field_map
}

pub fn get_field_map_2(header: &str, delimiter: u8, field: String) -> Vec<(usize, Option<String>)> {
    let cols: Vec<&str> = header.split(char::from(delimiter)).collect();
    let field_map: Vec<(usize, Option<String>)> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                let col = cols.iter().position(|c| c == &split[0]).unwrap_or(0);
                (col, Some(split[1].to_string()))
            } else {
                let col = cols.iter().position(|c| c == &e).expect(&format!("不明なフィールド: {}", e));
                (col, None)
            }
        }).collect();
    field_map
}

pub fn mcut<R: Read, W: Write>(reader: &mut BufReader<R>, writer: &mut W, cfg: Config) {
    let max: usize = *cfg.field_map.iter().map(|(i, _)|i).max().unwrap();
    // 読み込んだ文字列を格納する配列
    let mut buf: Vec<u8> = Vec::new();
    // 区切り文字のindexを格納する配列
    let mut split: Vec<usize> = vec![0; max + 2];
    while reader.read_until(b'\n', &mut buf).ok().unwrap() > 0 {
        // 必要なところまで読み込む
        for (i, position) in memchr::memchr_iter(cfg.delimiter, &buf).enumerate() {
            if i <= max {
                split[i + 1] = position;
            } else {
                break;
            }
        }

        // 書き込み処理
        match cfg.field_map[0] {
            (_, Some(ref default)) => {
                writer.write(default.as_bytes()).unwrap();
            },
            (col_idx, None) => {
                let start = split[col_idx] + 1;
                let end   = split[col_idx + 1];
                writer.write(&buf[start..end]).unwrap();
            }
        }
        for f in (&cfg.field_map)[1..].iter() {
            match f {
                &(_, Some(ref default)) => {
                    writer.write(&[cfg.delimiter]).unwrap();
                    writer.write(default.as_bytes()).unwrap();
                },
                &(col_idx, None) => {
                    let start = split[col_idx];
                    let end   = split[col_idx + 1];
                    writer.write(&buf[start..end]).unwrap();
                }
            }
        }
        writer.write(b"\n").unwrap();
        buf.clear();
    }
}

pub struct Config {
    pub delimiter: u8,
    pub field_map: Vec<(usize, Option<String>)>, // field_idx, default_value
}

impl Config {
    pub fn new(delimiter: u8, field_map: Vec<(usize, Option<String>)>) -> Self {
        Config { delimiter, field_map }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_field_map_1() {
        let field = String::from("2,4,6,2:,3:foo,:0,5");
        let field_map = get_field_map_1(field);
        let expected: Vec<(usize, Option<String>)> = vec![
            (2, None),
            (4, None),
            (6, None),
            (0, Some(String::from(""))),
            (0, Some(String::from("foo"))),
            (0, Some(String::from("0"))),
            (5, None),
        ];
        assert_eq!(expected, field_map);
    }

    #[test]
    fn test_get_field_map_2() {
        let field = String::from("title,field:word,src:0,kana:,title,narrow1,narrow2");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        let field_map = get_field_map_2(&header, b',', field);
        let expected: Vec<(usize, Option<String>)> = vec![
            (1, None),
            (0, Some(String::from("word"))),
            (0, Some(String::from("0"))),
            (0, Some(String::from(""))),
            (1, None),
            (6, None),
            (7, None),
        ];
        assert_eq!(expected, field_map);
    }
}
