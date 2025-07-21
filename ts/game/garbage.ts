import { ComboTable, ComboTableData, Spin } from "./data.js";

export function damageCalc(
  lines: number,
  spin: Spin,
  b2b: number,
  combo: number,
  comboTable: ComboTable,
  b2bChaining: boolean
): number {
  console.assert(lines <= 4, "Lines must be between 0 and 4");

  let damage: number = 0;
  switch (lines) {
    case 0:
      damage = 0;
      break;
    case 1:
      damage = spin === Spin.Normal ? 2 : 0;
      break;
    case 2:
      damage = spin === Spin.Normal ? 4 : 1;
      break;
    case 3:
      damage = spin === Spin.Normal ? 6 : 2;
      break;
    case 4:
      damage = spin !== Spin.None ? 10 : 4;
      break;
    default:
      throw new Error(`Invalid number of lines: ${lines}`);
  }

  damage +=
    lines > 0 && b2b > 0
      ? b2bChaining
        ? Math.floor(1.0 + Math.log1p(b2b * 0.8)) +
          (b2b === 1 ? 0.0 : (1.0 + (Math.log1p(b2b * 0.8) % 1)) / 3.0)
        : 1.0
      : 0.0;

  damage =
    combo > 0
      ? comboTable === ComboTable.Multiplier
        ? (() => {
            const g1 = damage * (1.0 + 0.25 * combo);
            return combo > 1 ? Math.max(Math.log1p(combo * 1.25), g1) : g1;
          })()
        : (() => {
            const t = ComboTableData.get(comboTable);
            return damage + (t[Math.max(0, Math.min(combo - 1, t.length - 1))] || 0);
          })()
      : damage;

  return damage;
}
