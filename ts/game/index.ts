import {
  Mino,
  MinoData,
  Spin,
  Spins,
  KickTable,
  KickTableData,
  GameConfig,
  Garbage,
} from "./data.js";
import { damageCalc } from "./garbage.js";

export const BOARD_WIDTH = 10;
export const BOARD_HEIGHT = 40;
export const BOARD_BUFFER = 20;

export const BOARD_UPPER_HALF = Math.floor(BOARD_HEIGHT / 2);
export const BOARD_UPPER_QUARTER = Math.floor((BOARD_HEIGHT / 4) * 3);

export const CENTER_4 = {
  start: Math.floor(BOARD_WIDTH / 2) - 2,
  end: Math.floor(BOARD_WIDTH / 2) + 2,
};
export const FULL_WIDTH = { start: 0, end: BOARD_WIDTH };

export function printBoard(
  board: number[],
  garbageHeight: number,
  highlight: [Mino, [number, number][]]
): void {
  let startRow = 0;
  for (let y = BOARD_HEIGHT - 1; y >= 0; y--) {
    let emptyRow = true;
    for (const col of board) {
      if ((col & (1 << y)) !== 0) {
        emptyRow = false;
        break;
      }
    }
    if (!emptyRow) {
      startRow = y;
      break;
    }
  }

  console.log("  +" + "--".repeat(board.length) + "+");

  for (let y = startRow; y >= 0; y--) {
    let line = `${(y + 1).toString().padStart(2)}|`;
    for (let x = 0; x < board.length; x++) {
      if ((board[x] & (1 << y)) !== 0) {
        if (highlight[1].some((v) => v[0] === x && v[1] === y)) {
          line += MinoData.getBlockStr(highlight[0]);
        } else if (y < garbageHeight) {
          line += "\x1B[48;2;68;68;68m  \x1B[49m";
        } else {
          line += "\x1B[100m  \x1B[49m";
        }
      } else {
        line += "  ";
      }
    }
    line += "|";
    console.log(line);
  }

  console.log("  +" + "--".repeat(board.length) + "+");
}

export class CollisionMap {
  public states: number[][];

  constructor(board: number[], piece: Falling) {
    this.states = Array(4)
      .fill(null)
      .map(() => Array(BOARD_WIDTH + 2).fill(0));

    for (let rot = 0; rot < 4; rot++) {
      const blocks = MinoData.getRot(piece.mino, rot);
      for (let x = 0; x < BOARD_WIDTH + 2; x++) {
        let collision = 0;
        for (const [bx, by] of blocks) {
          const px = x - bx;
          const col = px >= 0 && px < BOARD_WIDTH ? board[px] : ~0;
          collision |= ~(~col << by);
        }
        this.states[rot][x] = collision;
      }
    }
  }

  test(x: number, y: number, rot: number): boolean {
    if (x >= BOARD_WIDTH + 2 || y >= BOARD_HEIGHT) {
      return true;
    }
    // Check if the bit at position y is set
    return (this.states[rot][x] & (1 << y)) !== 0;
  }
}

export class Board {
  public cols: number[];
  public garbage: number;

  constructor() {
    this.cols = Array(BOARD_WIDTH).fill(0);
    this.garbage = 0;
  }

  set(x: number, y: number): void {
    console.assert(
      x < BOARD_WIDTH && y < BOARD_HEIGHT,
      `Invalid coordinates: ${x}, ${y}`
    );
    this.cols[x] |= 1 << y;
  }

  isOccupied(x: number, y: number): boolean {
    if (x < 0 || x >= BOARD_WIDTH || y < 0 || y >= BOARD_HEIGHT) {
      return true;
    }
    return (this.cols[x] & (1 << y)) !== 0;
  }

