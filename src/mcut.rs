extern crate memchr;

use std::io::prelude::*;
use std::io::BufReader;

/// readerから読み取った文字列をcfgの設定に従ってcutする
///
/// # Arguments
/// * `reader`
/// * `writer`
/// * `cfg`    - 区切り文字や出力対象カラム番号を格納したオブジェクト
pub fn mcut<R: Read, W: Write>(reader: &mut BufReader<R>, writer: &mut W, cfg: Config) {
    let last_col = cfg.columns.len() - 1;
    let col_len: usize = cfg.columns.iter().map(|column| column.idx).max().unwrap() + 1;
    // 読み込んだ文字列を格納する配列
    let mut buf: Vec<u8> = Vec::new();
    // 区切り文字のindexを格納する配列
    let mut split: Vec<usize> = vec![0; col_len + 1];
    while reader.read_until(b'\n', &mut buf).ok().unwrap() > 0 {
        // 改行を区切り文字に置換
        let buf_last = buf.len() - 1;
        if buf[buf_last] == b'\n' {
            buf[buf_last] = cfg.delimiter;
        } else {
            buf.push(cfg.delimiter);
        }

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
        for column in (&cfg.columns)[0..last_col].iter() {
            match column {
                &Column { idx: _, default: Some(ref default), name: _} => {
                    writer.write(default).unwrap();
                    writer.write(&[cfg.delimiter]).unwrap();
                },
                &Column { idx, default: None, name: _} => {
                    let start = split[idx];
                    let end   = split[idx + 1];
                    writer.write(&buf[start..end]).unwrap();
                }
            }
        }
        match cfg.columns[last_col] {
            Column { idx: _, default: Some(ref default), name: _} => {
                writer.write(default).unwrap();
            },
            Column { idx, default: None, name: _} => {
                let start = split[idx];
                let end   = split[idx + 1] - 1;
                writer.write(&buf[start..end]).unwrap();
            }
        }
        writer.write(b"\n").unwrap();
        buf.clear();
    }
}

pub struct Config {
    pub first_line: String,
    pub delimiter: u8,
    pub field: String,
    pub columns: Vec<Column>,
}

impl Config {
    fn new(first_line: String, delimiter: u8, field: String, columns: Vec<Column>) -> Config {
        Config { first_line, delimiter, field, columns }
    }

    fn col_to_idx(col_name: &str, header: &[&str], is_start: bool) -> usize {
        if col_name.is_empty() {
            return if is_start { 0 } else { header.len() };
        }
        if let Some(idx) = col_name.trim().parse::<usize>().ok() { // カラム番号が指定されている場合
            if idx < header.len() {
                return if is_start { idx } else { idx + 1};
            }
        }
        if let Some(idx) = header.iter().position(|e| e == &col_name) {
            return if is_start { idx } else { idx + 1};
        }
        panic!("不明なフィールド: {}", col_name);
    }

    fn number_to_idx(col_name: &str, header: &[&str], is_start: bool) -> usize {
        if col_name.is_empty() {
            return if is_start { 0 } else { header.len() };
        }
        if let Some(idx) = col_name.trim().parse::<usize>().ok() {
            if idx < header.len() {
                return if is_start { idx } else { idx + 1 };
            }
        }
        panic!("不明なフィールド: {}", col_name);
    }

    fn parse_field(field: &str) -> (Option<&str>, Option<&str>, Option<Vec<u8>>) {
        let v1: Vec<&str> = field.splitn(2, ':').collect();
        let s1: &[&str] = &v1;
        match s1 {
            &[col] => {
                let v2: Vec<&str> = col.splitn(2, "..").collect();
                let s2: &[&str] = &v2;
                match s2 {
                    &[start]      => return (Some(start), None, None),
                    &[start, end] => return (Some(start), Some(end), None),
                    _             => panic!("不明なフィールド: {}", field),
                }
            }
            &[col, default] => {
                let v2: Vec<&str> = col.splitn(2, "..").collect();
                let s2: &[&str] = &v2;
                match s2 {
                    &[start]      => return (Some(start), None, Some(default.as_bytes().to_vec())),
                    &[start, end] => return (Some(start), Some(end), Some(default.as_bytes().to_vec())),
                    _             => panic!("不明なフィールド: {}", field),
                }
            }
            _ => {
                panic!("不明なフィールド: {}", field);
            }
        }
    }

