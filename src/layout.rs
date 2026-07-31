use ratatui::layout::Rect;

mod navigation;
pub mod text;

pub use navigation::{
    CursorPosition, Direction, PageDirection, cursor_position, index_at_position, move_cursor,
    move_page,
};

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 15;
pub const SINGLE_ROW_FUNCTION_WIDTH: u16 = 96;
pub const MIN_PREVIEW_WIDTH: u16 = 120;
const MIN_COLUMN_WIDTH: u16 = 12;
const MAX_COLUMNS: u16 = 6;
/// Two Long-view columns need room for a name, size, and date in each column.
const MIN_TWO_COLUMN_WIDTH: u16 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnCountMode {
    #[default]
    Auto,
    Fixed(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnWidthMode {
    Compact,
    #[default]
    Normal,
    Wide,
    Custom(u16),
}

impl ColumnWidthMode {
    fn target_width(self) -> u16 {
        match self {
            Self::Compact => 24,
            Self::Normal => 32,
            Self::Wide => 40,
            Self::Custom(width) => width.clamp(MIN_COLUMN_WIDTH, 80),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutSettings {
    pub column_count: ColumnCountMode,
    pub column_width: ColumnWidthMode,
    pub preview: PreviewLayoutSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewLayoutSettings {
    pub enabled: bool,
    pub width_percent: u8,
}

impl Default for PreviewLayoutSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            width_percent: 50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Viewport {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutMetrics {
    pub viewport: Rect,
    pub preview: Option<Rect>,
    pub path_bar: Rect,
    pub list: Rect,
    pub item_detail: Rect,
    pub directory_summary: Rect,
    pub message_bar: Rect,
    pub function_bar: Rect,
    pub columns: Vec<Rect>,
    pub rows_per_column: usize,
    pub page_capacity: usize,
    pub too_small: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellLayout {
    pub viewport: Rect,
    pub path_bar: Rect,
    pub body: Rect,
    pub status_bar: Rect,
    pub function_bar: Rect,
    pub too_small: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentLayout {
    pub shell: ShellLayout,
    pub document: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitStatusLayout {
    pub shell: ShellLayout,
    pub changes: Rect,
    pub diff_preview: Option<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitCommitLayout {
    pub shell: ShellLayout,
    pub commit_and_files: Rect,
    pub separator: Rect,
    pub diff: Rect,
}

pub fn calculate_shell(viewport: Viewport) -> ShellLayout {
    let full = Rect::new(0, 0, viewport.width, viewport.height);
    if viewport.width < MIN_WIDTH || viewport.height < MIN_HEIGHT {
        return ShellLayout {
            viewport: full,
            path_bar: Rect::default(),
            body: Rect::default(),
            status_bar: Rect::default(),
            function_bar: Rect::default(),
            too_small: true,
        };
    }
    let function_rows = if viewport.width >= SINGLE_ROW_FUNCTION_WIDTH {
        1
    } else {
        2
    };
    ShellLayout {
        viewport: full,
        path_bar: Rect::new(0, 0, viewport.width, 1),
        body: Rect::new(0, 1, viewport.width, viewport.height - 2 - function_rows),
        status_bar: Rect::new(0, viewport.height - function_rows - 1, viewport.width, 1),
        function_bar: Rect::new(
            0,
            viewport.height - function_rows,
            viewport.width,
            function_rows,
        ),
        too_small: false,
    }
}

pub fn calculate_document(viewport: Viewport) -> DocumentLayout {
    let shell = calculate_shell(viewport);
    DocumentLayout {
        document: shell.body,
        shell,
    }
}

pub fn calculate_git_status(viewport: Viewport, preview: PreviewLayoutSettings) -> GitStatusLayout {
    let shell = calculate_shell(viewport);
    if shell.too_small {
        return GitStatusLayout {
            shell,
            changes: Rect::default(),
            diff_preview: None,
        };
    }
    let preview_width = (preview.enabled && shell.body.width >= MIN_PREVIEW_WIDTH).then(|| {
        let requested = shell.body.width * u16::from(preview.width_percent.clamp(35, 65)) / 100;
        requested.clamp(60, shell.body.width.saturating_sub(60))
    });
    let changes_width = preview_width.map_or(shell.body.width, |width| shell.body.width - width);
    GitStatusLayout {
        shell,
        changes: Rect::new(shell.body.x, shell.body.y, changes_width, shell.body.height),
        diff_preview: preview_width.map(|width| {
            Rect::new(
                shell.body.x + changes_width,
                shell.body.y,
                width,
                shell.body.height,
            )
        }),
    }
}

pub fn calculate_git_commit(viewport: Viewport) -> GitCommitLayout {
    let shell = calculate_shell(viewport);
    if shell.too_small {
        return GitCommitLayout {
            shell,
            commit_and_files: Rect::default(),
            separator: Rect::default(),
            diff: Rect::default(),
        };
    }
    let available = shell.body.width.saturating_sub(1);
    let left_width = (available.saturating_mul(42) / 100).clamp(24, available.saturating_sub(32));
    let right_width = available.saturating_sub(left_width);
    GitCommitLayout {
        shell,
        commit_and_files: Rect::new(shell.body.x, shell.body.y, left_width, shell.body.height),
        separator: Rect::new(
            shell.body.x + left_width,
            shell.body.y,
            1,
            shell.body.height,
        ),
        diff: Rect::new(
            shell.body.x + left_width + 1,
            shell.body.y,
            right_width,
            shell.body.height,
        ),
    }
}

pub fn calculate(viewport: Viewport, settings: LayoutSettings) -> LayoutMetrics {
    let shell = calculate_shell(viewport);
    if shell.too_small {
        return LayoutMetrics {
            viewport: shell.viewport,
            preview: None,
            path_bar: Rect::default(),
            list: Rect::default(),
            item_detail: Rect::default(),
            directory_summary: Rect::default(),
            message_bar: Rect::default(),
            function_bar: Rect::default(),
            columns: Vec::new(),
            rows_per_column: 0,
            page_capacity: 0,
            too_small: true,
        };
    }

    let preview_width =
        (settings.preview.enabled && shell.body.width >= MIN_PREVIEW_WIDTH).then(|| {
            let requested =
                shell.body.width * u16::from(settings.preview.width_percent.clamp(35, 65)) / 100;
            requested.clamp(60, shell.body.width.saturating_sub(60))
        });
    let browser_width = preview_width.map_or(shell.body.width, |width| shell.body.width - width);
    let list = Rect::new(shell.body.x, shell.body.y, browser_width, shell.body.height);
    let preview = preview_width
        .map(|width| Rect::new(shell.body.x + browser_width, list.y, width, list.height));
    let mut column_count = match settings.column_count {
        ColumnCountMode::Auto => {
            let target_width = settings.column_width.target_width();
            ((browser_width + target_width / 2) / target_width).clamp(1, MAX_COLUMNS)
        }
        ColumnCountMode::Fixed(count) => u16::from(count).clamp(1, MAX_COLUMNS),
    };
    while column_count > 1 && browser_width / column_count < MIN_COLUMN_WIDTH {
        column_count -= 1;
    }

    let base_width = browser_width / column_count;
    let remainder = browser_width % column_count;
    let mut x = 0;
    let mut columns = Vec::with_capacity(column_count as usize);
    for index in 0..column_count {
        let width = base_width + u16::from(index < remainder);
        columns.push(Rect::new(x, list.y, width, list.height));
        x += width;
    }

    let rows_per_column = list.height as usize;
    let page_capacity = rows_per_column * columns.len();

    LayoutMetrics {
        viewport: shell.viewport,
        preview,
        path_bar: shell.path_bar,
        list,
        item_detail: shell.status_bar,
        directory_summary: Rect::default(),
        message_bar: Rect::default(),
        function_bar: shell.function_bar,
        columns,
        rows_per_column,
        page_capacity,
        too_small: false,
    }
}

pub fn calculate_for_entries(
    viewport: Viewport,
    settings: LayoutSettings,
    entry_count: usize,
) -> LayoutMetrics {
    calculate_for_view(viewport, settings, entry_count, false)
}

pub fn calculate_for_view(
    viewport: Viewport,
    settings: LayoutSettings,
    entry_count: usize,
    long_view: bool,
) -> LayoutMetrics {
    let mut metrics = calculate(viewport, settings);
    if metrics.too_small {
        return metrics;
    }
    if long_view {
        metrics.rows_per_column = metrics.rows_per_column.saturating_sub(1);
        metrics.page_capacity = metrics.rows_per_column * metrics.columns.len();
    }
    if !matches!(settings.column_count, ColumnCountMode::Auto) {
        balance_visible_rows(&mut metrics, entry_count);
        return metrics;
    }
    let minimum_columns =
        if metrics.list.width >= MIN_TWO_COLUMN_WIDTH && metrics.columns.len() >= 2 {
            2
        } else {
            1
        };
    let required = entry_count
        .max(1)
        .div_ceil(metrics.rows_per_column)
        .clamp(minimum_columns.max(1), metrics.columns.len());
    if required == metrics.columns.len() {
        balance_visible_rows(&mut metrics, entry_count);
        return metrics;
    }
    let count = required as u16;
    let base_width = metrics.list.width / count;
    let remainder = metrics.list.width % count;
    let mut x = metrics.list.x;
    metrics.columns = (0..count)
        .map(|index| {
            let width = base_width + u16::from(index < remainder);
            let column = Rect::new(x, metrics.list.y, width, metrics.list.height);
            x += width;
            column
        })
        .collect();
    metrics.page_capacity = metrics.rows_per_column * metrics.columns.len();
    balance_visible_rows(&mut metrics, entry_count);
    metrics
}

fn balance_visible_rows(metrics: &mut LayoutMetrics, entry_count: usize) {
    let columns = metrics.columns.len();
    if columns == 0 || metrics.rows_per_column == 0 {
        return;
    }
    let physical_capacity = metrics.rows_per_column * columns;
    if entry_count <= physical_capacity {
        metrics.rows_per_column = entry_count.max(1).div_ceil(columns);
        metrics.page_capacity = metrics.rows_per_column * columns;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(width: u16, height: u16) -> LayoutMetrics {
        calculate(
            Viewport { width, height },
            settings(ColumnCountMode::Auto, ColumnWidthMode::Normal),
        )
    }

    fn settings(column_count: ColumnCountMode, column_width: ColumnWidthMode) -> LayoutSettings {
        LayoutSettings {
            column_count,
            column_width,
            preview: PreviewLayoutSettings {
                enabled: false,
                ..PreviewLayoutSettings::default()
            },
            ..LayoutSettings::default()
        }
    }

    #[test]
    fn normal_auto_columns_match_reference_sizes() {
        assert_eq!(metrics(80, 25).columns.len(), 3);
        assert_eq!(metrics(120, 30).columns.len(), 4);
        assert_eq!(metrics(160, 40).columns.len(), 5);
    }

    #[test]
    fn auto_columns_expand_and_grow_with_the_entry_count() {
        let viewport = Viewport {
            width: 120,
            height: 40,
        };
        let settings = settings(ColumnCountMode::Auto, ColumnWidthMode::Normal);
        let rows = calculate(viewport, settings).rows_per_column;
        let one = calculate_for_entries(viewport, settings, rows);
        assert_eq!(one.columns.len(), 1);
        assert_eq!(one.columns[0].width, 120);
        assert_eq!(
            calculate_for_entries(viewport, settings, rows + 1)
                .columns
                .len(),
            2
        );
        assert_eq!(
            calculate_for_entries(viewport, settings, rows * 20)
                .columns
                .len(),
            4
        );
    }

    #[test]
    fn wide_auto_layout_keeps_two_columns_even_for_a_short_listing() {
        let metrics = calculate_for_entries(
            Viewport {
                width: MIN_TWO_COLUMN_WIDTH,
                height: 25,
            },
            settings(ColumnCountMode::Auto, ColumnWidthMode::Normal),
            1,
        );
        assert_eq!(metrics.columns.len(), 2);
        assert_eq!(metrics.columns[0].width, MIN_TWO_COLUMN_WIDTH / 2);

        let long = calculate_for_view(
            Viewport {
                width: MIN_TWO_COLUMN_WIDTH,
                height: 25,
            },
            settings(ColumnCountMode::Auto, ColumnWidthMode::Normal),
            1,
            true,
        );
        assert_eq!(long.columns.len(), 2);
        assert_eq!(long.rows_per_column, 1);
    }

    #[test]
    fn short_listing_is_balanced_across_wide_columns() {
        let metrics = calculate_for_entries(
            Viewport {
                width: MIN_TWO_COLUMN_WIDTH,
                height: 25,
            },
            settings(ColumnCountMode::Auto, ColumnWidthMode::Normal),
            15,
        );
        assert_eq!(metrics.columns.len(), 2);
        assert_eq!(metrics.rows_per_column, 8);
        assert_eq!(metrics.page_capacity, 16);
    }

    #[test]
    fn auto_uses_the_selected_width_profile() {
        let viewport = Viewport {
            width: 120,
            height: 25,
        };
        assert_eq!(
            calculate(
                viewport,
                settings(ColumnCountMode::Auto, ColumnWidthMode::Compact)
            )
            .columns
            .len(),
            5
        );
        assert_eq!(
            calculate(
                viewport,
                settings(ColumnCountMode::Auto, ColumnWidthMode::Normal)
            )
            .columns
            .len(),
            4
        );
        assert_eq!(
            calculate(
                viewport,
                settings(ColumnCountMode::Auto, ColumnWidthMode::Wide)
            )
            .columns
            .len(),
            3
        );
        assert_eq!(
            calculate(
                viewport,
                settings(ColumnCountMode::Auto, ColumnWidthMode::Custom(60))
            )
            .columns
            .len(),
            2
        );
    }

    #[test]
    fn custom_width_is_clamped_to_supported_range() {
        assert_eq!(ColumnWidthMode::Custom(11).target_width(), 12);
        assert_eq!(ColumnWidthMode::Custom(12).target_width(), 12);
        assert_eq!(ColumnWidthMode::Custom(80).target_width(), 80);
        assert_eq!(ColumnWidthMode::Custom(81).target_width(), 80);

        let viewport = Viewport {
            width: 80,
            height: 25,
        };
        assert_eq!(
            calculate(
                viewport,
                settings(ColumnCountMode::Auto, ColumnWidthMode::Custom(1))
            )
            .columns
            .len(),
            6
        );
        assert_eq!(
            calculate(
                viewport,
                settings(ColumnCountMode::Auto, ColumnWidthMode::Custom(100))
            )
            .columns
            .len(),
            1
        );
    }

    #[test]
    fn fixed_mode_keeps_the_requested_count_when_it_fits() {
        let viewport = Viewport {
            width: 120,
            height: 25,
        };
        for requested in 1..=6 {
            let result = calculate(
                viewport,
                settings(ColumnCountMode::Fixed(requested), ColumnWidthMode::Normal),
            );
            assert_eq!(result.columns.len(), requested as usize);
        }
    }

    #[test]
    fn fixed_mode_reduces_count_below_minimum_column_width() {
        let viewport = Viewport {
            width: 60,
            height: 25,
        };
        let result = calculate(
            viewport,
            settings(ColumnCountMode::Fixed(6), ColumnWidthMode::Normal),
        );
        assert_eq!(result.columns.len(), 5);
        assert!(
            result
                .columns
                .iter()
                .all(|column| column.width >= MIN_COLUMN_WIDTH)
        );
    }

    #[test]
    fn fixed_count_is_clamped_to_one_through_six() {
        let viewport = Viewport {
            width: 120,
            height: 25,
        };
        assert_eq!(
            calculate(
                viewport,
                settings(ColumnCountMode::Fixed(0), ColumnWidthMode::Normal)
            )
            .columns
            .len(),
            1
        );
        assert_eq!(
            calculate(
                viewport,
                settings(ColumnCountMode::Fixed(9), ColumnWidthMode::Normal)
            )
            .columns
            .len(),
            6
        );
    }

    #[test]
    fn document_layout_uses_full_body_regardless_of_browser_preview_setting() {
        let viewport = Viewport {
            width: 160,
            height: 40,
        };
        let browser = calculate(viewport, LayoutSettings::default());
        assert_eq!(browser.list.width, 80);
        let document = calculate_document(viewport);
        assert_eq!(document.document, document.shell.body);
        assert_eq!(document.document.width, 160);
        assert_eq!(document.document.right(), document.shell.viewport.right());
    }

    #[test]
    fn git_commit_split_covers_the_full_body_without_overlap() {
        for width in [60, 80, 120, 160, 240] {
            let layout = calculate_git_commit(Viewport { width, height: 25 });
            assert!(!layout.shell.too_small);
            assert_eq!(layout.commit_and_files.right(), layout.separator.x);
            assert_eq!(layout.separator.right(), layout.diff.x);
            assert_eq!(layout.diff.right(), layout.shell.body.right());
            assert_eq!(
                layout.commit_and_files.width + layout.separator.width + layout.diff.width,
                layout.shell.body.width
            );
            assert!(layout.commit_and_files.width >= 24);
            assert!(layout.diff.width >= 32);
        }
    }

    #[test]
    fn git_status_preview_is_mode_specific_and_covers_the_body() {
        let viewport = Viewport {
            width: 160,
            height: 25,
        };
        let preview = calculate_git_status(viewport, PreviewLayoutSettings::default());
        assert_eq!(preview.changes.width, 80);
        assert_eq!(
            preview.diff_preview.unwrap().right(),
            preview.shell.body.right()
        );
        let without_preview = calculate_git_status(
            viewport,
            PreviewLayoutSettings {
                enabled: false,
                ..PreviewLayoutSettings::default()
            },
        );
        assert_eq!(without_preview.changes, without_preview.shell.body);
        assert!(without_preview.diff_preview.is_none());
    }

    #[test]
    fn columns_cover_the_list_without_gaps_or_overlap() {
        let modes = [
            settings(ColumnCountMode::Auto, ColumnWidthMode::Compact),
            settings(ColumnCountMode::Auto, ColumnWidthMode::Normal),
            settings(ColumnCountMode::Auto, ColumnWidthMode::Wide),
            settings(ColumnCountMode::Auto, ColumnWidthMode::Custom(17)),
            settings(ColumnCountMode::Fixed(6), ColumnWidthMode::Normal),
        ];
        for width in 60..=220 {
            for mode in modes {
                let result = calculate(Viewport { width, height: 25 }, mode);
                assert!(!result.too_small);
                assert_eq!(
                    result
                        .columns
                        .iter()
                        .map(|column| column.width)
                        .sum::<u16>(),
                    width
                );
                assert_eq!(result.columns.first().unwrap().x, 0);
                assert_eq!(result.columns.last().unwrap().right(), result.list.right());
                for pair in result.columns.windows(2) {
                    assert_eq!(pair[0].right(), pair[1].x);
                    assert!(pair[0].width.abs_diff(pair[1].width) <= 1);
                }
            }
        }
    }

    #[test]
    fn normal_boundary_widths_are_deterministic() {
        assert_eq!(metrics(79, 25).columns.len(), 2);
        assert_eq!(metrics(80, 25).columns.len(), 3);
        assert_eq!(metrics(81, 25).columns.len(), 3);
        assert_eq!(metrics(119, 25).columns.len(), 4);
        assert_eq!(metrics(120, 25).columns.len(), 4);
        assert_eq!(metrics(121, 25).columns.len(), 4);
        assert_eq!(metrics(159, 25).columns.len(), 5);
        assert_eq!(metrics(160, 25).columns.len(), 5);
        assert_eq!(metrics(161, 25).columns.len(), 5);
    }

    #[test]
    fn required_terminal_sizes_have_valid_regions() {
        let sizes = [
            (60, 15),
            (79, 24),
            (80, 25),
            (100, 30),
            (120, 40),
            (160, 50),
        ];
        for (width, height) in sizes {
            let result = metrics(width, height);
            assert!(!result.too_small);
            assert_eq!(result.viewport, Rect::new(0, 0, width, height));
            let function_rows = if width >= SINGLE_ROW_FUNCTION_WIDTH {
                1
            } else {
                2
            };
            assert_eq!(result.list.height, height - 2 - function_rows);
            assert_eq!(
                result.rows_per_column,
                usize::from(height - 2 - function_rows)
            );
            assert_eq!(
                result.page_capacity,
                result.rows_per_column * result.columns.len()
            );
            assert_eq!(result.path_bar.y, 0);
            assert_eq!(result.function_bar.y, height - function_rows);
            assert_eq!(result.function_bar.height, function_rows);
        }
    }

    #[test]
    fn too_small_terminal_is_safe() {
        let metrics = metrics(59, 14);
        assert!(metrics.too_small);
        assert_eq!(metrics.page_capacity, 0);
        assert_eq!(move_cursor(0, 10, Direction::Right, &metrics), 0);
    }

    #[test]
    fn right_uses_nearest_row_in_short_last_column() {
        let metrics = metrics(80, 9 + MIN_HEIGHT);
        let rows = metrics.rows_per_column;
        let count = rows * 2 + 2;
        let b3 = rows + 2;

        assert_eq!(
            move_cursor(b3, count, Direction::Right, &metrics),
            rows * 2 + 1
        );
    }

    #[test]
    fn spatial_navigation_stays_in_column_at_edges() {
        let metrics = metrics(80, 25);
        let rows = metrics.rows_per_column;

        assert_eq!(move_cursor(0, 100, Direction::Up, &metrics), 0);
        assert_eq!(
            move_cursor(rows - 1, 100, Direction::Down, &metrics),
            rows - 1
        );
        assert_eq!(move_cursor(1, 100, Direction::Right, &metrics), rows + 1);
        assert_eq!(move_cursor(rows + 1, 100, Direction::Left, &metrics), 1);
    }
}
