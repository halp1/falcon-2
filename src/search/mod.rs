#![allow(unused_variables)]

use std::{collections::HashSet, time::Instant};

use crate::game::{
  self, BOARD_WIDTH, Game, GameConfig,
  data::{Mino, Move, Spin},
};

pub mod eval;
use eval::{Weights, eval};

const MOVES: [[Move; 6]; 7] = [
  // None
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
  ],
  // Left
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::SoftDrop,
    Move::None,
  ],
  // Right
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Right,
    Move::SoftDrop,
    Move::None,
  ],
  // Softdrop
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::None,
  ],
  // CCW
  [
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::None,
  ],
  // CW
  [
    Move::CW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::None,
  ],
  // Flip
  [
    Move::CW,
    Move::CCW,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::None,
  ],
];

pub fn expand(
  mut state: &mut Game,
  config: &GameConfig,
  passed: &mut [u64; 2048],
  res: &mut [(u8, u8, u8, Spin); 512],
) -> (usize, u64) {
  passed.iter_mut().for_each(|m| *m = 0);

  let mut queue = [(0, 0, 0, Spin::None, Move::None); 512];

  let mut front_ptr = 0;
  let mut back_ptr = 1;
  let mut res_ptr = 0;

  let mut nodes = 0u64;

  queue[0] = (
    state.piece.x,
    state.piece.y,
    state.piece.rot,
    Spin::None,
    Move::None,
  );

  while front_ptr < back_ptr {
    let (x, y, rot, spin, prev_mv) = queue[front_ptr];
    front_ptr += 1;

    for &mv in &MOVES[prev_mv as usize] {
      nodes += 1;
      if mv == Move::None {
        break;
      }
      // Don't do these checks, because running the checks is more expensive
      // if (last == Move::CCW && mv == Move::CW)
      //     || (last == Move::CW && mv == Move::CCW)
      //     || (last == Move::Flip && mv == Move::Flip)
      //     || (last == Move::Left && mv == Move::Right)
      //     || (last == Move::Right && mv == Move::Left)
      //     || (last == Move::SoftDrop && mv == Move::SoftDrop)
      // {
      //     continue;
      // }

      state.piece.x = x;
      state.piece.y = y;
      state.piece.rot = rot;
      state.spin = spin;

      let fail = !mv.run(&mut state, config);

      let mut compressed =
        0u16 | (state.piece.x as u16 & 0b_1111) | ((state.piece.y as u16 & 0b_111111) << 4);

      if state.piece.mino != Mino::O {
        compressed |= ((state.piece.rot as u16 & 0b11) << 10) | ((state.spin as u16 & 0b11) << 12);
      }

      let idx = compressed as usize / 64;
      let bit = 1 << (compressed % 64);

      if mv == Move::SoftDrop && passed[1024 + idx] & bit == 0 {
        passed[1024 + idx] |= bit;
        res[res_ptr] = (state.piece.x, state.piece.y, state.piece.rot, state.spin);

        res_ptr += 1;
      }

      if fail || passed[idx] & bit != 0 {
        continue;
      }

      passed[idx] |= bit;

      queue[back_ptr] = (
        state.piece.x,
        state.piece.y,
        state.piece.rot,
        state.spin,
        mv,
      );
      back_ptr += 1;
    }
  }

  (res_ptr, nodes)
}

#[derive(Clone, Debug)]
struct SearchState {
  pub game: Game,
  pub depth: u8,
  pub lines_sent: u16,
  pub clears: Vec<Spin>,
  pub first_move: Option<(u8, u8, u8, bool, Spin)>,
}

