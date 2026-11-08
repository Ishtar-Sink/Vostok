export const artPalettes: ReadonlyArray<readonly [string, string]> = [
  ['#7C3AED', '#06B6D4'],
  ['#A855F7', '#22D3EE'],
  ['#6366F1', '#2DD4BF'],
  ['#8B5CF6', '#38BDF8'],
  ['#4C1D95', '#0E7490'],
  ['#9333EA', '#67E8F9'],
  ['#5B21B6', '#14B8A6'],
  ['#C084FC', '#0891B2'],
  ['#7E22CE', '#06B6D4'],
  ['#6D28D9', '#5EEAD4'],
];

/** Mesmo id, mesmo par de cores, em qualquer cliente e sem estado guardado. */
export function paletteFor(seed: string): readonly [string, string] {
  let h = 0;
  for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) >>> 0;
  return artPalettes[h % artPalettes.length] ?? artPalettes[0] ?? ['#7C3AED', '#06B6D4'];
}
