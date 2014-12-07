pub fn test_rect_point(a: (f32, f32, f32, f32), b: (f32, f32)) -> bool {
    let (ax1, ay1, ax2, ay2) = a;
    let (bx, by) = b;

    (bx >= ax1 && bx < ax2) && (by >= ay1 && by < ay2)
}

pub fn test_rects(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    let (ax1, ay1, ax2, ay2) = a;
    let (bx1, by1, bx2, by2) = b;

    test_rect_point(a, (bx1, by1)) || test_rect_point(a, (bx2, by1)) ||
    test_rect_point(a, (bx1, by2)) || test_rect_point(a, (bx2, by2)) ||
    test_rect_point(b, (ax1, ay1)) || test_rect_point(b, (ax2, ay1)) ||
    test_rect_point(b, (ax1, ay2)) || test_rect_point(b, (ax2, ay2))
}
