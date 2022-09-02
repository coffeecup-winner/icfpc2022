use std::fmt::Display;

use crate::{
    block::{Block, BlockId, Color, ComplexBlock, Point, Rect, SimpleBlock},
    canvas::Canvas,
};

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum Move {
    LineCut(BlockId, Orientation, u32),
    PointCut(BlockId, u32, u32),
    Color(BlockId, Color),
    Swap(BlockId, BlockId),
    Merge(BlockId, BlockId),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cost(pub u32);

#[derive(Debug, Clone)]
pub struct MoveError(String);

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Orientation::Horizontal => write!(f, "y"),
            Orientation::Vertical => write!(f, "x"),
        }
    }
}

impl Move {
    pub fn apply(&self, canvas: &mut Canvas) -> Cost {
        let res = match *self {
            Move::LineCut(ref block, orientation, offset) => {
                self.line_cut(canvas, block, orientation, offset)
            }
            Move::PointCut(ref block, x, y) => self.point_cut(canvas, block, x, y),
            Move::Color(ref block, c) => self.color(canvas, block, c),
            Move::Swap(ref block_a, ref block_b) => self.swap(canvas, block_a, block_b),
            Move::Merge(ref block_a, ref block_b) => self.merge(canvas, block_a, block_b),
        };
        dbg!(canvas);
        res
    }

    fn base_cost(&self) -> u32 {
        match self {
            Move::LineCut(..) => 7,
            Move::PointCut(..) => 10,
            Move::Color(..) => 5,
            Move::Swap(..) => 3,
            Move::Merge(..) => 1,
        }
    }

    fn compute_cost(&self, block_area: u32, canvas_area: u32) -> Cost {
        Cost((self.base_cost() as f32 * (canvas_area as f32 / block_area as f32)).round() as u32)
    }

    fn color(&self, canvas: &mut Canvas, block_id: &BlockId, new_color: Color) -> Cost {
        let canvas_area = canvas.area;
        let block = canvas.get_move_block_mut(block_id);
        let cost = self.compute_cost(block.size(), canvas_area);
        let (block_id, rect) = match block {
            // if the block is simple, change its color
            Block::Simple(ref mut simple) => {
                simple.c = new_color;
                return cost;
            }
            // if its complex, turn it into a simple block
            Block::Complex(ref mut complex) => (complex.id.clone(), complex.r.clone()),
        };

        *block = Block::Simple(SimpleBlock::new(block_id, rect, new_color));
        cost
    }

    fn line_cut(
        &self,
        canvas: &mut Canvas,
        block: &BlockId,
        orientation: Orientation,
        offset: u32,
    ) -> Cost {
        match orientation {
            Orientation::Horizontal => self.horizontal_cut(canvas, block, offset),
            Orientation::Vertical => self.vertical_cut(canvas, block, offset),
        }
    }

    fn vertical_cut(&self, canvas: &mut Canvas, block_id: &BlockId, cut_offset_x: u32) -> Cost {
        let block = canvas.remove_move_block(block_id);
        let cost = self.compute_cost(block.size(), canvas.area);
        if !(block.rect().bottom_left.x <= cut_offset_x && cut_offset_x < block.rect().top_right.x)
        {
            panic!(
                "Line number is out of the [{:?}]! Block is from {:?} to {:?}, point is at {:?}",
                block_id,
                block.rect().bottom_left,
                block.rect().top_right,
                cut_offset_x
            );
        }

        match block {
            Block::Simple(simple) => {
                let (left_r, right_r) = simple.r.vertical_cut(cut_offset_x);
                canvas.put_block(simple.split(0, left_r).into());
                canvas.put_block(simple.split(1, right_r).into());
            }
            Block::Complex(complex) => {
                let mut left_blocks: Vec<SimpleBlock> = vec![];
                let mut right_blocks: Vec<SimpleBlock> = vec![];
                for child in complex.bs {
                    if child.r.bottom_left.x >= cut_offset_x {
                        right_blocks.push(child);
                        continue;
                    }
                    if child.r.top_right.x <= cut_offset_x {
                        left_blocks.push(child);
                        continue;
                    }
                    let (left_r, right_r) = child.r.vertical_cut(cut_offset_x);
                    left_blocks.push(child.complex_split(left_r));
                    right_blocks.push(child.complex_split(right_r));
                }

                let (left_r, right_r) = complex.r.vertical_cut(cut_offset_x);
                canvas.put_block(
                    ComplexBlock::new(block_id.to_owned() + ".0", left_r, left_blocks).into(),
                );
                canvas.put_block(
                    ComplexBlock::new(block_id.to_owned() + ".1", right_r, right_blocks).into(),
                );
            }
        }
        cost
    }

