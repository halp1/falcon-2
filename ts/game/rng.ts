const MODULUS = 2147483647;
const MULTIPLIER = 16807;
const MAX_FLOAT = 2147483646;

export class RNG {
  public seed: number;
  public index: number;

  constructor(seed: number) {
    this.seed = seed % MODULUS;
    this.index = 0;

    if (this.seed <= 0) {
      this.seed += MAX_FLOAT;
    }
  }

  next(): number {
    this.index += 1;
    this.seed = (MULTIPLIER * this.seed) % MODULUS;
    return this.seed;
  }

  nextFloat(): number {
    return (this.next() - 1) / MAX_FLOAT;
  }

  shuffle<T>(array: T[]): T[] {
    if (array.length === 0) {
      return array;
    }

    const result = [...array];

    for (let i = result.length - 1; i > 0; i--) {
      const r = Math.floor(this.nextFloat() * (i + 1));
      [result[i], result[r]] = [result[r], result[i]];
    }

    return result;
  }

  setSeed(value: number): void {
    this.seed = value % MODULUS;

    if (this.seed <= 0) {
      this.seed += MAX_FLOAT;
    }
  }
}
