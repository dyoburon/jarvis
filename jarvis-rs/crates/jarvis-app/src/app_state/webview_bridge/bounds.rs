//! Coordinate conversion between tiling rects and wry rects.

use jarvis_common::types::Rect;

// =============================================================================
// COORDINATE CONVERSION
// =============================================================================

/// Convert a tiling `Rect` (f64 logical coords) to a wry `Rect`.
pub fn tiling_rect_to_wry(rect: &Rect) -> wry::Rect {
    wry::Rect {
        position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(rect.x, rect.y)),
        size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(rect.width, rect.height)),
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiling_rect_converts_to_wry_rect() {
        let tiling = Rect {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 600.0,
        };
        let wry_rect = tiling_rect_to_wry(&tiling);

        match wry_rect.position {
            wry::dpi::Position::Logical(pos) => {
                assert!((pos.x - 100.0).abs() < f64::EPSILON);
                assert!((pos.y - 50.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected logical position"),
        }

        match wry_rect.size {
            wry::dpi::Size::Logical(size) => {
                assert!((size.width - 800.0).abs() < f64::EPSILON);
                assert!((size.height - 600.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected logical size"),
        }
    }

    #[test]
    fn tiling_rect_zero_converts_correctly() {
        let tiling = Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
        let wry_rect = tiling_rect_to_wry(&tiling);

        match wry_rect.position {
            wry::dpi::Position::Logical(pos) => {
                assert!((pos.x).abs() < f64::EPSILON);
                assert!((pos.y).abs() < f64::EPSILON);
            }
            _ => panic!("Expected logical position"),
        }
        match wry_rect.size {
            wry::dpi::Size::Logical(size) => {
                assert!((size.width).abs() < f64::EPSILON);
                assert!((size.height).abs() < f64::EPSILON);
            }
            _ => panic!("Expected logical size"),
        }
    }

    #[test]
    fn tiling_rect_large_values() {
        let tiling = Rect {
            x: 0.0,
            y: 0.0,
            width: 3840.0,
            height: 2160.0,
        };
        let wry_rect = tiling_rect_to_wry(&tiling);

        match wry_rect.size {
            wry::dpi::Size::Logical(size) => {
                assert!((size.width - 3840.0).abs() < f64::EPSILON);
                assert!((size.height - 2160.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected logical size"),
        }
    }
}
