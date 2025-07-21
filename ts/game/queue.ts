import { Mino } from "./data.js";
import { RNG } from "./rng.js";

export enum Bag {
  Bag7 = "7-bag",
}

export class BagData {
  static getCycle(bag: Bag): Mino[] {
    switch (bag) {
      case Bag.Bag7:
        return [Mino.Z, Mino.L, Mino.O, Mino.S, Mino.I, Mino.J, Mino.T];
    }
  }
}

export class Queue {
  public bag: Bag;
  public rng: RNG;
  public minSize: number;
  public queue: Mino[];

  constructor(bag: Bag, seed: number, minSize: number, initial: Mino[]) {
    console.assert(minSize >= 16, "Bag min size must be at least 16");
    this.bag = bag;
    this.rng = new RNG(seed);
    this.minSize = minSize;
    this.queue = [...initial];

    while (this.queue.length < minSize) {
      const cycle = this.rng.shuffle(BagData.getCycle(bag));
      this.queue.push(...cycle);
    }
  }

  shift(): Mino {
    const res = this.queue.shift() || Mino.I;

    while (this.queue.length < this.minSize) {
      const cycle = this.rng.shuffle(BagData.getCycle(this.bag));
      this.queue.push(...cycle);
    }

    return res;
  }

  getFront16(): Mino[] {
    const res: Mino[] = new Array(16);

    for (let i = 0; i < 16; i++) {
      res[i] = this.queue[i] || Mino.I;
    }

    return res;
  }

  clone(): Queue {
    const cloned = new Queue(this.bag, 0, this.minSize, []);
    cloned.rng = new RNG(this.rng.seed);
    cloned.rng.index = this.rng.index;
    cloned.queue = [...this.queue];
    return cloned;
  }
}