pub fn search(
  state: Game,
  config: &GameConfig,
  max_depth: u8,
  weights: &Weights,
) -> Option<((u8, u8, u8, bool, Spin), Game)> {
  let mut best_result: Option<(Game, i32, (u8, u8, u8, bool, Spin))> = None;

  let mut queue: Vec<SearchState> = Vec::with_capacity(2usize.pow(19));

  let mut passed: HashSet<[u64; BOARD_WIDTH]> = HashSet::with_capacity(2usize.pow(20));

  let mut expand_passed = [0u64; 2048];
  let mut expand_res = [(0u8, 0u8, 0u8, Spin::None); 512];

  queue.push(SearchState {
    game: state,
    depth: 0,
    lines_sent: 0,
    clears: Vec::with_capacity(16),
    first_move: None,
  });
  let mut ptr = 0;

  let mut nodes = 0u64;
  let start = Instant::now();

  while ptr < queue.len() {
    let mut game_copy = queue[ptr].game.clone();
    let depth = queue[ptr].depth;
    let lines_sent = queue[ptr].lines_sent;
    let first_move = queue[ptr].first_move;
    let mut clears = queue[ptr].clears.clone();
    ptr += 1;

    let moves = expand(&mut game_copy, config, &mut expand_passed, &mut expand_res);

    if depth >= max_depth - 1 {
      for i in 0..moves.0 {
        let (x, y, rot, spin) = expand_res[i];
        game_copy.piece.x = x;
        game_copy.piece.y = y;
        game_copy.piece.rot = rot;
        game_copy.spin = spin;
        let (lines, clear) = game_copy.hard_drop(config);
        if let Some(c) = clear {
          clears.push(c);
        }
        nodes += 1;
        if game_copy.topped_out() {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }
        let score = eval(weights, &game_copy, lines_sent + lines, clears.clone());
        if best_result.is_none() || score > best_result.as_ref().unwrap().1 {
          best_result = Some((
            game_copy.clone(),
            score,
            first_move.unwrap_or((x, y, rot, false, spin)),
          ));
        }

        game_copy = queue[ptr - 1].game.clone();
      }
    } else {
      for i in 0..moves.0 {
        let (x, y, rot, spin) = expand_res[i];
        game_copy.piece.x = x;
        game_copy.piece.y = y;
        game_copy.piece.rot = rot;
        game_copy.spin = spin;
        let (lines, clear) = game_copy.hard_drop(config);
        if !passed.insert(game_copy.board.cols.clone()) {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }

        game_copy.regen_collision_map();

        nodes += 1;

        if game_copy.topped_out() {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }

        if let Some(c) = clear {
          clears.push(c);
        }

        queue.push(SearchState {
          game: game_copy.clone(),
          depth: depth + 1,
          lines_sent: lines_sent + lines,
          clears: clears.clone(),
          first_move: if first_move != None {
            first_move
          } else {
            Some((x, y, rot, false, spin))
          },
        });

        game_copy = queue[ptr - 1].game.clone();
      }
    }
  }

  let elapsed = start.elapsed();

  println!("Total nodes evaluated: {}", nodes);
  println!("NPS: {}", nodes as f32 / elapsed.as_secs_f32());

  if let Some(best) = best_result {
    Some((best.2, best.0))
  } else {
    None
  }
}

// beam search 🥶

use std::cmp::Reverse;
use std::collections::BinaryHeap;

// Assumes `SearchState`, `Game`, `GameConfig`, `Spin`,
// `expand(&mut Game, &GameConfig, &mut [u64;2048], &mut [(u8,u8,u8,Spin);512]) -> (usize, _)`,
// and `eval(&Game, u16, Vec<Spin>) -> i32` are defined elsewhere.

const BEAM_WIDTH: usize = 200;

#[derive(Clone)]
struct Candidate {
  state: SearchState,
  score: i32,
}

impl PartialEq for Candidate {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score
  }
}
impl Eq for Candidate {}
impl PartialOrd for Candidate {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
impl Ord for Candidate {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    // Min-heap via Reverse: higher scores are “greater”
    self.score.cmp(&other.score)
  }
}

