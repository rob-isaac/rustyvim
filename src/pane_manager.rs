use anyhow::{anyhow, Result};

#[derive(Copy, Clone)]
pub struct Pane {
    buf_num: usize,
    row0: u8,
    col0: u8,
    row1: u8,
    col1: u8,
}

const MAX_BASE_PANES: usize = 8;
const ROW_QUANTIZATION: usize = 256;
const COL_QUANTIZATION: usize = 256;
const ROW_MAX: usize = ROW_QUANTIZATION - 1;
const COL_MAX: usize = COL_QUANTIZATION - 1;

pub struct PaneManager {
    base_panes: [Option<Pane>; MAX_BASE_PANES],
    // TODO: floating panes
}

#[derive(Debug)]
enum SplitType {
    Row,
    Col,
}

impl PaneManager {
    pub fn new() -> Self {
        let mut ret = PaneManager {
            base_panes: [None; MAX_BASE_PANES],
        };
        ret.base_panes[0] = Some(Pane {
            buf_num: 0,
            row0: 0,
            col0: 0,
            row1: ROW_MAX as u8,
            col1: COL_MAX as u8,
        });
        ret
    }

    pub fn num_base_panes(&self) -> usize {
        self.base_panes
            .iter()
            .fold(0, |acc, e| acc + (e.is_some() as usize))
    }

    fn split(&mut self, pane_num: usize, split_type: SplitType) -> Result<usize> {
        let new_pane_num = self
            .base_panes
            .iter()
            .position(|e| e.is_none())
            .ok_or(anyhow!(
                "Already using maximum number of panes {}",
                MAX_BASE_PANES
            ))?;
        let mut pane = self
            .base_panes
            .get_mut(pane_num)
            .map(|opt| opt.take())
            .flatten()
            .ok_or(anyhow!("Invalid Pane Number {}", pane_num))?;

        let (lo, hi, max) = match split_type {
            SplitType::Row => (pane.row0, pane.row1, ROW_MAX),
            SplitType::Col => (pane.col0, pane.col1, COL_MAX),
        };
        let midpoint = lo + ((hi - lo) / 2);
        let (k, kN) = (hi - lo, max);
        let scaler = kN as f32 / (kN + k as usize) as f32;

        let scale_point = |point: u8| {
            if point < midpoint {
                (point as f32 * scaler) as u8
            } else if point > midpoint {
                max as u8 - ((max as u8 - point) as f32 * scaler) as u8
            } else {
                point
            }
        };

        for maybe_pane in self.base_panes.iter_mut() {
            if let Some(pane) = maybe_pane {
                let (lo, hi) = match split_type {
                    SplitType::Row => (&mut pane.row0, &mut pane.row1),
                    SplitType::Col => (&mut pane.col0, &mut pane.col1),
                };
                *lo = scale_point(*lo);
                *hi = scale_point(*hi);
            }
        }
        let (lo, hi) = (scale_point(lo), scale_point(hi));
        let midpoint = lo + ((hi - lo) / 2);

        let mut new_pane = pane.clone();
        match split_type {
            SplitType::Row => {
                pane.row0 = lo;
                pane.row1 = midpoint;
                new_pane.row0 = midpoint;
                new_pane.row1 = hi;
            }
            SplitType::Col => {
                pane.col0 = lo;
                pane.col1 = midpoint;
                new_pane.col0 = midpoint;
                new_pane.col1 = hi;
            }
        }
        self.base_panes[pane_num] = Some(pane);
        self.base_panes[new_pane_num] = Some(new_pane);

        Ok(new_pane_num)
    }

    pub fn hsplit(&mut self, pane_num: usize) -> Result<usize> {
        self.split(pane_num, SplitType::Row)
    }

    pub fn vsplit(&mut self, pane_num: usize) -> Result<usize> {
        self.split(pane_num, SplitType::Col)
    }
}

