use crate::enemie;
use crate::projectile;

pub fn check_collision(one: &projectile::Projectile, two: &enemie::Enemy) -> bool {
    let x_overlap: bool =
        one.position.x + one.size >= two.position.x && two.position.x + two.size >= one.position.x;

    let y_overlap: bool =
        one.position.y + one.size >= two.position.y && two.position.y + two.size >= one.position.y;

    x_overlap && y_overlap
}