    /// -f オプションをパースする
    ///
    /// # Arguments
    /// * `first_line` - ファイルの1行目
    /// * `delimiter`  - 区切り文字
    /// * `fields`      - -fオプションで指定した出力対象フィールド
    pub fn parse_field_as_number(first_line: String, delimiter: u8, fields: String) -> Self {
        let cols: Vec<&str> = first_line.split(char::from(delimiter)).collect();
        let mut columns: Vec<Column> = Vec::new();
        for field in fields.split(',') {
            match Self::parse_field(field) {
                (Some(start), None, None) => { // 範囲指定なし, デフォルト値なし
                    let idx = Self::number_to_idx(start, &cols, true);
                    columns.push(Column::new(idx, None, Vec::new()));
                }
                (Some(_), None, Some(default)) => { // 範囲指定なし, デフォルト値あり
                    columns.push(Column::new(0, Some(default), Vec::new()));
                }
                (Some(start), Some(end), None) => { // 範囲指定あり, デフォルト値なし
                    let start = Self::number_to_idx(start, &cols, true);
                    let end   = Self::number_to_idx(end, &cols, false);
                    for idx in start..end {
                        columns.push(Column::new(idx, None, Vec::new()));
                    }
                }
                (Some(start), Some(end), default) => { // 範囲指定あり, デフォルト値あり
                    let start = Self::number_to_idx(start, &cols, true);
                    let end   = Self::number_to_idx(end, &cols, false);
                    for _ in start..end {
                        columns.push(Column::new(0, default.clone(), Vec::new()));
                    }
                }
                (_,_,_) => panic!("不正な形式のフィールドです: {}", field)
            }
        }
        Config::new(first_line, delimiter, fields, columns)
    }

    /// -F オプションをパースする
    ///
    /// # Arguments
    /// * `first_line`    - ファイルの1行目のヘッダ文字列
    /// * `delimiter` - 区切り文字
    /// * `fields`     - -Fオプションで指定した出力対象フィールド
    pub fn parse_field_as_name(first_line: String, delimiter: u8, fields: String) -> Self {
        let cols: Vec<&str> = first_line.split(char::from(delimiter)).collect();
        let mut columns: Vec<Column> = Vec::new();
        for field in fields.split(',') {
            match Self::parse_field(field) {
                (Some(start), None, None) => { // 範囲指定なし, デフォルト値なし
                    let idx = Self::col_to_idx(start, &cols, true);
                    columns.push(Column::new(idx, None, cols[idx].as_bytes().to_vec()));
                }
                (Some(start), None, Some(default)) => { // 範囲指定なし, デフォルト値あり
                    columns.push(Column::new(0, Some(default), start.as_bytes().to_vec()));
                }
                (Some(start), Some(end), None) => { // 範囲指定あり, デフォルト値なし
                    let start = Self::col_to_idx(start, &cols, true);
                    let end   = Self::col_to_idx(end, &cols, false);
                    for idx in start..end {
                        columns.push(Column::new(idx, None, cols[idx].as_bytes().to_vec()));
                    }
                }
                (Some(start), Some(end), default) => { // 範囲指定あり, デフォルト値あり
                    let start = Self::col_to_idx(start, &cols, true);
                    let end   = Self::col_to_idx(end, &cols, false);
                    for idx in start..end {
                        columns.push(Column::new(0, default.clone(), cols[idx].as_bytes().to_vec()));
                    }
                }
                (_,_,_) => panic!("不正な形式のフィールドです: {}", field)
            }
        }
        Config::new(first_line, delimiter, fields, columns)
    }

    /// first_lineをヘッダとして出力する
    ///
    /// # Arguments
    /// * `writer` - ヘッダ行を書き込むwriter
    pub fn write_header<W: Write>(&self, writer: &mut W) {
        let mut buf: Vec<u8> = Vec::new();
        for column in self.columns.iter() {
            buf.extend_from_slice(&column.name);
            buf.push(self.delimiter);
        }
        buf.pop();
        buf.push(b'\n');
        writer.write(&buf).unwrap();
    }

    /// first_lineを以降の行と同様にパースして出力する。
    ///
    /// # Arguments
    /// * `writer` - ヘッダ行を書き込むwriter
    pub fn write_first_line<W: Write>(&self, writer: &mut W) {
        let cols: Vec<&[u8]> = self.first_line.as_bytes().split(|e| e == &self.delimiter).collect();
        let mut buf: Vec<u8> = Vec::new();
        for column in self.columns.iter() {
            if let Some(ref default) = column.default {
                buf.extend_from_slice(default);
            } else {
                buf.extend_from_slice(cols[column.idx])
            }
            buf.push(self.delimiter);
        }
        buf.pop();
        buf.push(b'\n');
        writer.write(&buf).unwrap();
    }
}