// TODO: The rescaling heuristic on a split should be the following:
// Determine the percentage of the axis taken up by the window we are splitting as 1/N
// Scale splits along that axis to the left and right by N/(N+1), squishing them towards edges
// Split the newly sized window in half
//
// 1/N = k/kN => N/(N+1) = kN/(kN + k) so for our quantized scheme of k/M we can scale by
// kM/(kM + k)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vsplit_half() {
        let mut pane_manager = PaneManager::new();
        assert_eq!(pane_manager.num_base_panes(), 1);
        let left_pane = 0;
        let right_pane = pane_manager.vsplit(left_pane).unwrap();
        assert_eq!(pane_manager.num_base_panes(), 2);

        for pane in [left_pane, right_pane] {
            assert_eq!(pane_manager.base_panes[pane].unwrap().row0, 0);
            assert_eq!(pane_manager.base_panes[pane].unwrap().row1, ROW_MAX as u8);
        }

        assert_eq!(pane_manager.base_panes[left_pane].unwrap().col0, 0);
        assert_eq!(
            pane_manager.base_panes[left_pane].unwrap().col1,
            (COL_MAX / 2) as u8
        );
        assert_eq!(
            pane_manager.base_panes[right_pane].unwrap().col0,
            (COL_MAX / 2) as u8
        );
        assert_eq!(
            pane_manager.base_panes[right_pane].unwrap().col1,
            COL_MAX as u8
        );
    }

    #[test]
    fn vsplit_thirds() {
        let mut pane_manager = PaneManager::new();
        assert_eq!(pane_manager.num_base_panes(), 1);
        let left_pane = 0;
        let mid_pane = pane_manager.vsplit(left_pane).unwrap();
        let right_pane = pane_manager.vsplit(mid_pane).unwrap();
        assert_eq!(pane_manager.num_base_panes(), 3);

        for pane in [left_pane, mid_pane, right_pane] {
            assert_eq!(pane_manager.base_panes[pane].unwrap().row0, 0);
            assert_eq!(pane_manager.base_panes[pane].unwrap().row1, ROW_MAX as u8);
        }

        const LEFT_BOUNDARY: u8 = (COL_MAX / 3) as u8 - 1; // -1 due to quantization error
        const RIGHT_BOUNDARY: u8 = (COL_MAX * 2 / 3) as u8 - 1; // -1 due to quantization error

        assert_eq!(pane_manager.base_panes[left_pane].unwrap().col0, 0);
        assert_eq!(
            pane_manager.base_panes[left_pane].unwrap().col1,
            LEFT_BOUNDARY
        );

        assert_eq!(
            pane_manager.base_panes[mid_pane].unwrap().col0,
            LEFT_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[mid_pane].unwrap().col1,
            RIGHT_BOUNDARY
        );

        assert_eq!(
            pane_manager.base_panes[right_pane].unwrap().col0,
            RIGHT_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[right_pane].unwrap().col1,
            COL_MAX as u8
        );
    }

    #[test]
    fn split() {
        let mut pane_manager = PaneManager::new();
        assert_eq!(pane_manager.num_base_panes(), 1);
        let left_pane = 0;
        let mid_top_pane = pane_manager.vsplit(left_pane).unwrap();
        let right_pane = pane_manager.vsplit(mid_top_pane).unwrap();
        let mid_mid_pane = pane_manager.hsplit(mid_top_pane).unwrap();
        let mid_bottom_pane = pane_manager.hsplit(mid_mid_pane).unwrap();

        const LEFT_BOUNDARY: u8 = (COL_MAX / 3) as u8 - 1; // -1 due to quantization error
        const RIGHT_BOUNDARY: u8 = (COL_MAX * 2 / 3) as u8 - 1; // -1 due to quantization error
        const TOP_BOUNDARY: u8 = (ROW_MAX / 3) as u8 - 1; // -1 due to quantization error
        const BOTTOM_BOUNDARY: u8 = (ROW_MAX * 2 / 3) as u8 - 1; // -1 due to quantization error

        assert_eq!(pane_manager.base_panes[left_pane].unwrap().row0, 0);
        assert_eq!(pane_manager.base_panes[left_pane].unwrap().col0, 0);
        assert_eq!(
            pane_manager.base_panes[left_pane].unwrap().row1,
            ROW_MAX as u8
        );
        assert_eq!(
            pane_manager.base_panes[left_pane].unwrap().col1,
            LEFT_BOUNDARY
        );

        assert_eq!(pane_manager.base_panes[mid_top_pane].unwrap().row0, 0);
        assert_eq!(
            pane_manager.base_panes[mid_top_pane].unwrap().col0,
            LEFT_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[mid_top_pane].unwrap().row1,
            TOP_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[mid_top_pane].unwrap().col1,
            RIGHT_BOUNDARY
        );

        assert_eq!(
            pane_manager.base_panes[mid_mid_pane].unwrap().row0,
            TOP_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[mid_mid_pane].unwrap().col0,
            LEFT_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[mid_mid_pane].unwrap().row1,
            BOTTOM_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[mid_mid_pane].unwrap().col1,
            RIGHT_BOUNDARY
        );

        assert_eq!(
            pane_manager.base_panes[mid_bottom_pane].unwrap().row0,
            BOTTOM_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[mid_bottom_pane].unwrap().col0,
            LEFT_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[mid_bottom_pane].unwrap().row1,
            ROW_MAX as u8
        );
        assert_eq!(
            pane_manager.base_panes[mid_bottom_pane].unwrap().col1,
            RIGHT_BOUNDARY
        );

        assert_eq!(pane_manager.base_panes[right_pane].unwrap().row0, 0);
        assert_eq!(
            pane_manager.base_panes[right_pane].unwrap().col0,
            RIGHT_BOUNDARY
        );
        assert_eq!(
            pane_manager.base_panes[right_pane].unwrap().row1,
            ROW_MAX as u8
        );
        assert_eq!(
            pane_manager.base_panes[right_pane].unwrap().col1,
            COL_MAX as u8
        );
    }
}
