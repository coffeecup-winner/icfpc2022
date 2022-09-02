use std::collections::HashMap;

use crate::{
    block::{Block, Color, Point, Rect, SimpleBlock},
    moves::{Cost, Move},
};

use super::Solver;

pub struct TopColor {}

impl Solver for TopColor {
    fn name(&self) -> &'static str {
        "top_color"
    }

    fn solve_core(
        &self,
        canvas: &mut crate::canvas::Canvas,
        painting: &crate::painting::Painting,
    ) -> (Vec<Move>, crate::moves::Cost) {
        let mut colors: HashMap<Color, u32> = HashMap::new();

        for x in 0..painting.width() {
            for y in 0..painting.height() {
                let pixel = painting.get_color(x, y);
                match colors.get(&pixel) {
                    Some(count) => {
                        colors.insert(pixel, count + 1);
                    }
                    None => {
                        colors.insert(pixel, 0);
                    }
                }
            }
        }

        let (top_color, top_color_count) = colors
            .into_iter()
            .max_by(|(_, count1), (_, count2)| count1.cmp(count2))
            .expect("TopColor solver: colors hash map is empty");

        println!(
            "TopColor solver: color={} with {} pixels",
            top_color, top_color_count
        );

        let mov_id = "0".to_string();
        let mov = Move::Color(mov_id, top_color);
        let cost = mov
            .apply(canvas)
            .expect("TopColor solver: couldn't perform color move"); // TODO refactor to return error, it's rust or javascript after all

        (vec![mov], cost)
    }
}