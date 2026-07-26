use super::LayoutMetrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    pub page_start: usize,
    pub column: usize,
    pub row: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageDirection {
    Up,
    Down,
}

pub fn cursor_position(index: usize, metrics: &LayoutMetrics) -> Option<CursorPosition> {
    if metrics.page_capacity == 0 || metrics.rows_per_column == 0 {
        return None;
    }
    let page_start = (index / metrics.page_capacity) * metrics.page_capacity;
    let local = index - page_start;
    Some(CursorPosition {
        page_start,
        column: local / metrics.rows_per_column,
        row: local % metrics.rows_per_column,
    })
}

pub fn index_at_position(
    position: CursorPosition,
    entry_count: usize,
    metrics: &LayoutMetrics,
) -> Option<usize> {
    if metrics.page_capacity == 0
        || position.column >= metrics.columns.len()
        || position.row >= metrics.rows_per_column
    {
        return None;
    }
    let index = position.page_start + position.column * metrics.rows_per_column + position.row;
    (index < entry_count).then_some(index)
}

pub fn move_cursor(
    index: usize,
    entry_count: usize,
    direction: Direction,
    metrics: &LayoutMetrics,
) -> usize {
    if entry_count == 0 || metrics.page_capacity == 0 {
        return 0;
    }
    let index = index.min(entry_count - 1);
    let Some(position) = cursor_position(index, metrics) else {
        return index;
    };
    let page_len = (entry_count - position.page_start).min(metrics.page_capacity);
    let column_len = |column: usize| {
        page_len
            .saturating_sub(column * metrics.rows_per_column)
            .min(metrics.rows_per_column)
    };

    match direction {
        Direction::Up if position.row > 0 => index - 1,
        Direction::Up if position.column == 0 && position.page_start > 0 => index - 1,
        Direction::Down if position.row + 1 < column_len(position.column) => index + 1,
        Direction::Down
            if position.column + 1 == metrics.columns.len()
                && position.page_start + page_len < entry_count =>
        {
            position.page_start + page_len
        }
        Direction::Left if position.column > 0 => {
            let target_column = position.column - 1;
            position.page_start
                + target_column * metrics.rows_per_column
                + position
                    .row
                    .min(column_len(target_column).saturating_sub(1))
        }
        Direction::Left if position.page_start > 0 => {
            let previous_page_start = position.page_start - metrics.page_capacity;
            let previous_page_len = metrics.page_capacity.min(entry_count - previous_page_start);
            let target_column = metrics.columns.len().saturating_sub(1);
            let target_len = previous_page_len
                .saturating_sub(target_column * metrics.rows_per_column)
                .min(metrics.rows_per_column);
            previous_page_start
                + target_column * metrics.rows_per_column
                + position.row.min(target_len.saturating_sub(1))
        }
        Direction::Right if position.column + 1 < metrics.columns.len() => {
            let target_column = position.column + 1;
            let target_len = column_len(target_column);
            if target_len == 0 {
                index
            } else {
                position.page_start
                    + target_column * metrics.rows_per_column
                    + position.row.min(target_len - 1)
            }
        }
        Direction::Right if position.page_start + page_len < entry_count => {
            let next_page_start = position.page_start + metrics.page_capacity;
            let first_column_len = (entry_count - next_page_start).min(metrics.rows_per_column);
            next_page_start + position.row.min(first_column_len.saturating_sub(1))
        }
        _ => index,
    }
}

pub fn move_page(
    index: usize,
    entry_count: usize,
    direction: PageDirection,
    metrics: &LayoutMetrics,
) -> usize {
    if entry_count == 0 || metrics.page_capacity == 0 {
        return 0;
    }
    let index = index.min(entry_count - 1);
    match direction {
        PageDirection::Up => index.saturating_sub(metrics.page_capacity),
        PageDirection::Down => index
            .saturating_add(metrics.page_capacity)
            .min(entry_count - 1),
    }
}
