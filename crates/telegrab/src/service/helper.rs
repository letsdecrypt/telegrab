use crate::model::dto::pagination::CursorBasedPaginationResponse;
use crate::model::Direction;

pub fn build_cursor_pagination<T>(
    data: Vec<T>,
    total: u64,
    limit: usize,
    direction: Direction,
    has_cursor: bool,
) -> CursorBasedPaginationResponse<T> {
    let has_additional = data.len() > limit;
    let mut actual_pics = if has_additional {
        data.into_iter().take(limit).collect()
    } else {
        data
    };

    let (has_next, has_prev) = match direction {
        Direction::Forward => {
            let has_next = has_additional;
            let has_prev = has_cursor;
            (has_next, has_prev)
        }
        Direction::Backward => {
            let has_prev = has_additional;
            let has_next = has_cursor;
            (has_prev, has_next)
        }
    };

    // 如果是向后翻页，需要反转结果以恢复正确的顺序
    if direction == Direction::Backward {
        actual_pics.reverse();
    }
    CursorBasedPaginationResponse {
        data: actual_pics,
        total,
        has_next,
        has_prev,
    }
}
