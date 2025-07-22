import { Game, BOARD_WIDTH } from "../game/index.js";
import { GameConfig, Mino, Move, Spin, MoveData } from "../game/data.js";
import { evaluate, Weights } from "./eval.js";

const MOVES: Move[][] = [
  [Move.CW, Move.CCW, Move.Flip, Move.Left, Move.Right, Move.SoftDrop],
  [Move.CW, Move.CCW, Move.Flip, Move.Left, Move.SoftDrop, Move.None],
  [Move.CW, Move.CCW, Move.Flip, Move.Right, Move.SoftDrop, Move.None],
  [Move.CW, Move.CCW, Move.Flip, Move.Left, Move.Right, Move.None],
  [Move.CCW, Move.Flip, Move.Left, Move.Right, Move.SoftDrop, Move.None],
  [Move.CW, Move.Flip, Move.Left, Move.Right, Move.SoftDrop, Move.None],
  [Move.CW, Move.CCW, Move.Left, Move.Right, Move.SoftDrop, Move.None],
];

function movesToIndex(move: Move): number {
  switch (move) {
    case Move.None:
      return 0;
    case Move.Left:
      return 1;
    case Move.Right:
      return 2;
    case Move.SoftDrop:
      return 3;
    case Move.CCW:
      return 4;
    case Move.CW:
      return 5;
    case Move.Flip:
      return 6;
    default:
      return 0;
  }
}

export function expand(
  state: Game,
  config: GameConfig,
  passed: number[],
  res: Array<[number, number, number, Spin]>
): [number, number] {
  for (let i = 0; i < passed.length; i++) {
    passed[i] = 0;
  }

  const queue: Array<[number, number, number, Spin, Move]> = Array(512).fill([
    0,
    0,
    0,
    Spin.None,
    Move.None,
  ]);
  let frontPtr = 0;
  let backPtr = 1;
  let resPtr = 0;
  let nodes = 0;
  queue[0] = [state.piece.x, state.piece.y, state.piece.rot, Spin.None, Move.None];
  while (frontPtr < backPtr) {
    const [x, y, rot, spin, prev] = queue[frontPtr++];
    for (const mv of MOVES[movesToIndex(prev)]) {
      nodes++;

      if (mv === Move.None) break;

      state.piece.x = x;
      state.piece.y = y;
      state.piece.rot = rot;
      state.spin = spin;

      const fail = !MoveData.run(mv, state, config);

      let compressed = x | (y << 4);
      if (state.piece.mino !== Mino.O) {
        compressed |= (rot << 10) | (state.spin << 12);
      }
      const idx = compressed >> 6;
      const bit = 1 << (compressed & 63);
      if (mv === Move.SoftDrop && (passed[1024 + idx] & bit) === 0) {
        passed[1024 + idx] |= bit;
        res[resPtr++] = [state.piece.x, state.piece.y, state.piece.rot, state.spin];
      }
      if (fail || (passed[idx] & bit) !== 0) continue;
      passed[idx] |= bit;
      queue[backPtr++] = [state.piece.x, state.piece.y, state.piece.rot, state.spin, mv];
    }
  }
  return [resPtr, nodes];
}

class SearchState {
  public game: Game;
  public depth: number;
  public linesSent: number;
  public clears: Spin[];
  public firstMove: [number, number, number, boolean, Spin] | null;

  constructor(
    game: Game,
    depth: number,
    linesSent: number,
    clears: Spin[],
    firstMove: [number, number, number, boolean, Spin] | null
  ) {
    this.game = game;
    this.depth = depth;
    this.linesSent = linesSent;
    this.clears = clears;
    this.firstMove = firstMove;
  }

  clone(): SearchState {
    return new SearchState(
      this.game.clone(),
      this.depth,
      this.linesSent,
      [...this.clears],
      this.firstMove ? [...this.firstMove] : null
    );
  }
}

export function search(
  state: Game,
  config: GameConfig,
  maxDepth: number,
  weights: Weights
): [[number, number, number, boolean, Spin], Game] | null {
  if (state.toppedOut()) {
    return null;
  }

  let bestScore = -Infinity;
  let bestResult: [[number, number, number, boolean, Spin], Game] | null = null;

  const queue: SearchState[] = [];
  const expandPassed = new Array(2048).fill(0);
  const expandRes: Array<[number, number, number, Spin]> = new Array(512);

  let ptr = 0;
  let nodes = 0;

  const initialState = new SearchState(state.clone(), 0, 0, [], null);
  queue.push(initialState);

  while (ptr < queue.length) {
    const current = queue[ptr];
    const gameCopy = current.game.clone();
    const depth = current.depth;
    const linesSent = current.linesSent;
    const firstMove = current.firstMove;
    const clears = [...current.clears];
    ptr++;

    const moves = expand(gameCopy, config, expandPassed, expandRes);

    if (depth >= maxDepth - 1) {
      for (let i = 0; i < moves[0]; i++) {
        const [x, y, rot, spin] = expandRes[i];
        gameCopy.piece.x = x;
        gameCopy.piece.y = y;
        gameCopy.piece.rot = rot;
        gameCopy.spin = spin;

        const [sent, clearType] = gameCopy.hardDrop(config);
        gameCopy.regenCollisionMap();

        const newClears = clearType ? [...clears, clearType] : clears;
        const score = evaluate(gameCopy, weights, linesSent + sent, newClears);

        if (score > bestScore) {
          bestScore = score;
          const shouldHold = firstMove ? firstMove[3] : false;
          const targetSpin = firstMove ? firstMove[4] : spin;
          bestResult = [[x, y, rot, shouldHold, targetSpin], gameCopy];
        }
      }
    } else {
      for (let i = 0; i < moves[0]; i++) {
        const [x, y, rot, spin] = expandRes[i];
        gameCopy.piece.x = x;
        gameCopy.piece.y = y;
        gameCopy.piece.rot = rot;
        gameCopy.spin = spin;

        const [sent, clearType] = gameCopy.hardDrop(config);
        gameCopy.regenCollisionMap();

        if (!gameCopy.toppedOut()) {
          gameCopy.queuePtr = 0;

          const newFirstMove = firstMove || [x, y, rot, false, spin];
          const newClears = clearType ? [...clears, clearType] : clears;

          const newState = new SearchState(
            gameCopy,
            depth + 1,
            linesSent + sent,
            newClears,
            newFirstMove
          );

          queue.push(newState);

          if (gameCopy.holdPiece !== null) {
            const holdGame = gameCopy.clone();
            holdGame.doHold();
            holdGame.regenCollisionMap();

            if (!holdGame.toppedOut()) {
              const holdFirstMove: [number, number, number, boolean, Spin] = [
                x,
                y,
                rot,
                true,
                spin,
              ];
              const holdState = new SearchState(
                holdGame,
                depth + 1,
                linesSent + sent,
                newClears,
                holdFirstMove
              );

              queue.push(holdState);
            }
          }
        }
      }
    }
  }

  return bestResult;
}

export function beamSearch(
  state: Game,
  config: GameConfig,
  beamWidth: number,
  weights: Weights
): [[number, number, number, boolean, Spin], Game] | null {
  return search(state, config, beamWidth, weights);
}
