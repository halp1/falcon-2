export enum Mino {
  I = "I",
  J = "J",
  L = "L",
  O = "O",
  S = "S",
  T = "T",
  Z = "Z",
}

export interface TetrominoMatrix {
  w: number;
  rots: [number, number][][];
}

export enum Spin {
  None = "none",
  Mini = "mini",
  Normal = "normal",
}

export enum Spins {
  None = "none",
  T = "T",
  Mini = "mini",
  MiniPlus = "mini+",
  All = "all",
}

export enum ComboTable {
  None = "none",
  Classic = "classic-guideline",
  Modern = "modern-guideline",
  Multiplier = "multiplier",
}

export enum Move {
  None = "none",
  Left = "moveLeft",
  Right = "moveRight",
  SoftDrop = "softDrop",
  CCW = "rotateCCW",
  CW = "rotateCW",
  Flip = "rotate180",
  DasLeft = "dasLeft",
  DasRight = "dasRight",
  Hold = "hold",
  HardDrop = "hardDrop",
}

export enum KickTable {
  SRS,
  SRSPlus,
}

const TETROMINO_I: TetrominoMatrix = {
  w: 4,
  rots: [
    [
      [0, 1],
      [1, 1],
      [2, 1],
      [3, 1],
    ],
    [
      [1, 3],
      [1, 2],
      [1, 1],
      [1, 0],
    ],
    [
      [3, 2],
      [2, 2],
      [1, 2],
      [0, 2],
    ],
    [
      [2, 0],
      [2, 1],
      [2, 2],
      [2, 3],
    ],
  ],
};

const TETROMINO_L: TetrominoMatrix = {
  w: 3,
  rots: [
    [
      [0, 0],
      [0, 1],
      [1, 1],
      [2, 1],
    ],
    [
      [0, 2],
      [1, 2],
      [1, 1],
      [1, 0],
    ],
    [
      [2, 2],
      [2, 1],
      [1, 1],
      [0, 1],
    ],
    [
      [2, 0],
      [1, 0],
      [1, 1],
      [1, 2],
    ],
  ],
};

const TETROMINO_J: TetrominoMatrix = {
  w: 3,
  rots: [
    [
      [2, 0],
      [2, 1],
      [1, 1],
      [0, 1],
    ],
    [
      [2, 2],
      [1, 2],
      [1, 1],
      [1, 0],
    ],
    [
      [0, 2],
      [0, 1],
      [1, 1],
      [2, 1],
    ],
    [
      [0, 0],
      [1, 0],
      [1, 1],
      [1, 2],
    ],
  ],
};

const TETROMINO_O: TetrominoMatrix = {
  w: 2,
  rots: [
    [
      [1, 0],
      [2, 0],
      [1, 1],
      [2, 1],
    ],
    [
      [1, 0],
      [2, 0],
      [1, 1],
      [2, 1],
    ],
    [
      [1, 0],
      [2, 0],
      [1, 1],
      [2, 1],
    ],
    [
      [1, 0],
      [2, 0],
      [1, 1],
      [2, 1],
    ],
  ],
};

const TETROMINO_S: TetrominoMatrix = {
  w: 3,
  rots: [
    [
      [1, 0],
      [2, 0],
      [0, 1],
      [1, 1],
    ],
    [
      [0, 0],
      [0, 1],
      [1, 1],
      [1, 2],
    ],
    [
      [1, 1],
      [2, 1],
      [0, 2],
      [1, 2],
    ],
    [
      [1, 0],
      [1, 1],
      [2, 1],
      [2, 2],
    ],
  ],
};

const TETROMINO_T: TetrominoMatrix = {
  w: 3,
  rots: [
    [
      [1, 0],
      [0, 1],
      [1, 1],
      [2, 1],
    ],
    [
      [0, 0],
      [0, 1],
      [1, 1],
      [0, 2],
    ],
    [
      [0, 1],
      [1, 1],
      [2, 1],
      [1, 2],
    ],
    [
      [1, 0],
      [1, 1],
      [2, 1],
      [1, 2],
    ],
  ],
};

const TETROMINO_Z: TetrominoMatrix = {
  w: 3,
  rots: [
    [
      [0, 0],
      [1, 0],
      [1, 1],
      [2, 1],
    ],
    [
      [1, 0],
      [0, 1],
      [1, 1],
      [0, 2],
    ],
    [
      [0, 1],
      [1, 1],
      [1, 2],
      [2, 2],
    ],
    [
      [2, 0],
      [2, 1],
      [1, 1],
      [1, 2],
    ],
  ],
};

const INDEX_LOOKUP_TABLE: number[][] = [
  [255, 0, 8, 7],
  [1, 255, 2, 9],
  [10, 3, 255, 4],
  [6, 11, 5, 255],
];

interface KickData {
	[key: string]: [number, number][][];
}

