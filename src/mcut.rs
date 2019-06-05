extern crate memchr;

use std::io::prelude::*;
use std::io::BufReader;

pub fn get_field_map_1(field: String) -> Vec<(usize, Option<Vec<u8>>)> {
    let field_map: Vec<(usize, Option<Vec<u8>>)> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                (0, Some(split[1].as_bytes().to_vec()))
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

pub fn get_field_map_2(header: &str, delimiter: u8, field: String) -> Vec<(usize, Option<Vec<u8>>)> {
    let cols: Vec<&str> = header.split(char::from(delimiter)).collect();
    let field_map: Vec<(usize, Option<Vec<u8>>)> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                let col = cols.iter().position(|c| c == &split[0]).unwrap_or(0);
                (col, Some(split[1].as_bytes().to_vec()))
            } else {
                let col = cols.iter().position(|c| c == &e).expect(&format!("不明なフィールド: {}", e));
                (col, None)
            }
        }).collect();
    field_map
}

pub fn mcut<R: Read, W: Write>(reader: &mut BufReader<R>, writer: &mut W, cfg: Config) {
    let last_col = cfg.field_map.len() - 1;
    let col_len: usize = *cfg.field_map.iter().map(|(i, _)|i).max().unwrap() + 1;
    // 読み込んだ文字列を格納する配列
    let mut buf: Vec<u8> = Vec::new();
    // 区切り文字のindexを格納する配列
    let mut split: Vec<usize> = vec![0; col_len + 1];
    while reader.read_until(b'\n', &mut buf).ok().unwrap() > 0 {
        // 改行を区切り文字に置換
        let buf_last = buf.len() - 1;
        buf[buf_last] = cfg.delimiter;

        // 必要なところまで読み込む
        for (i, position) in memchr::memchr_iter(cfg.delimiter, &buf).enumerate() {
            if i < col_len {
                // 区切り文字も含んだindex
                split[i + 1] = position + 1;
            } else {
                break;
            }
        }

        // 書き込み処理
        for f in (&cfg.field_map)[0..last_col].iter() {
            match f {
                &(_, Some(ref default)) => {
                    writer.write(default).unwrap();
                    writer.write(&[cfg.delimiter]).unwrap();
                },
                &(col_idx, None) => {
                    let start = split[col_idx];
                    let end   = split[col_idx + 1];
                    writer.write(&buf[start..end]).unwrap();
                }
            }
        }
        match cfg.field_map[last_col] {
            (_, Some(ref default)) => {
                writer.write(default).unwrap();
            },
            (col_idx, None) => {
                let start = split[col_idx];
                let end   = split[col_idx + 1] - 1;
                writer.write(&buf[start..end]).unwrap();
            }
        }
        writer.write(b"\n").unwrap();
        buf.clear();
    }
}

pub struct Config {
    pub delimiter: u8,
    pub field_map: Vec<(usize, Option<Vec<u8>>)>, // field_idx, default_value
}

impl Config {
    pub fn new(delimiter: u8, field_map: Vec<(usize, Option<Vec<u8>>)>) -> Self {
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
        let expected: Vec<(usize, Option<Vec<u8>>)> = vec![
            (2, None),
            (4, None),
            (6, None),
            (0, Some(b"".to_vec())),
            (0, Some(b"foo".to_vec())),
            (0, Some(b"0".to_vec())),
            (5, None),
        ];
        assert_eq!(expected, field_map);
    }

    #[test]
    fn test_get_field_map_2() {
        let field = String::from("title,field:word,src:0,kana:,title,narrow1,narrow2");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        let field_map = get_field_map_2(&header, b',', field);
        let expected: Vec<(usize, Option<Vec<u8>>)> = vec![
            (1, None),
            (0, Some(b"word".to_vec())),
            (0, Some(b"0".to_vec())),
            (0, Some(b"".to_vec())),
            (1, None),
            (6, None),
            (7, None),
        ];
        assert_eq!(expected, field_map);
    }
}