  clear(from: number, to: number): [number, boolean] {
    let cleared = 0;
    let garbageCleared = false;

    for (let y = to; y >= from; y--) {
      let fullRow = true;
      for (let x = 0; x < BOARD_WIDTH; x++) {
        if ((this.cols[x] & (1 << y)) === 0) {
          fullRow = false;
          break;
        }
      }

      if (fullRow) {
        cleared += 1;

        if (y < this.garbage) {
          garbageCleared = true;
          this.garbage -= 1;
        }

        for (let clearX = 0; clearX < BOARD_WIDTH; clearX++) {
          const lowMask = (1 << y) - 1;
          const low = this.cols[clearX] & lowMask;
          const high = this.cols[clearX] >> (y + 1);
          this.cols[clearX] = (high << y) | low;
        }
      }
    }

    return [cleared, garbageCleared];
  }

  isPc(): boolean {
    for (const col of this.cols) {
      if ((col & (1 << (BOARD_HEIGHT - 1))) !== 0) {
        return true;
      }
    }
    return false;
  }

  insertGarbage(amount: number, column: number): void {
    console.assert(column < BOARD_WIDTH, "hole-column out of bounds");

    if (amount === 0) {
      return;
    }

    this.garbage = Math.min(this.garbage + amount, BOARD_HEIGHT);

    const allMask = (1 << BOARD_HEIGHT) - 1;
    const bottomMask = (1 << amount) - 1;

    for (let x = 0; x < BOARD_WIDTH; x++) {
      const shifted = (this.cols[x] << amount) & allMask;
      this.cols[x] = x === column ? shifted : shifted | bottomMask;
    }
  }

  print(): void {
    printBoard([...this.cols], this.garbage, [Mino.I, []]);
  }

  collisionMap(piece: Falling): CollisionMap {
    return new CollisionMap(this.cols, piece);
  }

  maxHeight(): number {
    let maxH = 0;
    for (const col of this.cols) {
      const h = 64 - this.leadingZeros(col);
      if (h > maxH) {
        maxH = h;
      }
    }
    return maxH;
  }

  upperHalfHeight(): number {
    return Math.max(this.maxHeight() - BOARD_UPPER_HALF, 0);
  }

  upperQuarterHeight(): number {
    return Math.max(this.maxHeight() - BOARD_UPPER_QUARTER, 0);
  }

  centerHeight(): number {
    let maxH = 0;
    for (let x = CENTER_4.start; x < CENTER_4.end; x++) {
      const col = this.cols[x];
      const h = 64 - this.leadingZeros(col);
      if (h > maxH) {
        maxH = h;
      }
    }
    return maxH;
  }

  countHoles(): number {
    return this.cols.reduce((sum, col) => {
      const holeMap = ~col & ((1 << (64 - this.leadingZeros(col))) - 1);
      return sum + this.popcount(holeMap);
    }, 0);
  }

  unevenness(): number {
    let unevenness = 0;
    let last = 64 - this.leadingZeros(this.cols[0]);

    for (let i = 1; i < this.cols.length; i++) {
      const h = 64 - this.leadingZeros(this.cols[i]);
      unevenness += Math.abs(last - h);
      last = h;
    }

    return unevenness;
  }

  coveredHoles(): number {
    return this.cols.reduce((sum, col, x) => {
      const holeMap = ~col & ((1 << (64 - this.leadingZeros(col))) - 1);

      const leftCol = x === 0 ? ~0 : this.cols[x - 1];
      const rightCol = x === BOARD_WIDTH - 1 ? ~0 : this.cols[x + 1];

      return sum + this.popcount(holeMap & leftCol & rightCol);
    }, 0);
  }

  overstackedHoles(): number {
    return this.cols.reduce((sum, col) => {
      const mask = ~col & (col >> 1);

      if (mask === 0) {
        return sum;
      }

      return (
        sum + Math.max(0, 63 - this.leadingZeros(col) - this.trailingZeros(mask) - 1)
      );
    }, 0);
  }

