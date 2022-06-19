#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

impl Pos {
    #[allow(dead_code)]
    pub fn all(s: &str) -> Vec<Pos> {
        let chars = s.chars().collect::<Vec<char>>();
        let mut positions = vec![];

        let mut line = 1;
        let mut column = 1;
        for c in chars {
            let pos = Pos { line, column };
            positions.push(pos);
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        positions
    }
}
