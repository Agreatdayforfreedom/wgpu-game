use std::slice::SliceIndex;

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
            direction: Vector2::new(0.0, 0.0),
            active: false,
        }
    }

    pub fn get_direction(&mut self) -> Vector2<f32> {
        self.direction
    }

    pub fn decouple(&mut self) {
        self.active = false;
    }

    pub fn couple(&mut self, position: Vector2<f32>) {
        if !self.active {
            self.active = true;
            self.direction = self.get_direction_closer_point(position);
        }
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

    pub fn next(&mut self, position: Vector2<f32>) {
        if self.current_point == self.points.len() - 1 {
            self.current_point = 0;
        } else {
            self.current_point += 1;
        }

        let point = self.points.get(self.current_point as usize).unwrap();
        self.direction = (position - point).normalize();
    }

    pub fn is_over(&mut self, position: Vector2<f32>) -> bool {
        let point = self.points.get(self.current_point).unwrap();
        let padding = 5.0;
        let x_overlap: bool = point.x + padding >= position.x && position.x + padding >= point.x;

        let y_overlap: bool = point.y + padding >= position.y && position.y + padding >= point.y;
        let overlap: bool = y_overlap && x_overlap;

        overlap
    }
}