  wells(): number {
    let wells = 0;

    for (let x = 0; x < BOARD_WIDTH; x++) {
      const leftHigher = x === 0 ? ~0 : this.cols[x - 1];
      const rightHigher = x === BOARD_WIDTH - 1 ? ~0 : this.cols[x + 1];
      const wellMask = leftHigher & rightHigher & ~this.cols[x];

      wells += this.popcount(wellMask);
    }

    return wells;
  }

  private leadingZeros(n: number): number {
    if (n === 0) return 64;
    let count = 0;
    for (let i = 63; i >= 0; i--) {
      if ((n & (1 << i)) !== 0) break;
      count++;
    }
    return count;
  }

  private trailingZeros(n: number): number {
    if (n === 0) return 64;
    let count = 0;
    for (let i = 0; i < 64; i++) {
      if ((n & (1 << i)) !== 0) break;
      count++;
    }
    return count;
  }

  private popcount(n: number): number {
    let count = 0;
    while (n !== 0) {
      count += 1;
      n &= n - 1;
    }
    return count;
  }

  clone(): Board {
    const board = new Board();
    board.cols = [...this.cols];
    board.garbage = this.garbage;
    return board;
  }
}

export class Falling {
  public x: number;
  public y: number;
  public rot: number;
  public mino: Mino;

  constructor(x: number, y: number, rot: number, mino: Mino) {
    this.x = x;
    this.y = y;
    this.rot = rot;
    this.mino = mino;
  }

  blocks(): [number, number][] {
    return MinoData.getRot(this.mino, this.rot);
  }

  clone(): Falling {
    return new Falling(this.x, this.y, this.rot, this.mino);
  }
}

export class Game {
  public board: Board;
  public queue: Mino[];
  public queuePtr: number;
  public b2b: number;
  public combo: number;
  public holdPiece: Mino | null;
  public piece: Falling;
  public garbage: Garbage[];
  public collisionMap: CollisionMap;
  public spin: Spin;

  constructor(piece: Mino, queue: Mino[]) {
    const tetromino = MinoData.getData(piece);
    this.board = new Board();
    this.piece = new Falling(
      Math.floor((BOARD_WIDTH + tetromino.w) / 2) - 1,
      BOARD_HEIGHT - BOARD_BUFFER + 2,
      0,
      piece
    );

    this.b2b = -1;
    this.combo = -1;
    this.queue = queue;
    this.queuePtr = 0;
    this.holdPiece = null;
    this.garbage = [];
    this.collisionMap = this.board.collisionMap(this.piece);
    this.spin = Spin.None;
  }

  print(): void {
    const b = this.board.clone();
    const fallingTarget: [number, number][] = [];

    for (const [x, y] of this.piece.blocks()) {
      b.set(this.piece.x - x, this.piece.y - y);
      fallingTarget.push([this.piece.x - x, this.piece.y - y]);
    }

    printBoard([...b.cols], b.garbage, [this.piece.mino, fallingTarget]);
  }

  isImmobile(): boolean {
    return (
      this.collisionMap.test(this.piece.x, this.piece.y + 1, this.piece.rot) &&
      this.collisionMap.test(this.piece.x + 1, this.piece.y, this.piece.rot) &&
      this.collisionMap.test(this.piece.x, this.piece.y - 1, this.piece.rot) &&
      this.collisionMap.test(this.piece.x - 1, this.piece.y, this.piece.rot)
    );
  }

