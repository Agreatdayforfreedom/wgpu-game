use cgmath::Point2;
use cgmath::Vector2;

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub origin: Point2<f32>,
    pub area: Vector2<f32>,
}

pub fn check_collision(one: Bounds, two: Bounds) -> bool {
    let x_overlap: bool =
        one.origin.x + one.area.x >= two.origin.x && two.origin.x + two.area.x >= one.origin.x;

    let y_overlap: bool =
        one.origin.y + one.area.y >= two.origin.y && two.origin.y + two.area.y >= one.origin.y;

    x_overlap && y_overlap
}
