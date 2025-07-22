import { Game } from "./game/index.js";
import { Queue, Bag } from "./game/queue.js";
import {
  Mino,
  Spins,
  ComboTable,
  GameConfig,
  MoveData,
  MinoData,
  Spin,
} from "./game/data.js";
import { getKeys } from "./keyfinder.js";
import { beamSearch, expand } from "./search/index.js";
import { WEIGHTS_HANDTUNED } from "./search/eval.js";

export * from "./game/index.js";
export * from "./game/data.js";
export * from "./game/queue.js";
export * from "./keyfinder.js";
export * from "./search/index.js";
export * from "./search/eval.js";

function main(): void {
  tests.testExpansion();
}

export namespace tests {
  export function init(): [GameConfig, Queue, Game] {
    const config: GameConfig = {
      spins: Spins.MiniPlus,
      b2bChaining: false,
      b2bCharging: true,
      b2bChargeAt: 0,
      b2bChargeBase: 0,
      pcB2b: 1,
      pcSend: 5,
      comboTable: ComboTable.Multiplier,
      garbageMultiplier: 1.0,
      garbageSpecialBonus: true,
    };

    // const queue = new Queue(Bag.Bag7, Math.random() * 1000000, 16, []);
    const queue = new Queue(Bag.Bag7, 0, 16, []);
    const game = new Game(queue.shift(), queue.getFront16());

    return [config, queue, game];
  }

  export function testGame(): void {
    const [config] = init();

    const queue = new Queue(Bag.Bag7, 0, 16, []);
    const game = new Game(queue.shift(), queue.getFront16());

    game.hardDrop(config);
    game.regenCollisionMap();

    game.print();
  }

  export function testExpansion(): void {
    const [config, , game] = init();

    let avgTime = 0;
    const iters = 100000;

    const passed = new Array(2048).fill(0n);
    const res = new Array(512).fill([0, 0, 0, Spin.None]);

    for (let i = 0; i < iters + 5; i++) {
      const g = game.clone();
      const start = performance.now();
      const r = expand(g, config, passed, res);
      const duration = performance.now() - start;

      if (i === iters + 5 - 1) {
        console.log(
          `Total positions found for ${MinoData.getStr(game.piece.mino)}: ${r[0]}`
        );

        for (let j = 0; j < r[0]; j++) {
          const tester = game.clone();
          const searchGame = game.clone();
          const [x, y, rot, spin] = res[j];
          tester.piece.x = x;
          tester.piece.y = y;
          tester.piece.rot = rot;
          tester.spin = spin;
          console.log(`${x} ${y} ${rot} ${spin}`);
          tester.print();
          tester.hardDrop(config);
          const keyStart = performance.now();
          const keys = getKeys(searchGame, config, [x, y, rot, spin]);
          console.log(
            `${JSON.stringify(keys.map((key) => MoveData.getStr(key)))} in ${
              (performance.now() - keyStart) * 1000
            } μs`
          );

          console.log("------------------------");
        }
      }
      if (i > 5) {
        avgTime += duration;
      }
    }

    avgTime /= iters;
    console.log(`Average search time: ${avgTime * 1000000}μs`);
  }

  export function benchExpansion(): void {
    const [config, , game] = init();

    const iters = 1000000;
    let nodes = 0;

    const passed = new Array(2048).fill(0n);
    const res = new Array(512).fill([0, 0, 0, Spin.None]);

    const start = performance.now();

    const [x, y, rot, spin] = [game.piece.x, game.piece.y, game.piece.rot, game.spin];

    for (let i = 0; i < iters; i++) {
      game.piece.x = x;
      game.piece.y = y;
      game.piece.rot = rot;
      game.spin = spin;
      const r = expand(game, config, passed, res);
      nodes += Number(r[1]);
    }

    const duration = performance.now() - start;
    console.log(`NPS: ${nodes / (duration / 1000)}`);
  }

  export function testSearch(): void {
    const [config, , game] = init();

    console.log(
      `SEARCHING THROUGH: <${MinoData.getStr(game.piece.mino)}> ${JSON.stringify(
        game.queue
      )}`
    );

    let avgTime = 0;
    const iters = 10000;
    game.print();
    game.print();

    for (let i = 0; i < iters + 5; i++) {
      const g = game.clone();
      const start = performance.now();
      const res = beamSearch(g, config, 10, WEIGHTS_HANDTUNED);
      const duration = performance.now() - start;

      if (i === iters + 5 - 1) {
        if (res) {
          const g = res[1].clone();
          g.board.print();
          console.log("Stats:");
          console.log(`B2B: ${g.b2b}`);
          console.log(`TARGET: ${res[0][0]} ${res[0][1]} ${res[0][2]} ${res[0][3]}`);
          const g2 = game.clone();
          g2.piece.x = res[0][0];
          g2.piece.y = res[0][1];
          g2.piece.rot = res[0][2];

          g2.print();
        }
      }
      if (i > 5) {
        avgTime += duration;
      }
    }
    avgTime /= iters;
    console.log(`Average search time: ${avgTime}ms`);
  }

  export function testPlay(): void {
    const [config, queue, game] = init();

    let count = 0;
    let attack = 0;

    while (true) {
      count += 1;

      console.log(
        `SEARCHING THROUGH: <${MinoData.getStr(game.piece.mino)}> ${JSON.stringify(
          game.queue
        )}`
      );
      const start = performance.now();
      const res = beamSearch(game.clone(), config, 10, WEIGHTS_HANDTUNED);
      const elapsed = performance.now() - start;

      if (!res) {
        console.log("NO SOLUTION FOUND");
        break;
      }

      if (res[0][3]) {
        game.doHold();
        game.regenCollisionMap();
        queue.shift();
      }

      const [x, y, rot, , spin] = res[0];

      const keys = getKeys(game.clone(), config, [x, y, rot, spin]);
      for (const key of keys) {
        MoveData.run(key, game, config);
      }

      console.log(`PROJECTION (${res[1].b2b} b2b):`);
      res[1].board.print();
      console.log(
        `${MinoData.getStr(game.piece.mino)} ${res[0][0]} ${res[0][1]} ${res[0][2]}`
      );

      game.print();
      console.log(`KEYS: ${JSON.stringify(keys)}`);

      attack += game.hardDrop(config)[0];
      game.regenCollisionMap();

      queue.shift();
      game.queuePtr = 0;
      game.queue = queue.getFront16();

      console.log(`B2B: ${game.b2b}`);
      console.log(`PIECE #: ${count}`);
      console.log(`TIME: ${elapsed}ms`);
      console.log(`APP: ${(attack / count).toFixed(2)}`);

      if (game.toppedOut()) {
        break;
      }
    }

    console.log(`Game over, topped out (${game.toppedOut()} @seed ${queue.rng.seed}):`);
    game.print();
  }
}

// Run main function
main();