  rotate(amount: number, config: GameConfig): [boolean, boolean] {
    const to = (this.piece.rot + amount) % 4;
    let res: [boolean, boolean, boolean] = [false, false, false];

    if (!this.collisionMap.test(this.piece.x, this.piece.y, to)) {
      this.piece.rot = to;
      res = [true, false, false];
    } else {
      const from = this.piece.rot;
      const kickset = KickTableData.getData(KickTable.SRSPlus, this.piece.mino, from, to);

      for (const [dx, dy] of kickset) {
        if (!this.collisionMap.test(this.piece.x + dx, this.piece.y - dy, to)) {
          const isTstOrFin =
            (((from === 2 && to === 3) || (from === 0 && to === 3)) &&
              dx === 1 &&
              dy === -2) ||
            (((from === 2 && to === 1) || (from === 0 && to === 1)) &&
              dx === -1 &&
              dy === -2);

          this.piece.x += dx;
          this.piece.y -= dy;
          this.piece.rot = to;
          res = [true, true, isTstOrFin];
          break;
        }
      }
    }

    if (res[0]) {
      this.updateSpin(res[2], config);
    }

    return [res[0], res[1]];
  }

  updateSpin(isTstOrFin: boolean, config: GameConfig): void {
    if (config.spins === Spins.None) {
      return;
    }

    const tStatus =
      this.piece.mino === Mino.T
        ? isTstOrFin
          ? Spin.Normal
          : this.detectTSpin()
        : Spin.None;

    if (
      tStatus !== Spin.None ||
      config.spins === Spins.T ||
      config.spins === Spins.Mini ||
      config.spins === Spins.All
    ) {
      this.spin = tStatus;
      return;
    }

    const immobile = this.isImmobile();

    if (immobile) {
      this.spin = this.piece.mino === Mino.I ? Spin.None : Spin.Mini;
    }
  }

  detectTSpin(): Spin {
    if (this.piece.mino !== Mino.T) {
      return Spin.None;
    }

    const corners = [
      [this.piece.x, this.piece.y],
      [this.piece.x + 2, this.piece.y],
      [this.piece.x, this.piece.y + 2],
      [this.piece.x + 2, this.piece.y + 2],
    ];

    let filledCorners = 0;
    for (const [cx, cy] of corners) {
      if (this.board.isOccupied(cx, cy)) {
        filledCorners += 1;
      }
    }

    if (filledCorners < 3) {
      return Spin.None;
    }

    const frontCorners =
      this.piece.rot === 0
        ? [corners[2], corners[3]]
        : this.piece.rot === 1
        ? [corners[0], corners[2]]
        : this.piece.rot === 2
        ? [corners[0], corners[1]]
        : [corners[1], corners[3]];

    const frontFilled = frontCorners.every(([cx, cy]) => this.board.isOccupied(cx, cy));

    return frontFilled ? Spin.Normal : Spin.Mini;
  }

  moveLeft(): boolean {
    if (this.collisionMap.test(this.piece.x - 1, this.piece.y, this.piece.rot)) {
      return false;
    }
    this.piece.x -= 1;
    return true;
  }

  moveRight(): boolean {
    if (this.collisionMap.test(this.piece.x + 1, this.piece.y, this.piece.rot)) {
      return false;
    }
    this.piece.x += 1;
    return true;
  }

  dasRight(): boolean {
    let x = this.piece.x;
    let moved = false;
    while (!this.collisionMap.test(x + 1, this.piece.y, this.piece.rot)) {
      moved = true;
      x += 1;
    }
    this.piece.x = x;
    return moved;
  }

  dasLeft(): boolean {
    let x = this.piece.x;
    let moved = false;
    while (!this.collisionMap.test(x - 1, this.piece.y, this.piece.rot)) {
      moved = true;
      x -= 1;
    }
    this.piece.x = x;
    return moved;
  }

  softDrop(): boolean {
    const pieceX = this.piece.x;
    let pieceY = this.piece.y;
    let moved = false;

    while (pieceY > 0 && !this.collisionMap.test(pieceX, pieceY - 1, this.piece.rot)) {
      moved = true;
      pieceY -= 1;
    }

    this.piece.y = pieceY;

    return moved;
  }

  doHold(): boolean {
    if (this.holdPiece !== null) {
      const temp = this.holdPiece;
      this.holdPiece = this.piece.mino;
      this.piece.mino = temp;
    } else {
      console.assert(this.queuePtr < this.queue.length, "Queue is empty");
      this.holdPiece = this.piece.mino;
      this.nextPiece();
    }
    return true;
  }

