use crate::{
    block::{Block, BlockId},
    canvas::Canvas,
    moves::{AppliedMove, Move},
    painting::Painting,
};

use super::Solver;

pub struct Swapper;

impl Solver for Swapper {
    fn name(&self) -> &'static str {
        "swapper"
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let mut applied_moves = vec![];
        loop {
            let mut best_painting_score = painting.calculate_score_canvas(canvas);
            let mut best_move = None;
            let b0_id = self.find_worst_block_id(canvas, painting);
            let b0 = canvas.get_block(b0_id).unwrap();
            for b1 in canvas.blocks_iter() {
                if b0.get_id() == b1.get_id() {
                    continue;
                }
                if let (Block::Simple(b0), Block::Simple(b1)) = (b0, b1) {
                    if b0.c == b1.c
                        || b0.r.width() != b1.r.width()
                        || b0.r.height() != b1.r.height()
                    {
                        continue;
                    }

                    let mov = Move::Swap(b0.id.to_owned(), b1.id.to_owned());
                    let mut canvas_temp = canvas.clone();
                    let am = mov.clone().apply(&mut canvas_temp).unwrap();
                    let score = painting.calculate_score_canvas(&canvas_temp);
                    if (score.0 + am.cost.0) < best_painting_score.0 {
                        best_painting_score = score;
                        best_move = Some(mov);
                    }
                }
            }
            if let Some(mov) = best_move {
                let am = mov.apply(canvas).unwrap();
                applied_moves.push(am);
            } else {
                break;
            }
        }
        applied_moves
    }
}

impl Swapper {
    fn find_worst_block_id<'a>(&self, canvas: &'a Canvas, painting: &Painting) -> &'a BlockId {
        let mut worst_block = None;
        let mut worst_score = 0.0;
        for b in canvas.blocks_iter() {
            if let Block::Simple(b) = b {
                let score = painting.calculate_score_block(b);
                if score > worst_score {
                    worst_score = score;
                    worst_block = Some(&b.id);
                }
            }
        }
        worst_block.unwrap()
    }
}