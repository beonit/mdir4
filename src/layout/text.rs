use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn cell_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

pub fn truncate_end(text: &str, max_cells: usize, ellipsis: &str) -> String {
    if cell_width(text) <= max_cells {
        return text.to_string();
    }
    if max_cells == 0 {
        return String::new();
    }
    let ellipsis_width = cell_width(ellipsis).min(max_cells);
    if ellipsis_width == max_cells {
        return take_cells(ellipsis, max_cells);
    }
    let content_limit = max_cells - ellipsis_width;
    let mut output = take_cells(text, content_limit);
    output.push_str(&take_cells(ellipsis, max_cells - cell_width(&output)));
    output
}

pub fn pad_or_truncate(text: &str, cells: usize) -> String {
    let mut output = truncate_end(text, cells, "…");
    output.extend(std::iter::repeat_n(
        ' ',
        cells.saturating_sub(cell_width(&output)),
    ));
    output
}

fn take_cells(text: &str, max_cells: usize) -> String {
    let mut output = String::new();
    let mut used = 0;
    for grapheme in UnicodeSegmentation::graphemes(text, true) {
        let width = cell_width(grapheme);
        if used + width > max_cells {
            break;
        }
        output.push_str(grapheme);
        used += width;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_results_never_exceed_requested_cells() {
        for text in [
            "ascii.txt",
            "한글파일.txt",
            "日本語.txt",
            "e\u{301}.txt",
            "😀x",
        ] {
            for width in 0..12 {
                assert!(cell_width(&truncate_end(text, width, "…")) <= width);
                assert_eq!(cell_width(&pad_or_truncate(text, width)), width);
            }
        }
    }
}
