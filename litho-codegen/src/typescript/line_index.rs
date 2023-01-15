use std::collections::BTreeMap;

pub struct LineIndex {
    /// Maps the start index of a line to a line number.
    map: BTreeMap<usize, usize>,
    len: usize,
}

impl LineIndex {
    pub fn new(source: &str) -> LineIndex {
        let mut len = 0;

        LineIndex {
            map: source
                .split_inclusive('\n')
                .enumerate()
                .map(|(i, line)| {
                    let result = (len, i);
                    len += line.len();
                    result
                })
                .collect(),
            len,
        }
    }

    pub fn lookup(&self, index: usize) -> (usize, usize) {
        match self.map.range(..=index).rev().next() {
            Some((&offset, &line)) => (line, index - offset),
            None => (self.map.len(), index - self.len),
        }
    }
}
