use super::rect::Rect;

pub trait Collidable<R, P> {
    fn test_point(point: &P) -> bool;
    fn test_rect(rect: &R) -> bool;
}

pub fn test_rect_point(a: &Rect<f32>, b: (f32, f32)) -> bool {
    let (ax1, ay1, ax2, ay2) = a.ltrb();
    let (bx, by) = b;

    assert!(ax1 >= 0.0 && ay1 >= 0.0 && ax2 >= ax1 && ay2 >= ay1);
    assert!(bx >= 0.0 && by >= 0.0);

    (bx >= ax1 && bx < ax2) && (by >= ay1 && by < ay2)
}

pub fn test_rects(a: &Rect<f32>, b: &Rect<f32>) -> bool {
    let (ax1, ay1, ax2, ay2) = a.ltrb();
    let (bx1, by1, bx2, by2) = b.ltrb();

    test_rect_point(a, (bx1, by1)) || test_rect_point(a, (bx2, by1)) ||
    test_rect_point(a, (bx1, by2)) || test_rect_point(a, (bx2, by2)) ||
    test_rect_point(b, (ax1, ay1)) || test_rect_point(b, (ax2, ay1)) ||
    test_rect_point(b, (ax1, ay2)) || test_rect_point(b, (ax2, ay2))
}

pub fn test_rect_vert_line(a: &Rect<f32>, x: f32, width: f32) -> bool {
    let (ax1, _, ax2, _) = a.ltrb();

    (x >= ax1 && x < ax2) || (x+width >= ax1 && x+width < ax2)
}
