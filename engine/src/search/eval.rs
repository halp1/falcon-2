use serde::{Deserialize, Serialize};
use triangle::types::game::Spin;

use crate::game::{BOARD_WIDTH, Game, HoleData};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DangerWeights {
  pub well_depth: f64,
  pub outer_height: f64,
  pub inner_height: f64,
  pub unevenness: f64,
  pub upper_holes: f64,
}

impl DangerWeights {
  pub fn eval(&self, game: &Game) -> f64 {
    let heights = &game.board.column_heights();
    let well = game.board.well(heights);

    let mut score = 0f64;

    score += match well {
      Some(idx) => self.well_depth as f64 * game.board.well_depth(heights, idx) as f64,
      None => 0.0,
    };

    let (outer, inner) = game.board.heights();
    score += self.outer_height as f64 * outer as f64;
    score += self.inner_height as f64 * inner as f64;

    score += self.unevenness as f64 * game.board.unevenness(heights, well) as f64;

    let mut board = game.board.clone();
    board.cols.iter_mut().for_each(|col| *col = *col >> 12);
    score += self.upper_holes * board.count_holes(&board.column_heights()) as f64;

    score
  }

  #[inline(always)]
  pub fn as_array(&self) -> [f64; 5] {
    [
      self.well_depth,
      self.outer_height,
      self.inner_height,
      self.unevenness,
      self.upper_holes,
    ]
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weights {
  pub outer_height: f64,
  pub inner_height: f64,
  pub unevenness: f64,

  pub wells: [f64; BOARD_WIDTH],

  pub holes: HoleData<f64>,

  // spins
  pub clear: [[f64; 4]; 3],

  // pub t_hole: f64,
  // pub i_hole: f64,

  // // waste mino type
  // pub waste: [f64; 7],
  pub sent: f64,

  pub b2b: f64,
  pub combo: f64,

  pub opponent_danger: DangerWeights,
  pub danger_prophecy: f64,
  pub killpower: f64,
}

impl HoleData<f64> {
  pub fn eval(&self, other: &HoleData<u32>) -> f64 {
    self.holes * other.holes as f64
      + self.depth * other.depth as f64
      + self.accessible * other.accessible as f64
      + self.inaccessible * other.inaccessible as f64
  }
}

impl<T> HoleData<T> {
  pub fn as_array(&self) -> [T; 4]
  where
    T: Copy,
  {
    [self.holes, self.depth, self.accessible, self.inaccessible]
  }
}

pub struct MoveInfo {
  pub clear: (Spin, u8),
  pub sent: u16,
  pub attack: u16,
  pub time: u8,
}

impl Weights {
  pub fn eval(self: &Self, state: &Game, move_info: &MoveInfo, opponent_danger: f64) -> f64 {
    let mut score = 0f64;

    let heights = &state.board.column_heights();
    let well = state.board.well(heights);

    score += match well {
      Some(idx) => self.wells[idx],
      None => 0.0,
    };

    let (outer, inner) = state.board.heights();
    score += self.outer_height * outer as f64;
    score += self.inner_height * inner as f64;

    score += self.unevenness * state.board.unevenness(heights, well) as f64;

    score += if move_info.clear.1 == 0 {
      0.0
    } else {
      self.clear[move_info.clear.0 as usize][move_info.clear.1 as usize - 1]
    };

    score += self.holes.eval(&state.board.holes(heights));

    // TODO: waste mino type

    score += self.sent * move_info.sent as f64;

    score += self.b2b * (state.b2b + 1) as f64;
    score += self.combo * (state.combo + 1) as f64;

    let kill_time_factor = ((self.danger_prophecy - move_info.time as f64).max(0.0)
      / (self.danger_prophecy - 1.0).max(1.0))
    .min(1.0);

    score += self.killpower * move_info.sent as f64 * kill_time_factor;

    score
  }

  #[inline(always)]
  pub fn eval_opponent(self: &Self, opponent: &Game) -> f64 {
    self.opponent_danger.eval(opponent)
  }
}

impl Into<Vec<f64>> for Weights {
  fn into(self) -> Vec<f64> {
    let mut v = Vec::new();
    v.push(self.outer_height);
    v.push(self.inner_height);
    v.push(self.unevenness);
    v.extend_from_slice(&self.wells);
    v.extend_from_slice(&self.holes.as_array());
    for i in 0..3 {
      for j in 0..4 {
        v.push(self.clear[i][j]);
      }
    }
    // v.push(self.t_hole);
    // v.push(self.i_hole);
    // v.extend_from_slice(&self.waste);
    v.push(self.sent);
    v.push(self.b2b);
    v.push(self.combo);

    v.extend_from_slice(&self.opponent_danger.as_array());

    v.push(self.danger_prophecy);
    v.push(self.killpower);

    v
  }
}

impl Into<Weights> for Vec<f64> {
  fn into(self) -> Weights {
    let mut iter = self.into_iter();
    let mut v = || iter.next().unwrap_or(0.0);
    Weights {
      outer_height: v(),
      inner_height: v(),
      unevenness: v(),
      wells: [v(), v(), v(), v(), v(), v(), v(), v(), v(), v()],
      holes: HoleData {
        holes: v(),
        depth: v(),
        accessible: v(),
        inaccessible: v(),
      },
      clear: [
        [v(), v(), v(), v()],
        [v(), v(), v(), v()],
        [v(), v(), v(), v()],
      ],
      // t_hole: v(),
      // i_hole: v(),
      // waste: [v(), v(), v(), v(), v(), v(), v()],
      sent: v(),
      b2b: v(),
      combo: v(),
      opponent_danger: DangerWeights {
        well_depth: v(),
        outer_height: v(),
        inner_height: v(),
        unevenness: v(),
        upper_holes: v(),
      },
      danger_prophecy: v(),
      killpower: v(),
    }
  }
}

pub const WEIGHTS_HANDTUNED: Weights = Weights {
  outer_height: -50.0,
  inner_height: -100.0,
  unevenness: -3.0,

  wells: [
    -20.0, -30.0, -10.0, 30.0, 20.0, 20.0, 30.0, -10.0, -30.0, -20.0,
  ],

  clear: [
    [-10.0, -10.0, -10.0, 50.0],
    [20.0, 25.0, 30.0, 60.0],
    [40.0, 80.0, 120.0, 180.0],
  ],

  sent: 20.0,

  b2b: 8.0,
  combo: 3.0,

  holes: HoleData {
    holes: -10.0,
    depth: -5.0,
    accessible: 0.0,
    inaccessible: -20.0,
  },

  // i_hole: 0.0,
  // t_hole: 0.0,

  // waste: [0.0; 7],
  opponent_danger: DangerWeights {
    well_depth: -20.0,
    outer_height: -10.0,
    inner_height: -20.0,
    unevenness: -1.0,
    upper_holes: -5.0,
  },

  danger_prophecy: 20.0,
  killpower: 10.0,
};

pub const WEIGHTS_ZERO: Weights = Weights {
  outer_height: 0.0,
  inner_height: 0.0,
  unevenness: 0.0,

  wells: [0.0; BOARD_WIDTH],

  holes: HoleData {
    holes: 0.0,
    depth: 0.0,
    accessible: 0.0,
    inaccessible: 0.0,
  },

  clear: [[0.0; 4]; 3],

  sent: 0.0,

  b2b: 0.0,
  combo: 0.0,

  opponent_danger: DangerWeights {
    well_depth: 0.0,
    outer_height: 0.0,
    inner_height: 0.0,
    unevenness: 0.0,
    upper_holes: 0.0,
  },

  danger_prophecy: 0.0,
  killpower: 0.0,
};