/// Beam-search replacement for BFS-based Tetris search.
///
/// Signature matches the original `search` function.
pub fn beam_search(
  root_game: Game,
  config: &GameConfig,
  max_depth: u8,
  weights: &Weights,
) -> Option<((u8, u8, u8, bool, Spin), Game)> {
  // Initial SearchState
  let init_state = SearchState {
    game: root_game.clone(),
    depth: 0,
    lines_sent: 0,
    clears: Vec::new(),
    first_move: None,
  };
  let init_score = eval(weights, &root_game, 0, Vec::new());

  // Beam as min-heap of size BEAM_WIDTH
  let mut beam: BinaryHeap<Reverse<Candidate>> = BinaryHeap::with_capacity(BEAM_WIDTH);
  beam.push(Reverse(Candidate {
    state: init_state,
    score: init_score,
  }));

  let mut passed = [0u64; 2048];
  let mut res_buf = [(0u8, 0u8, 0u8, Spin::None); 512];

  // Iterate placements
  for depth in 0..max_depth {
    let mut next_beam: BinaryHeap<Reverse<Candidate>> = BinaryHeap::with_capacity(BEAM_WIDTH);

    while let Some(Reverse(cand)) = beam.pop() {
      for n in 0..=1 {
        let mut game_copy = cand.state.game.clone();

        if n == 1 {
          game_copy.hold();
          game_copy.regen_collision_map();
        }
        // Expand moves
        let moves = expand(&mut game_copy, config, &mut passed, &mut res_buf);

        for i in 0..moves.0 {
          let (x, y, rot, spin) = res_buf[i];
          let mut g2 = game_copy.clone();
          g2.piece.x = x;
          g2.piece.y = y;
          g2.piece.rot = rot;
          g2.spin = spin;

          let (lines, clear) = g2.hard_drop(config);
          g2.regen_collision_map();
          if g2.topped_out() {
            continue;
          }

          // Build next SearchState
          let mut next_clears = cand.state.clears.clone();
          if let Some(c) = clear {
            next_clears.push(c);
          }
          let next_sent = cand.state.lines_sent + lines;
          let next_depth = cand.state.depth + 1;
          let next_first = cand.state.first_move.or(Some((x, y, rot, n == 1, spin)));

          let next_state = SearchState {
            game: g2.clone(),
            depth: next_depth,
            lines_sent: next_sent,
            clears: next_clears.clone(),
            first_move: next_first,
          };

          let score = eval(weights, &g2, next_sent, next_clears);
          let candidate = Candidate {
            state: next_state,
            score,
          };

          // Insert into next beam with pruning
          if next_beam.len() < BEAM_WIDTH {
            next_beam.push(Reverse(candidate));
          } else if let Some(Reverse(worst)) = next_beam.peek() {
            if score > worst.score {
              next_beam.pop();
              next_beam.push(Reverse(candidate));
            }
          }
        }
      }
    }

    if next_beam.is_empty() {
      break;
    }
    beam = next_beam;
  }

  // Select best final candidate
  beam
    .into_iter()
    .map(|rev| rev.0)
    .max_by_key(|cand| cand.score)
    .and_then(|cand| cand.state.first_move.map(|m| (m, cand.state.game)))
}

// A* search implementation
use std::collections::HashMap;

#[derive(Clone)]
struct AStarNode {
  state: SearchState,
  f_cost: i32,
  g_cost: i32,
}

impl PartialEq for AStarNode {
  fn eq(&self, other: &Self) -> bool {
    self.f_cost == other.f_cost
  }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for AStarNode {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.f_cost.cmp(&other.f_cost)
  }
}

fn board_hash(board: &[u64; BOARD_WIDTH]) -> u64 {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};
  
  let mut hasher = DefaultHasher::new();
  board.hash(&mut hasher);
  hasher.finish()
}