#[derive(PartialEq,Debug)]
pub struct Column {
    pub idx: usize,
    pub default: Option<Vec<u8>>,
    pub name: Vec<u8>,
}

impl Column {
    pub fn new(idx: usize, default: Option<Vec<u8>>, name: Vec<u8>) -> Self {
        Column { idx, default, name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_col_to_idx_1() {
        let header = vec!["col0", "col1", "col2"];
        let col_name_list = vec!["", "col1", "2"];
        let expected = [0, 1, 2];
        for (i, col_name) in col_name_list.iter().enumerate() {
            assert_eq!(expected[i], Config::col_to_idx(col_name, &header, true));
        }
        let expected = [3, 2, 3];
        for (i, col_name) in col_name_list.iter().enumerate() {
            assert_eq!(expected[i], Config::col_to_idx(col_name, &header, false));
        }
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: col3")]
    fn test_col_to_idx_2() {
        let header = vec!["col0", "col1", "col2"];
        let col_name = "col3";
        Config::col_to_idx(col_name, &header, true);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: 100")]
    fn test_col_to_idx_3() {
        let header = vec!["col0", "col1", "col2"];
        let col_name = "100";
        Config::col_to_idx(col_name, &header, true);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: -100")]
    fn test_col_to_idx_4() {
        let header = vec!["col0", "col1", "col2"];
        let col_name = "-100";
        Config::col_to_idx(col_name, &header, true);
    }

    #[test]
    fn test_number_to_idx_1() {
        let header = vec!["col0", "col1", "col2"];
        let col_name_list = vec!["", "2"];
        let expected = [0, 2];
        for (i, col_name) in col_name_list.iter().enumerate() {
            assert_eq!(expected[i], Config::number_to_idx(col_name, &header, true));
        }
        let expected = [3, 3];
        for (i, col_name) in col_name_list.iter().enumerate() {
            assert_eq!(expected[i], Config::number_to_idx(col_name, &header, false));
        }
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: 100")]
    fn test_number_to_idx_2() {
        let header = vec!["col0", "col1", "col2"];
        let col_name = "100";
        Config::number_to_idx(col_name, &header, true);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: col0")]
    fn test_number_to_idx_3() {
        let header = vec!["col0", "col1", "col2"];
        let col_name = "col0";
        Config::number_to_idx(col_name, &header, true);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: -100")]
    fn test_number_to_idx_4() {
        let header = vec!["col0", "col1", "col2"];
        let col_name = "-100";
        Config::number_to_idx(col_name, &header, true);
    }

    #[test]
    fn test_parse_field_1() {
        let fields = vec![
            ""     , "col0"     , "col0..col3"     , "col0.."     , "..col3"     , "..",
            ":def1", "col0:def1", "col0..col3:def1", "col0..:def1", "..col3:def1", "..:def1",
        ];
        let expected: Vec<(Option<&str>, Option<&str>, Option<Vec<u8>>)> = vec![
            (Some("")    , None        , None),
            (Some("col0"), None        , None),
            (Some("col0"), Some("col3"), None),
            (Some("col0"), Some("")    , None),
            (Some("")    , Some("col3"), None),
            (Some("")    , Some("")    , None),
            (Some("")    , None        , Some(b"def1".to_vec())),
            (Some("col0"), None        , Some(b"def1".to_vec())),
            (Some("col0"), Some("col3"), Some(b"def1".to_vec())),
            (Some("col0"), Some("")    , Some(b"def1".to_vec())),
            (Some("")    , Some("col3"), Some(b"def1".to_vec())),
            (Some("")    , Some("")    , Some(b"def1".to_vec())),

        ];
        for (i, field) in fields.iter().enumerate() {
            assert_eq!(expected[i], Config::parse_field(field));
        }
    }

    #[test]
    fn test_parse_field_as_number_1() {
        let field = String::from("2,4,6,2:,3:foo,:0,5");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        let cfg = Config::parse_field_as_number(header, b',', field);
        let expected: Vec<Column> = vec![
            Column::new(2, None                 , Vec::new()),
            Column::new(4, None                 , Vec::new()),
            Column::new(6, None                 , Vec::new()),
            Column::new(0, Some(b"".to_vec())   , Vec::new()),
            Column::new(0, Some(b"foo".to_vec()), Vec::new()),
            Column::new(0, Some(b"0".to_vec())  , Vec::new()),
            Column::new(5, None                 , Vec::new()),
        ];
        assert_eq!(expected, cfg.columns);
    }

    #[test]
    fn test_parse_field_as_number_2() {
        let field = String::from("1..2,..3,3..,..");
        let header = String::from("col0,col1,col2,col3,col4,col5");
        let cfg = Config::parse_field_as_number(header, b',', field);
        let expected: Vec<Column> = vec![
            Column::new(1, None , Vec::new()),
            Column::new(2, None , Vec::new()),
            Column::new(0, None , Vec::new()),
            Column::new(1, None , Vec::new()),
            Column::new(2, None , Vec::new()),
            Column::new(3, None , Vec::new()),
            Column::new(3, None , Vec::new()),
            Column::new(4, None , Vec::new()),
            Column::new(5, None , Vec::new()),
            Column::new(0, None , Vec::new()),
            Column::new(1, None , Vec::new()),
            Column::new(2, None , Vec::new()),
            Column::new(3, None , Vec::new()),
            Column::new(4, None , Vec::new()),
            Column::new(5, None , Vec::new()),
        ];
        assert_eq!(expected, cfg.columns);
    }

    #[test]
    fn test_parse_field_as_number_3() {
        let field = String::from("1..2:def1,..3:def2,3..:def3,..:def4");
        let header = String::from("col0,col1,col2,col3,col4,col5");
        let cfg = Config::parse_field_as_number(header, b',', field);
        let expected: Vec<Column> = vec![
            Column::new(0, Some(b"def1".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def1".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def2".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def2".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def2".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def2".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def3".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def3".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def3".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def4".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def4".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def4".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def4".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def4".to_vec()) , Vec::new()),
            Column::new(0, Some(b"def4".to_vec()) , Vec::new()),
        ];
        assert_eq!(expected, cfg.columns);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: 100")]
    fn test_parse_field_as_number_4() {
        // 存在しないカラムが指定されている: 100
        let field = String::from("2,4,6,2:,100,3:foo,:0,5");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        Config::parse_field_as_number(header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: title")]
    fn test_parse_field_as_number_5() {
        // 数値でないカラムが指定されている: title
        let field = String::from("2,4,6,2:,3:foo,:0,5,title");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        Config::parse_field_as_number(header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: -1")]
    fn test_parse_field_as_number_6() {
        let field = String::from("-1..");
        let header = String::from("col0,col1,col2");
        Config::parse_field_as_number(header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: 50")]
    fn test_parse_field_as_number_7() {
        let field = String::from("..50");
        let header = String::from("col0,col1,col2");
        Config::parse_field_as_number(header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: title")]
    fn test_parse_field_as_number_8() {
        let field = String::from("title..50");
        let header = String::from("col0,col1,col2");
        Config::parse_field_as_number(header, b',', field);
    }

    #[test]
    fn test_parse_field_as_name_1() {
        let field = String::from("title,field:word,src:0,kana:,title,narrow1,narrow2");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        let cfg = Config::parse_field_as_name(header, b',', field);
        let expected: Vec<Column> = vec![
            Column::new(1 , None                  , b"title".to_vec()),
            Column::new(0 , Some(b"word".to_vec()), b"field".to_vec()),
            Column::new(0 , Some(b"0".to_vec())   , b"src".to_vec()),
            Column::new(0 , Some(b"".to_vec())    , b"kana".to_vec()),
            Column::new(1 , None                  , b"title".to_vec()),
            Column::new(6 , None                  , b"narrow1".to_vec()),
            Column::new(7 , None                  , b"narrow2".to_vec()),
        ];
        assert_eq!(expected, cfg.columns);
    }

    #[test]
    fn test_parse_field_as_name_2() {
        let field = String::from("1..2,..3,3..,..,col1..col2,..col3,col3..");
        let header = String::from("col0,col1,col2,col3,col4,col5");
        let cfg = Config::parse_field_as_name(header, b',', field);
        let expected: Vec<Column> = vec![
            Column::new(1, None , b"col1".to_vec()),
            Column::new(2, None , b"col2".to_vec()),
            Column::new(0, None , b"col0".to_vec()),
            Column::new(1, None , b"col1".to_vec()),
            Column::new(2, None , b"col2".to_vec()),
            Column::new(3, None , b"col3".to_vec()),
            Column::new(3, None , b"col3".to_vec()),
            Column::new(4, None , b"col4".to_vec()),
            Column::new(5, None , b"col5".to_vec()),
            Column::new(0, None , b"col0".to_vec()),
            Column::new(1, None , b"col1".to_vec()),
            Column::new(2, None , b"col2".to_vec()),
            Column::new(3, None , b"col3".to_vec()),
            Column::new(4, None , b"col4".to_vec()),
            Column::new(5, None , b"col5".to_vec()),
            Column::new(1, None , b"col1".to_vec()),
            Column::new(2, None , b"col2".to_vec()),
            Column::new(0, None , b"col0".to_vec()),
            Column::new(1, None , b"col1".to_vec()),
            Column::new(2, None , b"col2".to_vec()),
            Column::new(3, None , b"col3".to_vec()),
            Column::new(3, None , b"col3".to_vec()),
            Column::new(4, None , b"col4".to_vec()),
            Column::new(5, None , b"col5".to_vec()),
        ];
        assert_eq!(expected, cfg.columns);
    }

    #[test]
    fn test_parse_field_as_name_3() {
        let field = String::from("1..2:def1,..3:def2,3..:def3,..:def4,col1..col2:def5,..col3:def6,col3..:def7");
        let header = String::from("col0,col1,col2,col3,col4,col5");
        let cfg = Config::parse_field_as_name(header, b',', field);
        let expected: Vec<Column> = vec![
            Column::new(0, Some(b"def1".to_vec()) , b"col1".to_vec()),
            Column::new(0, Some(b"def1".to_vec()) , b"col2".to_vec()),
            Column::new(0, Some(b"def2".to_vec()) , b"col0".to_vec()),
            Column::new(0, Some(b"def2".to_vec()) , b"col1".to_vec()),
            Column::new(0, Some(b"def2".to_vec()) , b"col2".to_vec()),
            Column::new(0, Some(b"def2".to_vec()) , b"col3".to_vec()),
            Column::new(0, Some(b"def3".to_vec()) , b"col3".to_vec()),
            Column::new(0, Some(b"def3".to_vec()) , b"col4".to_vec()),
            Column::new(0, Some(b"def3".to_vec()) , b"col5".to_vec()),
            Column::new(0, Some(b"def4".to_vec()) , b"col0".to_vec()),
            Column::new(0, Some(b"def4".to_vec()) , b"col1".to_vec()),
            Column::new(0, Some(b"def4".to_vec()) , b"col2".to_vec()),
            Column::new(0, Some(b"def4".to_vec()) , b"col3".to_vec()),
            Column::new(0, Some(b"def4".to_vec()) , b"col4".to_vec()),
            Column::new(0, Some(b"def4".to_vec()) , b"col5".to_vec()),
            Column::new(0, Some(b"def5".to_vec()) , b"col1".to_vec()),
            Column::new(0, Some(b"def5".to_vec()) , b"col2".to_vec()),
            Column::new(0, Some(b"def6".to_vec()) , b"col0".to_vec()),
            Column::new(0, Some(b"def6".to_vec()) , b"col1".to_vec()),
            Column::new(0, Some(b"def6".to_vec()) , b"col2".to_vec()),
            Column::new(0, Some(b"def6".to_vec()) , b"col3".to_vec()),
            Column::new(0, Some(b"def7".to_vec()) , b"col3".to_vec()),
            Column::new(0, Some(b"def7".to_vec()) , b"col4".to_vec()),
            Column::new(0, Some(b"def7".to_vec()) , b"col5".to_vec()),
        ];
        assert_eq!(expected, cfg.columns);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: not_exists")]
    fn test_parse_field_as_name_4() {
        // 存在しないカラムが指定されている: not_exists
        let field = String::from("title,field:word,src:0,kana:,title,narrow1,not_exists,narrow2");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        Config::parse_field_as_name(header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: -1")]
    fn test_parse_field_as_name_5() {
        let field = String::from("-1..");
        let header = String::from("col0,col1,col2");
        Config::parse_field_as_name(header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: 50")]
    fn test_parse_field_as_name_7() {
        let field = String::from("..50");
        let header = String::from("col0,col1,col2");
        Config::parse_field_as_name(header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: title")]
    fn test_parse_field_as_name_6() {
        let field = String::from("title..50");
        let header = String::from("col0,col1,col2");
        Config::parse_field_as_name(header, b',', field);
    }
}
