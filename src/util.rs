/// seq を delimiter で連結した文字列に整形する
///
/// # Arguments
/// * `delimiter` - 区切り文字
/// * `seq`       - 連結対象の配列
pub fn join<T: std::fmt::Display>(delimiter: char, seq: &[T]) -> String {
    use std::fmt::Write;
    let mut ret = String::new();
    for e in seq {
        write!(ret, "{}", e).unwrap();
        ret.push(delimiter);
    }
    ret.pop();
    return ret;
}

# [cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() {
        assert_eq!("1;2;3;4", join(';', &[1, 2, 3, 4]));
    }
}
