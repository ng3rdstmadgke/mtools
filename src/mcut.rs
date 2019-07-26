extern crate memchr;

use std::io::prelude::*;
use std::io::BufReader;

/// -f オプションをパースする
///
/// # Arguments
/// * `header`    - ファイルの1行目
/// * `delimiter` - 区切り文字
/// * `field`     - -fオプションで指定した出力対象フィールド
pub fn fmap_from_col_number(header: &str, delimiter: u8, field: String) -> Vec<(usize, Option<Vec<u8>>, Vec<u8>)> {
    let cols: Vec<&str> = header.split(char::from(delimiter)).collect();
    let fmap: Vec<(usize, Option<Vec<u8>>, Vec<u8>)> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                return (0, Some(split[1].as_bytes().to_vec()), split[0].as_bytes().to_vec());
            } else if split.len() == 1{
                if let Some(idx) = e.trim().parse::<usize>().ok() {
                    if idx < cols.len() {
                        return (idx, None, e.as_bytes().to_vec());
                    }
                }
            }
            panic!("不明なフィールド: {}", e);
        }).collect();
    fmap
}

/// -F オプションをパースする
///
/// # Arguments
/// * `header`    - ファイルの1行目のヘッダ文字列
/// * `delimiter` - 区切り文字
/// * `field`     - -Fオプションで指定した出力対象フィールド
pub fn fmap_from_col_name(header: &str, delimiter: u8, field: String) -> Vec<(usize, Option<Vec<u8>>, Vec<u8>)> {
    let cols: Vec<&str> = header.split(char::from(delimiter)).collect();
    let fmap: Vec<(usize, Option<Vec<u8>>, Vec<u8>)> = field.split(',')
        .map(|e| {
            let split: Vec<&str> = e.splitn(2, ':').collect();
            if split.len() == 2 {
                // デフォルト値が存在する場合
                if let Some(idx) = e.trim().parse::<usize>().ok() { // カラム番号が指定されている場合
                    if idx < cols.len() {
                        return (
                            idx,
                            Some(split[1].as_bytes().to_vec()),
                            cols[idx].as_bytes().to_vec()
                        );
                    }
                } else { // カラム名が指定されている場合
                    // デフォルト値が存在してかつカラム名が指定されている場合はidxにusizeのmax値を格納する
                    let idx = cols.iter().position(|c| c == &split[0]).unwrap_or(0);
                    return (
                        idx,
                        Some(split[1].as_bytes().to_vec()),
                        split[0].as_bytes().to_vec()
                    );
                }
            }  else if split.len() == 1 {
                // デフォルト値が存在しない場合
                if let Some(idx) = e.trim().parse::<usize>().ok() { // カラム番号が指定されている場合
                    if idx < cols.len() {
                        return (
                            idx,
                            None,
                            cols[idx].as_bytes().to_vec()
                        );
                    }
                } else { // カラム名が指定されている場合
                    if let Some(idx) = cols.iter().position(|c| c == &e) {
                        return (
                            idx,
                            None,
                            e.as_bytes().to_vec()
                        );
                    }
                }
            }
            panic!("不明なフィールド: {}", e);
        }).collect();
    fmap
}

/// -F オプションを指定した場合にヘッダ行を出力するメソッド
///
/// # Arguments
/// * `writer` - ヘッダ行を書き込むwriter
/// * `cfg`    - 区切り文字や出力対象カラム番号を格納したオブジェクト
pub fn write_header<W: Write>(writer: &mut W, cfg: &Config) {
    let mut buf: Vec<u8> = Vec::new();
    for (_, _, col_name) in cfg.fmap.iter() {
        buf.extend_from_slice(col_name);
        buf.push(cfg.delimiter);
    }
    buf.pop();
    buf.push(b'\n');
    writer.write(&buf).unwrap();
}

/// 1行をパースして出力するメソッド
/// -f オプションを指定した場合に、最初の1行目を出力するのに利用する
///
/// # Arguments
/// * `line`   - 1行目の文字列
/// * `writer` - ヘッダ行を書き込むwriter
/// * `cfg`    - 区切り文字や出力対象カラム番号を格納したオブジェクト
pub fn mcut_line<W: Write>(line: &[u8], writer: &mut W, cfg: &Config) {
    let cols: Vec<&[u8]> = line.split(|e| e == &cfg.delimiter).collect();
    let mut buf: Vec<u8> = Vec::new();
    for (i, default, _) in cfg.fmap.iter() {
        if let Some(default) = default {
            buf.extend_from_slice(default);
        } else {
            buf.extend_from_slice(cols[*i])
        }
        buf.push(cfg.delimiter);
    }
    buf.pop();
    buf.push(b'\n');
    writer.write(&buf).unwrap();
}