  nextPiece(): void {
    console.assert(this.queue.length > 0, "Queue is empty");
    const next = this.queue[this.queuePtr];
    this.queuePtr += 1;

    this.piece.mino = next;

    const tetromino = MinoData.getData(next);
    this.piece.x = Math.floor((BOARD_WIDTH + tetromino.w) / 2) - 1;
    this.piece.y = BOARD_HEIGHT - BOARD_BUFFER + 2;
    this.piece.rot = 0;
  }

  toppedOut(): boolean {
    return this.collisionMap.test(this.piece.x, this.piece.y, this.piece.rot);
  }

  regenCollisionMap(): void {
    this.collisionMap = this.board.collisionMap(this.piece);
  }

  hardDrop(config: GameConfig): [number, Spin | null] {
    console.log(
      `HARD DROP ${MinoData.getStr(this.piece.mino)} ${this.piece.x} ${this.piece.y} ${
        this.piece.rot
      }`
    );
    this.softDrop();

    const blocks = this.piece.blocks();

    let maxY = blocks[0][1];
    let minY = blocks[0][1];

    for (const [x, y] of blocks) {
      this.board.set(this.piece.x - x, this.piece.y - y);
      if (y > maxY) maxY = y;
      if (y < minY) minY = y;
    }

    const [cleared, garbageCleared] = this.board.clear(
      this.piece.y - maxY,
      this.piece.y - minY
    );
    const pc = this.board.isPc();

    let brokeB2b: number | null = this.b2b;

    if (cleared > 0) {
      if (cleared >= 4 || this.spin !== Spin.None) {
        this.b2b += 1;
        brokeB2b = null;
      } else {
        this.b2b = -1;
      }
      this.combo += 1;
    } else {
      this.combo = -1;
      brokeB2b = null;
    }

    const garbageSpecialBonus = config.garbageSpecialBonus && garbageCleared ? 1.0 : 0.0;

    let sent = Math.floor(
      damageCalc(
        cleared,
        this.spin,
        this.b2b,
        this.combo,
        config.comboTable,
        config.b2bChaining
      ) *
        config.garbageMultiplier +
        garbageSpecialBonus
    );

    if (pc) {
      sent += config.pcSend;
    }

    if (brokeB2b !== null && config.b2bCharging && brokeB2b + 1 > config.b2bChargeAt) {
      sent += Math.floor(
        (brokeB2b - config.b2bChargeAt + config.b2bChargeBase + 1) *
          config.garbageMultiplier
      );
    }

    if (cleared > 0) {
      while (sent > 0 && this.garbage.length > 0) {
        const g = this.garbage[0];
        const g16 = g.amt;

        if (g16 > sent) {
          g.amt -= sent;
          sent = 0;
          break;
        } else {
          sent -= g16;
          this.garbage.shift();
        }
      }
    } else {
      while (this.garbage.length > 0 && this.garbage[0].time === 0) {
        const g = this.garbage.shift()!;
        this.board.insertGarbage(g.amt, g.col);
      }
    }

    for (const g of this.garbage) {
      if (g.time > 0) {
        g.time -= 1;
      }
    }

    const clearType = cleared >= 4 ? Spin.Normal : cleared > 0 ? this.spin : null;

    this.spin = Spin.None;
    this.nextPiece();
    return [sent, clearType];
  }

  clone(): Game {
    const game = new Game(this.piece.mino, [...this.queue]);
    game.board = this.board.clone();
    game.queuePtr = this.queuePtr;
    game.b2b = this.b2b;
    game.combo = this.combo;
    game.holdPiece = this.holdPiece;
    game.piece = this.piece.clone();
    game.garbage = this.garbage.map((g) => ({ ...g }));
    game.collisionMap = this.board.collisionMap(game.piece);
    game.spin = this.spin;
    return game;
  }
}
