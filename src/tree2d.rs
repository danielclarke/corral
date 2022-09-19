#[derive(Debug, PartialEq)]
struct BoundingBox {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl BoundingBox {
    fn area(&self) -> u32 {
        self.width * self.height
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

pub enum Tree2d<T> {
    Leaf {
        bb: BoundingBox,
    },
    Node {
        bb: BoundingBox,
        right: Box<Self>,
        down: Box<Self>,
        data: T,
    },
}

impl<T> Tree2d<T> {
    fn new(data: T, width: u32, height: u32) -> Self {
        Tree2d::Node {
            bb: BoundingBox {
                x: 0,
                y: 0,
                width,
                height,
            },
            right: Box::new(Self::Leaf {
                bb: BoundingBox {
                    x: width,
                    y: 0,
                    width: 0,
                    height: 0,
                },
            }),
            down: Box::new(Self::Leaf {
                bb: BoundingBox {
                    x: 0,
                    y: height,
                    width: 0,
                    height: 0,
                },
            }),
            data,
        }
    }

    fn min_bounding_box(&self, width: u32, height: u32) -> (u32, u32) {
        (0, 0)
    }

    // fn insert(&mut self, data: T, width: u32, height: u32) -> Box<Self> {
    //     match self {
    //         Self::Leaf { x, y } => Box::new(Self::Node {
    //             x: *x,
    //             y: *y,
    //             width,
    //             height,
    //             right: Box::new(Self::Leaf {
    //                 x: *x + width,
    //                 y: *y,
    //             }),
    //             down: Box::new(Self::Leaf {
    //                 x: *x,
    //                 y: *y + height,
    //             }),
    //             data,
    //         }),
    //         Self::Node {
    //             x: _x,
    //             y: _y,
    //             width: _width,
    //             height: _height,
    //             right,
    //             down: _down,
    //             data: _data,
    //         } => right.insert(data, width, height),
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb_horizontal_add() {
        let bb1 = BoundingBox{x: 0, y: 0, width: 1, height: 1};
        let bb2 = BoundingBox{x: 1, y: 0, width: 1, height: 1};
        let expected_output = BoundingBox{x: 0, y: 0, width: 2, height: 1};
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_horizontal_overlap_add() {
        let bb1 = BoundingBox{x: 0, y: 0, width: 2, height: 1};
        let bb2 = BoundingBox{x: 1, y: 0, width: 2, height: 1};
        let expected_output = BoundingBox{x: 0, y: 0, width: 3, height: 1};
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_vertival_add() {
        let bb1 = BoundingBox{x: 0, y: 0, width: 1, height: 1};
        let bb2 = BoundingBox{x: 0, y: 1, width: 1, height: 1};
        let expected_output = BoundingBox{x: 0, y: 0, width: 1, height: 2};
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_vertival_overlap_add() {
        let bb1 = BoundingBox{x: 0, y: 0, width: 1, height: 2};
        let bb2 = BoundingBox{x: 0, y: 1, width: 1, height: 2};
        let expected_output = BoundingBox{x: 0, y: 0, width: 1, height: 3};
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_add() {
        let bb1 = BoundingBox{x: 0, y: 0, width: 1, height: 1};
        let bb2 = BoundingBox{x: 1, y: 1, width: 1, height: 1};
        let expected_output = BoundingBox{x: 0, y: 0, width: 2, height: 2};
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_overlap_add() {
        let bb1 = BoundingBox{x: 0, y: 0, width: 2, height: 2};
        let bb2 = BoundingBox{x: 1, y: 1, width: 2, height: 2};
        let expected_output = BoundingBox{x: 0, y: 0, width: 3, height: 3};
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }

    #[test]
    fn bb_diagonal_disjoint_add() {
        let bb1 = BoundingBox{x: 0, y: 0, width: 1, height: 1};
        let bb2 = BoundingBox{x: 2, y: 2, width: 1, height: 1};
        let expected_output = BoundingBox{x: 0, y: 0, width: 3, height: 3};
        assert_eq!(expected_output, &bb1 + &bb2);
        assert_eq!(expected_output, &bb2 + &bb1);
    }
}
