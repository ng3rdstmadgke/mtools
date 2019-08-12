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

    /// -f オプションをパースする
    ///
    /// # Arguments
    /// * `first_line` - ファイルの1行目
    /// * `delimiter`  - 区切り文字
    /// * `field`      - -fオプションで指定した出力対象フィールド
    pub fn parse_field_as_number(first_line: String, delimiter: u8, field: String) -> Self {
        let cols: Vec<&str> = first_line.split(char::from(delimiter)).collect();
        let columns: Vec<Column> = field.split(',')
            .map(|e| {
                let split: Vec<&str> = e.splitn(2, ':').collect();
                if split.len() == 2 {
                    // default値が存在する場合
                    return Column::new(0, Some(split[1].as_bytes().to_vec()), split[0].as_bytes().to_vec());
                } else if split.len() == 1{
                    // default値が存在しない場合
                    if let Some(idx) = e.trim().parse::<usize>().ok() {
                        if idx < cols.len() {
                            return Column::new(idx, None, e.as_bytes().to_vec());
                        }
                    }
                }
                panic!("不明なフィールド: {}", e);
            }).collect();
        Config::new(first_line, delimiter, field, columns)
    }

    /// -F オプションをパースする
    ///
    /// # Arguments
    /// * `first_line`    - ファイルの1行目のヘッダ文字列
    /// * `delimiter` - 区切り文字
    /// * `field`     - -Fオプションで指定した出力対象フィールド
    pub fn parse_field_as_name(first_line: String, delimiter: u8, field: String) -> Self {
        let cols: Vec<&str> = first_line.split(char::from(delimiter)).collect();
        let columns: Vec<Column> = field.split(',')
            .map(|e| {
                let split: Vec<&str> = e.splitn(2, ':').collect();
                if split.len() == 2 {
                    // デフォルト値が存在する場合
                    if let Some(idx) = e.trim().parse::<usize>().ok() { // カラム番号が指定されている場合
                        if idx < cols.len() {
                            return Column::new(idx, Some(split[1].as_bytes().to_vec()), cols[idx].as_bytes().to_vec());
                        }
                    } else { // カラム名が指定されている場合
                        // デフォルト値が存在してかつカラム名が指定されている場合はidxにusizeのmax値を格納する
                        let idx = cols.iter().position(|c| c == &split[0]).unwrap_or(0);
                        return Column::new(idx, Some(split[1].as_bytes().to_vec()), split[0].as_bytes().to_vec());
                    }
                }  else if split.len() == 1 {
                    // デフォルト値が存在しない場合
                    if let Some(idx) = e.trim().parse::<usize>().ok() { // カラム番号が指定されている場合
                        if idx < cols.len() {
                            return Column::new(idx, None, cols[idx].as_bytes().to_vec());
                        }
                    } else { // カラム名が指定されている場合
                        if let Some(idx) = cols.iter().position(|c| c == &e) {
                            return Column::new(idx, None, e.as_bytes().to_vec());
                        }
                    }
                }
                panic!("不明なフィールド: {}", e);
            }).collect();
        Config::new(first_line, delimiter, field, columns)
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
    fn test_fmap_from_col_number_1() {
        let field = String::from("2,4,6,2:,3:foo,:0,5");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        let cfg = Config::parse_field_as_number(header, b',', field);
        let expected: Vec<Column> = vec![
            Column::new(2, None                 , b"2".to_vec()),
            Column::new(4, None                 , b"4".to_vec()),
            Column::new(6, None                 , b"6".to_vec()),
            Column::new(0, Some(b"".to_vec())   , b"2".to_vec()),
            Column::new(0, Some(b"foo".to_vec()), b"3".to_vec()),
            Column::new(0, Some(b"0".to_vec())  , b"".to_vec()),
            Column::new(5, None                 , b"5".to_vec()),
        ];
        assert_eq!(expected, cfg.columns);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: 100")]
    fn test_fmap_from_col_number_2() {
        // 存在しないカラムが指定されている: 100
        let field = String::from("2,4,6,2:,100,3:foo,:0,5");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        Config::parse_field_as_number(header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: title")]
    fn test_fmap_from_col_number_3() {
        // 数値でないカラムが指定されている: title
        let field = String::from("2,4,6,2:,3:foo,:0,5,title");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        Config::parse_field_as_number(header, b',', field);
    }

    #[test]
    fn test_fmap_from_col_name_1() {
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
    #[should_panic(expected = "不明なフィールド: not_exists")]
    fn test_fmap_from_col_name_2() {
        // 存在しないカラムが指定されている: not_exists
        let field = String::from("title,field:word,src:0,kana:,title,narrow1,not_exists,narrow2");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        Config::parse_field_as_name(header, b',', field);
    }
}
