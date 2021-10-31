use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    iter::{FromIterator, IntoIterator},
    ops::Deref,
};

use log::trace;

/// A multi-map from header name to a set of values for that header
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Headers {
    /// A multi-map from header name to a set of values for that header
    pub inner: HashMap<String, HashSet<String>>,
}

impl FromIterator<(String, String)> for Headers {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        let mut inner = HashMap::default();
        for (k, v) in iter {
            let entry = inner.entry(k).or_insert_with(HashSet::default);
            entry.insert(v);
        }
        Headers { inner }
    }
}

impl<'a> FromIterator<(&'a String, &'a String)> for Headers {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'a String, &'a String)>,
    {
        let mut inner = HashMap::default();
        for (k, v) in iter {
            let k = k.to_string();
            let v = v.to_string();
            let entry = inner.entry(k).or_insert_with(HashSet::default);
            entry.insert(v);
        }
        Headers { inner }
    }
}

impl<'a> FromIterator<&'a (&'a String, &'a String)> for Headers {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a (&'a String, &'a String)>,
    {
        let mut inner = HashMap::default();
        for (k, v) in iter {
            let k = k.to_string();
            let v = v.to_string();
            let entry = inner.entry(k).or_insert_with(HashSet::default);
            entry.insert(v);
        }
        Headers { inner }
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for Headers {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut inner = HashMap::default();
        for (k, v) in iter {
            let k = k.to_string();
            let v = v.to_string();
            let entry = inner.entry(k).or_insert_with(HashSet::default);
            entry.insert(v);
        }
        Headers { inner }
    }
}

impl<'a> FromIterator<&'a (&'a str, &'a str)> for Headers {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a (&'a str, &'a str)>,
    {
        let mut inner = HashMap::default();
        for (k, v) in iter {
            let k = k.to_string();
            let v = v.to_string();
            let entry = inner.entry(k).or_insert_with(HashSet::default);
            entry.insert(v);
        }
        Headers { inner }
    }
}

fn parse_error<T, E: AsRef<str>>(e: E) -> std::io::Result<T> {
    trace!("header parse error: {}", e.as_ref());
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        e.as_ref(),
    ))
}

fn is_continuation(c: char) -> bool {
    return c == ' ' || c == '\t';
}

impl TryFrom<&[u8]> for Headers {
    type Error = std::io::Error;

    fn try_from(buf: &[u8]) -> std::io::Result<Self> {
        let mut inner = HashMap::default();
        let mut lines = if let Ok(line) = std::str::from_utf8(buf) {
            line.lines().peekable()
        } else {
            return parse_error("invalid utf8 received");
        };

        if let Some(line) = lines.next() {
            if !line.starts_with("NATS/") {
                return parse_error("version line does not begin with NATS/");
            }
        } else {
            return parse_error("expected header information not present");
        };

        while let Some(line) = lines.next() {
            let splits = line.splitn(2, ':').map(str::trim).collect::<Vec<_>>();
            match splits[..] {
                [k, v] => {
                    let entry = inner
                        .entry(k.to_string())
                        .or_insert_with(HashSet::default);

                    let mut s = String::new();
                    s.push_str(v);

                    while let Some(v) =
                        lines.next_if(|s| s.starts_with(&is_continuation))
                    {
                        s.push_str(&v[1..]);
                    }

                    for v in s.split(',') {
                        entry.insert(v.to_string());
                    }
                }
                [""] => continue,
                _ => {
                    return parse_error("malformed header input");
                }
            }
        }

        Ok(Headers { inner })
    }
}

impl Deref for Headers {
    type Target = HashMap<String, HashSet<String>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Headers {
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        // `<version line>\r\n[headers]\r\n\r\n[payload]\r\n`
        let mut buf = vec![];
        buf.extend_from_slice(b"NATS/1.0\r\n");
        for (k, vs) in &self.inner {
            for v in vs {
                buf.extend_from_slice(k.trim().as_bytes());
                buf.push(b':');
                buf.extend_from_slice(v.trim().as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
        }
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

#[cfg(test)]
mod try_from {
    use super::*;

    #[test]
    fn single_line_single_value() {
        let headers = Headers::try_from(
            "NATS/1.0 200\r\naccept-encoding: json\r\nauthorization: s3cr3t\r\n"
                .as_bytes(),
        )
        .unwrap();

        assert_eq!(
            headers.inner.get(&"accept-encoding".to_string()),
            Some(&HashSet::from_iter(vec!["json".to_string()]))
        );

        assert_eq!(
            headers.inner.get(&"authorization".to_string()),
            Some(&HashSet::from_iter(vec!["s3cr3t".to_string()]))
        );
    }

    #[test]
    fn single_line_multi_value() {
        let headers = Headers::try_from(
            "NATS/1.0 200\r\naccept-encoding: html,json,text\r\nauthorization: s3cr3t\r\n"
                .as_bytes(),
        )
        .unwrap();

        assert_eq!(
            headers.inner.get(&"accept-encoding".to_string()),
            Some(&HashSet::from_iter(vec![
                "html".to_string(),
                "json".to_string(),
                "text".to_string(),
            ]))
        );

        assert_eq!(
            headers.inner.get(&"authorization".to_string()),
            Some(&HashSet::from_iter(vec!["s3cr3t".to_string()]))
        );
    }

    #[test]
    fn multi_line_single_value_with_tab() {
        let headers = Headers::try_from(
            "NATS/1.0 200\r\nx-test: one\r\n\t two\r\n\t three\r\n".as_bytes(),
        )
        .unwrap();

        assert_eq!(
            headers.inner.get(&"x-test".to_string()),
            Some(&HashSet::from_iter(vec!["one two three".to_string(),]))
        );
    }

    #[test]
    fn multi_line_single_value_with_space() {
        let headers = Headers::try_from(
            "NATS/1.0 200\r\nx-test: one\r\n  two\r\n  three\r\n".as_bytes(),
        )
        .unwrap();

        assert_eq!(
            headers.inner.get(&"x-test".to_string()),
            Some(&HashSet::from_iter(vec!["one two three".to_string(),]))
        );
    }

    #[test]
    fn multi_line_multi_value_with_tab() {
        let headers = Headers::try_from(
            "NATS/1.0 200\r\nx-test: one, \r\n\ttwo,\r\n\tthree\r\n".as_bytes(),
        )
        .unwrap();

        assert_eq!(
            headers.inner.get(&"x-test".to_string()),
            Some(&HashSet::from_iter(vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
            ]))
        );
    }

    #[test]
    fn multi_line_multi_value_with_spaces() {
        let headers = Headers::try_from(
            "NATS/1.0 200\r\nx-test: one,\r\n two,\r\n three\r\n".as_bytes(),
        )
        .unwrap();

        assert_eq!(
            headers.inner.get(&"x-test".to_string()),
            Some(&HashSet::from_iter(vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
            ]))
        );
    }
}
