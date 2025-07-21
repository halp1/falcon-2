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
  passed: bigint[],
  res: Array<[number, number, number, Spin]>
): [number, bigint] {
  passed.fill(0n);

  const queue: Array<[number, number, number, Spin, Move]> = [];
  let frontPtr = 0;
  let backPtr = 1;
  let resPtr = 0;
  let nodes = 0n;

  queue[0] = [state.piece.x, state.piece.y, state.piece.rot, Spin.None, Move.None];

  while (frontPtr < backPtr && backPtr < 512) {
    const [x, y, rot, spin, lastMove] = queue[frontPtr];
    frontPtr++;
    nodes++;

    const stateHash =
      BigInt(x) |
      (BigInt(y) << 8n) |
      (BigInt(rot) << 16n) |
      (BigInt(spin === Spin.Normal ? 1 : 0) << 18n);
    const slot = Number(stateHash % BigInt(passed.length));

    if ((passed[slot] & (1n << stateHash % 64n)) !== 0n) {
      continue;
    }
    passed[slot] |= 1n << stateHash % 64n;

    state.piece.x = x;
    state.piece.y = y;
    state.piece.rot = rot;
    state.spin = spin;

    if (!state.collisionMap.test(x, y - 1, rot)) {
      continue;
    }

    res[resPtr] = [x, y, rot, spin];
    resPtr++;

    if (resPtr >= res.length) {
      break;
    }

    const availableMoves = MOVES[movesToIndex(lastMove)];
    const oldSpin = state.spin;

    for (const move of availableMoves) {
      if (move === Move.None) continue;

      const oldX = state.piece.x;
      const oldY = state.piece.y;
      const oldRot = state.piece.rot;
      state.spin = oldSpin;

      state.piece.x = x;
      state.piece.y = y;
      state.piece.rot = rot;

      if (MoveData.run(move, state, config)) {
        if (backPtr < 512) {
          queue[backPtr] = [
            state.piece.x,
            state.piece.y,
            state.piece.rot,
            state.spin,
            move,
          ];
          backPtr++;
        }
      }

      state.piece.x = oldX;
      state.piece.y = oldY;
      state.piece.rot = oldRot;
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
  const expandPassed = new Array(2048).fill(0n);
  const expandRes: Array<[number, number, number, Spin]> = new Array(512);

  let ptr = 0;
  let nodes = 0n;

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
