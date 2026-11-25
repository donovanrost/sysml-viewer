const paths: Record<string, string> = {
  cube: 'M12 3 3 8v9l9 5 9-5V8L12 3Zm0 10L3 8m9 5 9-5m-9 5v9m-4-17 9 5',
  folder: 'M3 7V5h6l2 2h10v13H3V7Z',
  search: 'M21 21l-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Z',
  open: 'M3 10V5h6l2 2h10v3M3 10h19l-3 10H3V10Z',
  arrow: 'M5 12h14m-6-6 6 6-6 6',
  up: 'm8 5-5 5 5 5M3 10h11a6 6 0 0 1 0 12',
  chevron: 'm9 5 7 7-7 7',
  link: 'm10 13 4-4m-6 7-2 2a4 4 0 0 1-6-6l4-4a4 4 0 0 1 6 0m4 0 2-2a4 4 0 1 0-6-6l-4 4',
  port: 'M4 12h16M4 5v14m16-14v14M9 8h6v8H9V8Z',
  attribute: 'M9 3 7 21M17 3l-2 18M3 8h18M2 16h18',
  requirement: 'm3 6 2 2 4-4M12 6h9M3 14l2 2 4-4m3 2h9M12 21h9',
  source: 'm8 5-6 7 6 7M16 5l6 7-6 7m-5 2 2-18',
  grid: 'M3 3h7v7H3V3Zm11 0h7v7h-7V3ZM3 14h7v7H3v-7Zm11 0h7v7h-7v-7Z',
  lock: 'M6 10V7a6 6 0 0 1 12 0v3M4 10h16v12H4V10Zm8 5v3',
  info: 'M12 11v6m0-10v1M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0Z',
  warning: 'm12 2 11 20H1L12 2Zm0 7v5m0 3v1',
  reload: 'M20 8a9 9 0 1 0 1 8M20 2v6h-6',
  panel: 'M3 3h18v18H3V3Zm12 0v18',
  fit: 'M3 9V3h6m6 0h6v6m0 6v6h-6m-6 0H3v-6',
  sun: 'M12 1v3m0 16v3M1 12h3m16 0h3M4 4l2 2m12 12 2 2M4 20l2-2M18 6l2-2M17 12a5 5 0 1 1-10 0 5 5 0 0 1 10 0Z',
  close: 'm5 5 14 14M5 19 19 5',
  back: 'M19 12H5m6-6-6 6 6 6',
  target: 'M12 2v4m0 12v4M2 12h4m12 0h4M19 12a7 7 0 1 1-14 0 7 7 0 0 1 14 0Z',
  down: 'm6 9 6 6 6-6',
};
export function icon(name: string, cls = '') {
  return `<svg class="icon ${cls}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${paths[name] ?? paths.cube}"/></svg>`;
}