    fn horizontal_cut(&self, canvas: &mut Canvas, block_id: &BlockId, cut_offset_y: u32) -> Cost {
        let block = canvas.remove_move_block(block_id);
        let cost = self.compute_cost(block.size(), canvas.area);
        if !(block.rect().bottom_left.y <= cut_offset_y && cut_offset_y < block.rect().top_right.y)
        {
            panic!(
                "Col number is out of the [{:?}]! Block is from {:?} to {:?}, point is at {:?}",
                block_id,
                block.rect().bottom_left,
                block.rect().top_right,
                cut_offset_y
            );
        }

        match block {
            Block::Simple(simple) => {
                let (bottom_r, top_r) = simple.r.horizontal_cut(cut_offset_y);
                canvas.put_block(simple.split(0, bottom_r).into());
                canvas.put_block(simple.split(1, top_r).into());
            }
            Block::Complex(complex) => {
                let mut bottom_blocks: Vec<SimpleBlock> = vec![];
                let mut top_blocks: Vec<SimpleBlock> = vec![];
                for child in complex.bs {
                    if child.r.bottom_left.y >= cut_offset_y {
                        top_blocks.push(child);
                        continue;
                    }
                    if child.r.top_right.y <= cut_offset_y {
                        bottom_blocks.push(child);
                        continue;
                    }
                    let (bottom_r, top_r) = child.r.horizontal_cut(cut_offset_y);
                    bottom_blocks.push(child.complex_split(bottom_r));
                    top_blocks.push(child.complex_split(top_r));
                }

                let (bottom_r, top_r) = complex.r.horizontal_cut(cut_offset_y);
                canvas.put_block(
                    ComplexBlock::new(block_id.to_owned() + ".0", bottom_r, bottom_blocks).into(),
                );
                canvas.put_block(
                    ComplexBlock::new(block_id.to_owned() + ".1", top_r, top_blocks).into(),
                );
            }
        }
        cost
    }

    fn point_cut(&self, canvas: &mut Canvas, block_id: &BlockId, cut_x: u32, cut_y: u32) -> Cost {
        let block = canvas.remove_move_block(block_id);
        let cost = self.compute_cost(block.size(), canvas.area);

        if !block.rect().contains(cut_x, cut_y) {
            panic!(
                "Point is out of [{}]! Block is from {:?} to {:?}, point is at {} {}!",
                block_id,
                block.rect().bottom_left,
                block.rect().top_right,
                cut_x,
                cut_y
            );
        }

        let complex_block = match block {
            Block::Simple(simple) => {
                let (bottom_left_bl, bottom_right_bl, top_right_bl, top_left_bl) =
                    simple.r.cross_cut(cut_x, cut_y);
                canvas.put_block(simple.split(0, bottom_left_bl).into());
                canvas.put_block(simple.split(1, bottom_right_bl).into());
                canvas.put_block(simple.split(2, top_right_bl).into());
                canvas.put_block(simple.split(3, top_left_bl).into());
                return cost;
            }
            Block::Complex(complex) => complex,
        };

        todo!()
    }

    fn swap(&self, canvas: &mut Canvas, block0: &BlockId, block1: &BlockId) -> Cost {
        // assert!(block0.rect() == block1.rect());
        // std::mem::swap(block0, block1);
        // Move::Swap(block1.id().clone(), block0.id().clone())
        todo!()
    }

    fn merge(&self, canvas: &mut Canvas, block0: &BlockId, block1: &BlockId) -> Cost {
        todo!()
    }
}

impl Canvas {
    fn get_move_block(&self, block_id: &BlockId) -> &Block {
        match self.get_block(block_id) {
            Some(block) => block,
            None => panic!("missing block: {}", block_id),
        }
    }

    fn get_move_block_mut(&mut self, block_id: &BlockId) -> &mut Block {
        match self.get_block_mut(block_id) {
            Some(block) => block,
            None => panic!("missing block: {}", block_id),
        }
    }

    fn remove_move_block(&mut self, block_id: &BlockId) -> Block {
        match self.remove_block(block_id) {
            Some(block) => block,
            None => panic!("missing block: {}", block_id),
        }
    }
}
