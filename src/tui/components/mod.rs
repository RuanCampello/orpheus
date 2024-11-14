pub mod playlist;

pub(self) fn pad<'a>(content: &'a str, size: usize) -> String {
    let padding = " ".repeat(size);
    format!("{padding}{content}{padding}")
}
