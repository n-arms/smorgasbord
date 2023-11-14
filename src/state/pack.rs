use crate::widgets::Size;

use super::grid::GridPosition;

#[derive(Copy, Clone, Debug)]
struct Rect {
    size: Size,
    id: usize,
}

pub struct Packer {
    /// occupation_matrix[col + row * width]
    occupation_matrix: Vec<bool>,
    /// the list of small rectangles, sorted from widest to narrowest
    small: Vec<Rect>,
    /// the big rectangle
    big: Size,
}

// Pack all the given small rectangles into one big rectangle such that none overlap.
// it fills in the big rectangle 1 row at a time. For each row it finds the widest rectangle
// that will fit and insert it, then continue until no more rectangles will fit.
// It then continues to the next row. This algorithm isn't necessarily optimal,
// but it is Good Enough™
impl Packer {
    pub fn new(big: Size, small: Vec<Size>) -> Self {
        let mut small: Vec<_> = small
            .iter()
            .copied()
            .enumerate()
            .map(|(id, size)| Rect { size, id })
            .collect();
        small.sort_by_key(|Rect { size, .. }| big.width - size.width);

        let occupation_matrix = vec![false; big.width * big.height];

        Self {
            occupation_matrix,
            small,
            big,
        }
    }

    fn index(&self, row: usize, col: usize) -> usize {
        col + row * self.big.width
    }

    fn fits(&self, size: Size, row: usize, col: usize) -> bool {
        for y in row..row + size.height {
            for x in col..col + size.width {
                if x >= self.big.width || y >= self.big.height {
                    return false;
                }
                if self.occupation_matrix[self.index(y, x)] {
                    return true;
                }
            }
        }
        false
    }

    fn fill(&mut self, size: Size, row: usize, col: usize) {
        for y in row..row + size.height {
            for x in col..col + size.width {
                let index = self.index(y, x);
                self.occupation_matrix[index] = true;
            }
        }
    }

    pub fn pack(mut self) -> Vec<GridPosition> {
        let mut positions = Vec::new();

        for row in 0..self.big.height {
            let mut to_remove = Vec::new();
            for i in 0..self.small.len() {
                let current = self.small[i];

                'fit_in_row: for col in 0..self.big.width {
                    if self.fits(current.size, row, col) {
                        self.fill(current.size, row, col);

                        positions.push((current, GridPosition { x: col, y: row }));

                        to_remove.push(i);

                        break 'fit_in_row;
                    }
                }
            }

            for index in to_remove.iter().rev() {
                self.small.remove(*index);
            }
        }

        positions.sort_by_key(|(rect, _)| rect.id);

        positions.iter().map(|(_, position)| *position).collect()
    }
}
