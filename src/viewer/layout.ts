/** Layout only consumes display IDs. It has no dependency on the language kernel. */
export function gridLayout(ids: string[]) {
  const columns = Math.min(2, Math.max(1, ids.length));
  const positions = new Map(
    ids.map((id, i) => [
      id,
      { x: 50 + (i % columns) * 390, y: 40 + Math.floor(i / columns) * 250 },
    ]),
  );
  return {
    width: columns * 240 + (columns - 1) * 150 + 100,
    height: Math.ceil(ids.length / columns) * 250 + 50,
    positions,
  };
}
