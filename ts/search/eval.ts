import { Game, BOARD_BUFFER, BOARD_HEIGHT } from "../game/index.js";
import { Spin } from "../game/data.js";

export interface Weights {
  height: number;
  upperHalfHeight: number;
  upperQuarterHeight: number;
  centerHeight: number;
  extraWells: number;
  clearNone: number;
  clearMini: number;
  clearNormal: number;
  sent: number;
  b2b: number;
  combo: number;
  holes: number;
  coveredHoles: number;
  overstackedHoles: number;
  unevenness: number;
}

export class WeightsData {
  static mutate(weights: Weights, threshold: number, changeFactor: number): Weights {
    const getMultiplier = () => {
      const v = Math.random();
      if (v < threshold) {
        return 0;
      } else if (v < threshold + (1.0 - threshold) / 2.0) {
        return 1;
      } else {
        return -1;
      }
    };

    return {
      height: weights.height + changeFactor * getMultiplier(),
      upperHalfHeight: weights.upperHalfHeight + changeFactor * getMultiplier(),
      upperQuarterHeight: weights.upperQuarterHeight + changeFactor * getMultiplier(),
      centerHeight: weights.centerHeight + changeFactor * getMultiplier(),
      extraWells: weights.extraWells + changeFactor * getMultiplier(),
      clearNone: weights.clearNone + changeFactor * getMultiplier(),
      clearMini: weights.clearMini + changeFactor * getMultiplier(),
      clearNormal: weights.clearNormal + changeFactor * getMultiplier(),
      sent: weights.sent + changeFactor * getMultiplier(),
      b2b: weights.b2b + changeFactor * getMultiplier(),
      combo: weights.combo + changeFactor * getMultiplier(),
      holes: weights.holes + changeFactor * getMultiplier(),
      coveredHoles: weights.coveredHoles + changeFactor * getMultiplier(),
      overstackedHoles: weights.overstackedHoles + changeFactor * getMultiplier(),
      unevenness: weights.unevenness + changeFactor * getMultiplier(),
    };
  }
}

export const WEIGHTS_HANDTUNED: Weights = {
  height: -51,
  upperHalfHeight: -25,
  upperQuarterHeight: -34,
  centerHeight: -100,
  extraWells: -100,
  clearNone: 0,
  clearMini: 100,
  clearNormal: 0,
  sent: 324,
  b2b: 164,
  combo: 191,
  holes: -472,
  coveredHoles: -361,
  overstackedHoles: -81,
  unevenness: -184,
};

export function evaluate(
  game: Game,
  weights: Weights,
  linesSent: number,
  clearTypes: Spin[]
): number {
  let score = 0;

  score += game.board.maxHeight() * weights.height;
  score += game.board.upperHalfHeight() * weights.upperHalfHeight;
  score += game.board.upperQuarterHeight() * weights.upperQuarterHeight;
  score += game.board.centerHeight() * weights.centerHeight;

  score += Math.max(0, game.board.wells() - 1) * weights.extraWells;

  const clearCounts = clearTypes.reduce(
    (counts, spin) => {
      switch (spin) {
        case Spin.None:
          counts.none++;
          break;
        case Spin.Mini:
          counts.mini++;
          break;
        case Spin.Normal:
          counts.normal++;
          break;
      }
      return counts;
    },
    { none: 0, mini: 0, normal: 0 }
  );

  score += clearCounts.none * weights.clearNone;
  score += clearCounts.mini * weights.clearMini;
  score += clearCounts.normal * weights.clearNormal;

  score += linesSent * weights.sent;

  score += game.b2b * weights.b2b;
  score += game.combo * weights.combo;

  score += game.board.countHoles() * weights.holes;
  score += game.board.coveredHoles() * weights.coveredHoles;
  score += game.board.overstackedHoles() * weights.overstackedHoles;

  score += game.board.unevenness() * weights.unevenness;

  if (game.toppedOut()) {
    score -= 1000000;
  }

  if (BOARD_HEIGHT - BOARD_BUFFER < game.board.maxHeight()) {
    score -= Math.pow(game.board.maxHeight() - (BOARD_HEIGHT - BOARD_BUFFER), 3) * 10;
  }

  return score;
}