const SRS_KICKS: KickData = {
  standard: [
    [
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
    ],
    [
      [-1, 0],
      [-1, 1],
      [0, -2],
      [-1, -2],
      [0, 0],
    ],
    [
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
    ],
    [
      [1, 0],
      [1, 1],
      [0, -2],
      [1, -2],
      [0, 0],
    ],
    [
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
    ],
    [
      [1, 0],
      [1, -1],
      [0, 2],
      [1, 2],
      [0, 0],
    ],
    [
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
    ],
    [
      [-1, 0],
      [-1, -1],
      [0, 2],
      [-1, 2],
      [0, 0],
    ],
    [
      [-1, 0],
      [-1, 1],
      [0, -2],
      [-1, -2],
      [0, 0],
    ],
    [
      [1, 0],
      [1, -1],
      [0, 2],
      [1, 2],
      [0, 0],
    ],
    [
      [1, 0],
      [1, 1],
      [0, -2],
      [1, -2],
      [0, 0],
    ],
    [
      [-1, 0],
      [-1, -1],
      [0, 2],
      [-1, 2],
      [0, 0],
    ],
  ],
  i: [
    [
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
    ],
    [
      [-2, 0],
      [1, 0],
      [-2, -1],
      [1, 2],
      [0, 0],
    ],
    [
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
    ],
    [
      [-1, 0],
      [2, 0],
      [-1, 2],
      [2, -1],
      [0, 0],
    ],
    [
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
    ],
    [
      [2, 0],
      [-1, 0],
      [2, 1],
      [-1, -2],
      [0, 0],
    ],
    [
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
      [0, 0],
    ],
    [
      [1, 0],
      [-2, 0],
      [1, -2],
      [-2, 1],
      [0, 0],
    ],
    [
      [-2, 0],
      [1, 0],
      [-2, -1],
      [1, 2],
      [0, 0],
    ],
    [
      [2, 0],
      [-1, 0],
      [2, 1],
      [-1, -2],
      [0, 0],
    ],
    [
      [-1, 0],
      [2, 0],
      [-1, 2],
      [2, -1],
      [0, 0],
    ],
    [
      [1, 0],
      [-2, 0],
      [1, -2],
      [-2, 1],
      [0, 0],
    ],
  ],
};

const SRS_PLUS_KICKS: KickData = {
  standard: SRS_KICKS.standard,
  i: SRS_KICKS.i,
};

export class MinoData {
  static getData(mino: Mino): TetrominoMatrix {
    switch (mino) {
      case Mino.I:
        return TETROMINO_I;
      case Mino.J:
        return TETROMINO_J;
      case Mino.L:
        return TETROMINO_L;
      case Mino.O:
        return TETROMINO_O;
      case Mino.S:
        return TETROMINO_S;
      case Mino.T:
        return TETROMINO_T;
      case Mino.Z:
        return TETROMINO_Z;
    }
  }

  static getRot(mino: Mino, rot: number): [number, number][] {
    console.assert(rot < 4, `Invalid rotation index: ${rot}`);
    return this.getData(mino).rots[rot];
  }

  static getStr(mino: Mino): string {
    return mino.toString();
  }

  static getBlockStr(mino: Mino): string {
    switch (mino) {
      case Mino.I:
        return "\x1b[46m  \x1b[49m";
      case Mino.J:
        return "\x1b[44m  \x1b[49m";
      case Mino.L:
        return "\x1b[43m  \x1b[49m";
      case Mino.O:
        return "\x1b[47m  \x1b[49m";
      case Mino.S:
        return "\x1b[102m  \x1b[49m";
      case Mino.T:
        return "\x1b[105m  \x1b[49m";
      case Mino.Z:
        return "\x1b[101m  \x1b[49m";
    }
  }
}

export class KickTableData {
  static getIndex(from: number, to: number): number {
    return INDEX_LOOKUP_TABLE[from][to];
  }

  static getData(
    kickTable: KickTable,
    mino: Mino,
    from: number,
    to: number
  ): [number, number][] {
    const kicks = kickTable === KickTable.SRS ? SRS_KICKS : SRS_PLUS_KICKS;
    const index = this.getIndex(from, to);
    return mino === Mino.I ? kicks.i[index] : kicks.standard[index];
  }
}

export class ComboTableData {
  static get(comboTable: ComboTable): number[] {
    switch (comboTable) {
      case ComboTable.None:
        return [];
      case ComboTable.Classic:
        return [
          0, 0, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4,
          4, 4, 4, 4,
        ];
      case ComboTable.Modern:
        return [
          0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
          5, 5, 5, 5,
        ];
      case ComboTable.Multiplier:
        return [];
    }
  }
}

export class MoveData {
  static run(move: Move, game: any, config: GameConfig): boolean {
    switch (move) {
      case Move.Left:
        return game.moveLeft();
      case Move.Right:
        return game.moveRight();
      case Move.SoftDrop:
        return game.softDrop();
      case Move.CCW:
        return game.rotate(3, config)[0];
      case Move.CW:
        return game.rotate(1, config)[0];
      case Move.Flip:
        return game.rotate(2, config)[0];
      case Move.None:
        throw new Error("None move called");
      case Move.DasLeft:
        return game.dasLeft();
      case Move.DasRight:
        return game.dasRight();
      case Move.Hold:
        return game.doHold();
      case Move.HardDrop: {
        game.softDrop();
        return true;
      }
    }
  }

  static getStr(move: Move): string {
    return move.toString();
  }

  static getTriangleKey(move: Move): string {
    switch (move) {
      case Move.None:
        return "";
      case Move.Left:
        return "L";
      case Move.Right:
        return "R";
      case Move.SoftDrop:
        return "D";
      case Move.CCW:
        return "A";
      case Move.CW:
        return "B";
      case Move.Flip:
        return "AB";
      case Move.DasLeft:
        return "LL";
      case Move.DasRight:
        return "RR";
      case Move.Hold:
        return "C";
      case Move.HardDrop:
        return "DD";
    }
  }
}

export interface GameConfig {
  spins: Spins;
  b2bCharging: boolean;
  b2bChargeAt: number;
  b2bChargeBase: number;
  b2bChaining: boolean;
  comboTable: ComboTable;
  garbageMultiplier: number;
  pcB2b: number;
  pcSend: number;
  garbageSpecialBonus: boolean;
}

export interface Garbage {
  col: number;
  amt: number;
  time: number;
}
