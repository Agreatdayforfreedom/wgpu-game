use cgmath::{InnerSpace, Vector2};

use crate::util::distance;

pub struct PatrolArea {
    points: Vec<Vector2<f32>>,
    current_point: usize,
    direction: Vector2<f32>,
    active: bool,
}

impl PatrolArea {
    pub fn new(points: Vec<Vector2<f32>>) -> Self {
        Self {
            points,
            current_point: 0,
            direction: (0.0, 0.0).into(),
            active: false,
        }
    }

    // pub fn active(&mut self, position: Vector2<f32>) {
    //     // position starts in any position
    //     if !self.active {
    //         self.active = true;

    //         self.direction = self.get_direction_closer_point(position);
    //     }
    //     // get the closer point between all the point and position

    //     // create a direction vector to the point

    //     // check if position reach the point

    //     // if reach the point update the direction vector to the next point

    //     // if reach the last point, point to the first one
    // }

    // pub fn deactive(&mut self, position: Vector2<f32>) {
    //     if self.active {
    //         if self.is_over(position) {
    //             self.direction = self.get_direction_closer_point(position);
    //         }
    //     }
    // }

    pub fn get_direction(&mut self, position: Vector2<f32>) -> Vector2<f32> {
        let dir: Vector2<f32>;
        if !self.active {
            dir = self.get_direction_closer_point(position);
            self.active = true;
        } else {
            let point = self.points.get(self.current_point as usize).unwrap();
            dir = (position - point).normalize();
        }

        dir
    }

    fn get_direction_closer_point(&mut self, position: Vector2<f32>) -> Vector2<f32> {
        let mut min_dist = f32::MAX;

        let mut i = 0;
        let mut closer_point_index = 0;
        while i < self.points.len() {
            let point = self.points.get(i).unwrap();
            let dist = distance(position, *point);

            if dist < min_dist && self.current_point != i {
                min_dist = dist;
                closer_point_index = i;
            }
            i += 1;
        }

        let to = self.points.get(closer_point_index).unwrap();
        self.current_point = closer_point_index;

        (position - to).normalize()
    }

    pub fn next(&mut self) {
        if self.current_point == self.points.len() - 1 {
            self.current_point = 0;
        } else {
            self.current_point += 1;
        }
    }

    pub fn is_over(&mut self, position: Vector2<f32>) -> bool {
        let point = self.points.get(self.current_point).unwrap();
        let padding = 5.0;
        let x_overlap: bool = point.x + padding >= position.x && position.x + padding >= point.x;

        let y_overlap: bool = point.y + padding >= position.y && position.y + padding >= point.y;

        y_overlap && x_overlap
    }
}
