import { Game } from "./game/index.js";
import { GameConfig, Move, Spin, MoveData } from "./game/data.js";

const MOVES: Move[][] = [
  [
    Move.CW,
    Move.CCW,
    Move.Flip,
    Move.Left,
    Move.Right,
    Move.SoftDrop,
    Move.DasLeft,
    Move.DasRight,
    Move.HardDrop,
  ],
  [
    Move.CW,
    Move.CCW,
    Move.Flip,
    Move.Left,
    Move.SoftDrop,
    Move.DasRight,
    Move.HardDrop,
    Move.None,
    Move.None,
  ],
  [
    Move.CW,
    Move.CCW,
    Move.Flip,
    Move.Right,
    Move.SoftDrop,
    Move.DasLeft,
    Move.HardDrop,
    Move.None,
    Move.None,
  ],
  [
    Move.CW,
    Move.CCW,
    Move.Flip,
    Move.Left,
    Move.Right,
    Move.DasLeft,
    Move.DasRight,
    Move.HardDrop,
    Move.None,
  ],
  [
    Move.CCW,
    Move.Flip,
    Move.Left,
    Move.Right,
    Move.SoftDrop,
    Move.DasLeft,
    Move.DasRight,
    Move.HardDrop,
    Move.None,
  ],
  [
    Move.CW,
    Move.Flip,
    Move.Left,
    Move.Right,
    Move.SoftDrop,
    Move.DasLeft,
    Move.DasRight,
    Move.HardDrop,
    Move.None,
  ],
  [
    Move.CW,
    Move.CCW,
    Move.Left,
    Move.Right,
    Move.SoftDrop,
    Move.DasLeft,
    Move.DasRight,
    Move.HardDrop,
    Move.None,
  ],
  [
    Move.CW,
    Move.CCW,
    Move.Flip,
    Move.Right,
    Move.SoftDrop,
    Move.DasRight,
    Move.HardDrop,
    Move.None,
    Move.None,
  ],
  [
    Move.CW,
    Move.CCW,
    Move.Flip,
    Move.Left,
    Move.SoftDrop,
    Move.DasLeft,
    Move.HardDrop,
    Move.None,
    Move.None,
  ],
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
    case Move.DasLeft:
      return 7;
    case Move.DasRight:
      return 8;
    default:
      return 0;
  }
}

export function getKeys(
  startGame: Game,
  config: GameConfig,
  target: [number, number, number, Spin]
): Move[] {
  const [targetX, targetY, targetRot, targetSpin] = target;

  const queue: Array<{
    game: Game;
    moves: Move[];
  }> = [];

  const visited = new Set<string>();
  queue.push({ game: startGame.clone(), moves: [] });

  while (queue.length > 0) {
    const { game, moves } = queue.shift()!;

    if (
      game.piece.x === targetX &&
      game.piece.y === targetY &&
      game.piece.rot === targetRot &&
      game.spin === targetSpin
    ) {
      return moves;
    }

    const state = `${game.piece.x},${game.piece.y},${game.piece.rot},${game.spin}`;
    if (visited.has(state) || moves.length > 20) {
      continue;
    }
    visited.add(state);

    const lastMove = moves.length > 0 ? moves[moves.length - 1] : Move.None;
    const availableMoves = MOVES[movesToIndex(lastMove)];

    for (const move of availableMoves) {
      if (move === Move.None) continue;

      const testGame = game.clone();
      const success = MoveData.run(move, testGame, config);

      if (success) {
        const newMoves = [...moves, move];
        queue.push({ game: testGame, moves: newMoves });
      }
    }
  }

  return [];
}