/// readerから読み取った文字列をcfgの設定に従ってcutする
///
/// # Arguments
/// * `reader`
/// * `writer`
/// * `cfg`    - 区切り文字や出力対象カラム番号を格納したオブジェクト
pub fn mcut<R: Read, W: Write>(reader: &mut BufReader<R>, writer: &mut W, cfg: Config) {
    let last_col = cfg.fmap.len() - 1;
    let col_len: usize = *cfg.fmap.iter().map(|(i, _, _)| i).max().unwrap() + 1;
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
        for f in (&cfg.fmap)[0..last_col].iter() {
            match f {
                &(_, Some(ref default), _) => {
                    writer.write(default).unwrap();
                    writer.write(&[cfg.delimiter]).unwrap();
                },
                &(col_idx, None, _) => {
                    let start = split[col_idx];
                    let end   = split[col_idx + 1];
                    writer.write(&buf[start..end]).unwrap();
                }
            }
        }
        match cfg.fmap[last_col] {
            (_, Some(ref default), _) => {
                writer.write(default).unwrap();
            },
            (col_idx, None, _) => {
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
    pub fmap: Vec<(usize, Option<Vec<u8>>, Vec<u8>)>, // field_idx, default_value, col_name
}

impl Config {
    pub fn new(delimiter: u8, fmap: Vec<(usize, Option<Vec<u8>>, Vec<u8>)>) -> Self {
        Config { delimiter, fmap }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmap_from_col_number_1() {
        let field = String::from("2,4,6,2:,3:foo,:0,5");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        let fmap = fmap_from_col_number(&header, b',', field);
        let expected: Vec<(usize, Option<Vec<u8>>, Vec<u8>)> = vec![
            (2, None                 , b"2".to_vec()),
            (4, None                 , b"4".to_vec()),
            (6, None                 , b"6".to_vec()),
            (0, Some(b"".to_vec())   , b"2".to_vec()),
            (0, Some(b"foo".to_vec()), b"3".to_vec()),
            (0, Some(b"0".to_vec())  , b"".to_vec()),
            (5, None                 , b"5".to_vec()),
        ];
        assert_eq!(expected, fmap);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: 100")]
    fn test_fmap_from_col_number_2() {
        // 存在しないカラムが指定されている: 100
        let field = String::from("2,4,6,2:,100,3:foo,:0,5");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        fmap_from_col_number(&header, b',', field);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: title")]
    fn test_fmap_from_col_number_3() {
        // 数値でないカラムが指定されている: title
        let field = String::from("2,4,6,2:,3:foo,:0,5,title");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        fmap_from_col_number(&header, b',', field);
    }

    #[test]
    fn test_fmap_from_col_name_1() {
        let field = String::from("title,field:word,src:0,kana:,title,narrow1,narrow2");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        let fmap = fmap_from_col_name(&header, b',', field);
        let expected: Vec<(usize, Option<Vec<u8>>, Vec<u8>)> = vec![
            (1 , None                  , b"title".to_vec()),
            (0 , Some(b"word".to_vec()), b"field".to_vec()),
            (0 , Some(b"0".to_vec())   , b"src".to_vec()),
            (0 , Some(b"".to_vec())    , b"kana".to_vec()),
            (1 , None                  , b"title".to_vec()),
            (6 , None                  , b"narrow1".to_vec()),
            (7 , None                  , b"narrow2".to_vec()),
        ];
        assert_eq!(expected, fmap);
    }

    #[test]
    #[should_panic(expected = "不明なフィールド: not_exists")]
    fn test_fmap_from_col_name_2() {
        // 存在しないカラムが指定されている: not_exists
        let field = String::from("title,field:word,src:0,kana:,title,narrow1,not_exists,narrow2");
        let header = String::from("itemid,title,url,desc,keyword1,keyword2,narrow1,narrow2,data1,data2");
        fmap_from_col_name(&header, b',', field);
    }
}
