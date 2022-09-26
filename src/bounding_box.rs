use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl BoundingBox {
    pub fn can_contain(&self, width: u32, height: u32) -> bool {
        width <= self.width && height <= self.height
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    pub fn perimeter(&self) -> u32 {
        self.width * 2 + self.height * 2
    }
}

impl std::ops::Add<&BoundingBox> for &BoundingBox {
    type Output = BoundingBox;
    fn add(self, v: &BoundingBox) -> BoundingBox {
        BoundingBox {
            x: self.x.min(v.x),
            y: self.y.min(v.y),
            width: (self.x + self.width).max(v.x + v.width) - self.x.min(v.x),
            height: (self.y + self.height).max(v.y + v.height) - self.y.min(v.y),
        }
    }
}

impl std::ops::Add<BoundingBox> for BoundingBox {
    type Output = BoundingBox;
    fn add(self, v: BoundingBox) -> BoundingBox {
        &self + &v
    }
}

impl Ord for BoundingBox {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.area().cmp(&other.area()) {
            Ordering::Equal => self.perimeter().cmp(&other.perimeter()),
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

impl PartialOrd for BoundingBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb_horizontal_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 1,
            y: 0,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_horizontal_overlap_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 1,
            y: 0,
            width: 2,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 3,
            height: 1,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_vertival_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 1,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 2,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_vertival_overlap_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 2,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 1,
            width: 1,
            height: 2,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 3,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 1,
            y: 1,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_overlap_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        let bb2 = BoundingBox {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 3,
            height: 3,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_disjoint_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 2,
            y: 2,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 3,
            height: 3,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_inside_add() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let expected_output = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_cmp_area() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        assert_eq!(true, bb1 < bb2);
        assert_eq!(false, bb2 < bb1);
    }

    #[test]
    fn bb_cmp_perimeter() {
        let bb1 = BoundingBox {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        let bb2 = BoundingBox {
            x: 0,
            y: 0,
            width: 4,
            height: 1,
        };
        assert_eq!(bb1.area(), bb2.area());
        assert_eq!(true, bb1 < bb2);
        assert_eq!(false, bb2 < bb1);
    }
}
