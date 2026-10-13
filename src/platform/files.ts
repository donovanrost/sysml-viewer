import type { SourceDocument } from '../kernel/generated/SourceDocument';

export const desktop = '__TAURI_INTERNALS__' in window;
export interface FileSelection {
  documents: SourceDocument[];
  paths: string[];
}

function checkFiles(names: string[], sizes: number[]) {
  if (names.length > 32 || names.some((n) => !/\.(sysml|kerml)$/i.test(n)))
    throw new Error('Select up to 32 .sysml or .kerml text files.');
  if (sizes.some((s) => s > 5_000_000) || sizes.reduce((a, b) => a + b, 0) > 20_000_000)
    throw new Error('Files must be at most 5 MB each and 20 MB combined.');
}

export async function readBrowserFiles(files: File[]): Promise<FileSelection> {
  checkFiles(
    files.map((f) => f.name),
    files.map((f) => f.size),
  );
  const documents = await Promise.all(
    files.map(async (file, i) => ({
      id: `${i}:${file.name}`,
      name: file.name,
      text: new TextDecoder('utf-8', { fatal: true }).decode(await file.arrayBuffer()),
    })),
  );
  return { documents, paths: [] };
}

export async function openNativeFiles(
  previousPaths: string[],
  reload: boolean,
): Promise<FileSelection | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const { readTextFile, stat } = await import('@tauri-apps/plugin-fs');
  const result =
    reload && previousPaths.length
      ? previousPaths
      : await open({
          multiple: true,
          directory: false,
          filters: [{ name: 'SysML / KerML model', extensions: ['sysml', 'kerml'] }],
        });
  if (!result) return null;
  const paths = Array.isArray(result) ? result : [result];
  checkFiles(paths, await Promise.all(paths.map(async (p) => (await stat(p)).size)));
  const documents = await Promise.all(
    paths.map(async (p) => ({
      id: p,
      name: p.split(/[/\\]/).at(-1)!,
      text: await readTextFile(p),
    })),
  );
  checkFiles(
    paths,
    documents.map((d) => new TextEncoder().encode(d.text).length),
  );
  return { documents, paths };
}
