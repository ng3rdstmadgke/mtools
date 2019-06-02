use std::io::prelude::*;
use std::io::BufReader;

pub fn get_field_map_1(field: String) -> (Vec<usize>, Vec<Option<String>>) {
    let field_map: Vec<usize> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                0
            } else {
                if let Some(f) = e.trim().parse::<usize>().ok() {
                    f
                } else {
                    panic!("不明なフィールド: {}", e);
                }
            }
        }).collect();

    let default_map: Vec<Option<String>> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                Some(split[1].to_string())
            } else {
                None
            }
        }).collect();
    (field_map, default_map)
}

pub fn get_field_map_2(header: &str, delimiter: char, field: String) -> (Vec<usize>, Vec<Option<String>>) {
    let cols: Vec<&str> = header.split(delimiter).collect();
    let field_map: Vec<usize> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                cols.iter().position(|c| c == &split[0]).unwrap_or(0)
            } else {
                cols.iter().position(|c| c == &e).expect(&format!("不明なフィールド: {}", e))
            } }).collect();

    let default_map: Vec<Option<String>> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                Some(split[1].to_string())
            } else {
                None
            }
        }).collect();
    (field_map, default_map)
}

pub fn ecut<R: Read, W: Write>(reader: &mut BufReader<R>, writer: &mut W, cfg: Config) {
    let max: usize = *cfg.field_map.iter().max().unwrap();
    let mut row: String = String::new();
    for result in reader.lines() {
        let line = result.ok().unwrap();
        // 必要なところまで読み込む
        let mut cols: Vec<&str> = Vec::with_capacity(max);
        for (i, col) in line.split(cfg.delimiter).enumerate() {
            if i > max {
                break;
            }
            cols.push(col);
        }
        // デフォルト値がある場合はデフォルト値を出力する
        for i in 0..(cfg.field_map.len()) {
            if let Some(ref default) = cfg.default_map[i] {
                row.push_str(default);
            } else {
                row.push_str(cols[cfg.field_map[i]]); } row.push(cfg.delimiter); } row.pop(); row.push('\n'); writer.write(row.as_bytes()).unwrap();
        row.clear();
    }
}

pub struct Config {
    pub delimiter: char,
    pub field_map: Vec<usize>,
    pub default_map: Vec<Option<String>>,
}

impl Config {
    pub fn new(delimiter: char, field_map: Vec<usize>, default_map: Vec<Option<String>>) -> Self {
        Config { delimiter, field_map, default_map}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_field_map_1() {
        let field = String::from("2,4,6,2:,3:foo,:0,5");
        let (f, d) = get_field_map_1(field);
        let expected_f: Vec<usize> = vec![2,4,6,0,0,0,5];
        assert_eq!(expected_f, f);
        let expected_d: Vec<Option<String>> = vec![
            None,
            None,
            None,
            Some(String::from("")),
            Some(String::from("foo")),
            Some(String::from("0")),
            None,
        ];
        assert_eq!(expected_d, d);
    }

    #[test]
    fn test_get_field_map_2() {
        let field = String::from("title,field:word,src:0,kana:,title,narrow1,narrow2");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        let (f, d) = get_field_map_2(&header, ',', field);
        let expected_f: Vec<usize> = vec![1,0,0,0,1,6,7];
        assert_eq!(expected_f, f);
        let expected_d: Vec<Option<String>> = vec![
            None,
            Some(String::from("word")),
            Some(String::from("0")),
            Some(String::from("")),
            None,
            None,
            None,
        ];
        assert_eq!(expected_d, d);
    }
}