/// A* search for optimal Tetris play.
/// Uses evaluation function as heuristic for maximum performance.
pub fn astar_search(
  root_game: Game,
  config: &GameConfig,
  max_depth: u8,
  weights: &Weights,
) -> Option<((u8, u8, u8, bool, Spin), Game)> {
  let start = Instant::now();
  
  // Priority queue (min-heap) for A* open set
  let mut open_set: BinaryHeap<Reverse<AStarNode>> = BinaryHeap::with_capacity(16384);
  
  // Track visited board states with their best g_cost to avoid revisiting worse paths
  let mut visited: HashMap<u64, (i32, u8)> = HashMap::with_capacity(32768);
  
  // Reusable buffers for expand function
  let mut expand_passed = [0u64; 2048];
  let mut expand_res = [(0u8, 0u8, 0u8, Spin::None); 512];
  
  // Initialize with root state
  let init_state = SearchState {
    game: root_game.clone(),
    depth: 0,
    lines_sent: 0,
    clears: Vec::new(),
    first_move: None,
  };
  
  let init_eval = eval(weights, &root_game, 0, Vec::new());
  let init_heuristic = -init_eval * (max_depth as i32 - 0);
  
  let init_node = AStarNode {
    state: init_state,
    f_cost: init_heuristic,
    g_cost: 0,
  };
  
  open_set.push(Reverse(init_node));
  
  let mut nodes_expanded = 0u64;
  let mut best_result: Option<(Game, i32, (u8, u8, u8, bool, Spin))> = None;

  while let Some(Reverse(current)) = open_set.pop() {
    nodes_expanded += 1;
    
    // If we've reached max depth, evaluate and potentially update best
    if current.state.depth >= max_depth {
      let score = eval(weights, &current.state.game, current.state.lines_sent, current.state.clears.clone());
      if best_result.is_none() || score > best_result.as_ref().unwrap().1 {
        if let Some(first_move) = current.state.first_move {
          best_result = Some((current.state.game.clone(), score, first_move));
        }
      }
      continue;
    }
    
    // Get board hash for visited check
    let board_hash_val = board_hash(&current.state.game.board.cols);
    
    // Skip if we've already visited this state with better or equal g_cost at same/better depth
    if let Some(&(best_g, best_depth)) = visited.get(&board_hash_val) {
      if best_g <= current.g_cost && best_depth <= current.state.depth {
        continue;
      }
    }
    
    visited.insert(board_hash_val, (current.g_cost, current.state.depth));
    
    // Expand current node with and without hold
    for use_hold in [false, true] {
      let mut game_copy = current.state.game.clone();
      
      if use_hold {
        game_copy.hold();
        game_copy.regen_collision_map();
      }
      
      // Get all possible moves
      let (num_moves, _) = expand(&mut game_copy, config, &mut expand_passed, &mut expand_res);
      
      for i in 0..num_moves {
        let (x, y, rot, spin) = expand_res[i];
        let mut next_game = game_copy.clone();
        next_game.piece.x = x;
        next_game.piece.y = y;
        next_game.piece.rot = rot;
        next_game.spin = spin;
        
        let (lines, clear) = next_game.hard_drop(config);
        next_game.regen_collision_map();
        
        // Skip if game over
        if next_game.topped_out() {
          continue;
        }
        
        // Build successor state
        let mut next_clears = current.state.clears.clone();
        if let Some(c) = clear {
          next_clears.push(c);
        }
        
        let next_sent = current.state.lines_sent + lines;
        let next_depth = current.state.depth + 1;
        let next_first = current.state.first_move.or(Some((x, y, rot, use_hold, spin)));
        
        let next_state = SearchState {
          game: next_game.clone(),
          depth: next_depth,
          lines_sent: next_sent,
          clears: next_clears.clone(),
          first_move: next_first,
        };
        
        // Calculate costs
        let step_cost = 1; // Uniform cost per move
        let next_g_cost = current.g_cost + step_cost;
        
        // Use negative evaluation as heuristic (higher eval = lower h_cost)
        let eval_score = eval(weights, &next_game, next_sent, next_clears);
        let remaining_depth = max_depth as i32 - next_depth as i32;
        let h_cost = -eval_score * remaining_depth.max(1);
        
        let f_cost = next_g_cost + h_cost;
        
        let next_node = AStarNode {
          state: next_state,
          f_cost,
          g_cost: next_g_cost,
        };
        
        open_set.push(Reverse(next_node));
      }
    }
    
    // Early termination if we have good solution and searched enough
    if nodes_expanded > 50000 && best_result.is_some() {
      break;
    }
  }
  
  let elapsed = start.elapsed();
  println!("A* nodes expanded: {}", nodes_expanded);
  println!("A* NPS: {}", nodes_expanded as f32 / elapsed.as_secs_f32());
  
  best_result.map(|(game, _, first_move)| (first_move, game))
}
