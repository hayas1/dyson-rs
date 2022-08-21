pub(crate) fn postr((row, col): (usize, usize)) -> String {
    format!("line {} (col {})", row + 1, col + 1)
}
