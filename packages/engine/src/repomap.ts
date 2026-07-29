import { type Dirent, lstatSync, readFileSync, readdirSync } from 'node:fs';
import { basename, extname, join, relative } from 'node:path';

export type SymbolKind =
  | 'class'
  | 'function'
  | 'method'
  | 'interface'
  | 'type'
  | 'enum'
  | 'const'
  | 'struct'
  | 'trait'
  | 'var'
  | 'namespace';

export interface SymbolDef {
  file: string;
  name: string;
  kind: SymbolKind;
  line: number;
  parent?: string;
}

export interface RepoMapOptions {
  root: string;
  maxSymbols?: number;
  includeTests?: boolean;
  maxFileBytes?: number;
}

export interface RepoMapResult {
  symbols: SymbolDef[];
  ranked: SymbolDef[];
  mapText: string;
  fileCount: number;
  symbolCount: number;
  refCounts: Map<string, number>;
}

const SKIP_DIRS = new Set([
  'node_modules',
  'dist',
  '.git',
  '.workbuddy',
  'delivery',
  'build',
  'coverage',
  '.turbo',
  'target',
  '.next',
  '.cache'
]);

const TEST_RE = /(\.test|\.spec|__tests__|\/tests?\/|_test)\.(ts|tsx|js|jsx|go|py|rs)$/;
const SUPPORTED_EXT = new Set(['.ts', '.tsx', '.js', '.jsx', '.mts', '.cts', '.go', '.py', '.rs']);

const METHOD_KEYWORDS = new Set([
  'if',
  'for',
  'while',
  'switch',
  'catch',
  'function',
  'return',
  'else',
  'do',
  'with',
  'new',
  'typeof',
  'await',
  'static',
  'return'
]);

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function isTestFile(relPath: string): boolean {
  return TEST_RE.test(relPath.replace(/\\/g, '/'));
}

interface ScanResult {
  paths: string[];
  contents: Map<string, string>;
  globalCounts: Map<string, number>;
}

function scanFiles(root: string, opts: RepoMapOptions): ScanResult {
  const paths: string[] = [];
  const contents = new Map<string, string>();
  const globalCounts = new Map<string, number>();
  const maxBytes = opts.maxFileBytes ?? 512 * 1024;
  const wordRe = /[A-Za-z_$][\w$]*/g;

  const walk = (dir: string): void => {
    let entries: Dirent<string>[] = [];
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const e of entries) {
      const full = join(dir, e.name);
      if (e.isSymbolicLink()) continue;
      if (e.isDirectory()) {
        if (SKIP_DIRS.has(e.name)) continue;
        walk(full);
        continue;
      }
      if (!e.isFile()) continue;
      const ext = extname(e.name).toLowerCase();
      if (!SUPPORTED_EXT.has(ext)) continue;
      const rel = relative(root, full).replace(/\\/g, '/');
      if (!opts.includeTests && isTestFile(rel)) continue;
      let content: string;
      try {
        const st = lstatSync(full);
        if (st.size > maxBytes) continue;
        content = readFileSync(full, 'utf8');
      } catch {
        continue;
      }
      paths.push(rel);
      contents.set(rel, content);
      let m: RegExpExecArray | null;
      wordRe.lastIndex = 0;
      m = wordRe.exec(content);
      while (m !== null) {
        const w = m[0];
        globalCounts.set(w, (globalCounts.get(w) ?? 0) + 1);
        m = wordRe.exec(content);
      }
    }
  };

  walk(root);
  return { paths, contents, globalCounts };
}

function extractTsLike(content: string): SymbolDef[] {
  const out: SymbolDef[] = [];
  const lines = content.split('\n');
  let brace = 0;
  const classScopes: Array<{ name: string; depth: number }> = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    for (const ch of line) {
      if (ch === '{') brace++;
      else if (ch === '}') brace--;
    }
    const trimmed = line.trim();
    if (trimmed.startsWith('//') || trimmed.startsWith('/*') || trimmed.startsWith('*')) continue;

    let m =
      /^\s*(?:export\s+)?(?:default\s+)?(class|interface|enum|namespace)\s+([A-Za-z_$][\w$]*)/.exec(
        line
      );
    if (m) {
      const kind = m[1] as SymbolKind;
      out.push({ file: '', name: m[2], kind, line: i + 1 });
      if (kind === 'class') {
        classScopes.push({ name: m[2], depth: brace - 1 < 0 ? 0 : brace - 1 });
      }
      continue;
    }

    m = /^\s*(?:export\s+)?type\s+([A-Za-z_$][\w$]*)\s*=/.exec(line);
    if (m) {
      out.push({ file: '', name: m[1], kind: 'type', line: i + 1 });
      continue;
    }

    m = /^\s*(?:export\s+)?(const|let|var)\s+([A-Za-z_$][\w$]*)\s*=/.exec(line);
    if (m) {
      out.push({
        file: '',
        name: m[2],
        kind: m[1] === 'const' ? 'const' : 'var',
        line: i + 1
      });
      continue;
    }

    m = /^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][\w$]*)/.exec(line);
    if (m) {
      out.push({ file: '', name: m[1], kind: 'function', line: i + 1 });
      continue;
    }

    if (classScopes.length > 0) {
      const top = classScopes[classScopes.length - 1];
      const inClass = brace > top.depth;
      if (inClass) {
        m = /^\s*(?:static\s+)?(?:async\s+)?([A-Za-z_$][\w$]*)\s*\([^)]*\)\s*(?::\s*[^{]+)?\{/.exec(
          line
        );
        if (m && !METHOD_KEYWORDS.has(m[1])) {
          out.push({
            file: '',
            name: m[1],
            kind: 'method',
            line: i + 1,
            parent: top.name
          });
          continue;
        }
      }
    }

    while (classScopes.length > 0 && brace <= classScopes[classScopes.length - 1].depth) {
      classScopes.pop();
    }
  }
  return out;
}

function extractGo(content: string): SymbolDef[] {
  const out: SymbolDef[] = [];
  const lines = content.split('\n');
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();
    if (trimmed.startsWith('//') || trimmed.startsWith('/*') || trimmed.startsWith('*')) continue;

    let m = /^\s*func\s+\(([^)]*)\)\s*([A-Za-z_]\w*)\s*\(/.exec(line);
    if (m) {
      const recvTokens = m[1].replace(/[*]/g, '').trim().split(/\s+/);
      const recv = recvTokens[recvTokens.length - 1] || recvTokens[0] || '';
      out.push({ file: '', name: m[2], kind: 'method', line: i + 1, parent: recv });
      continue;
    }
    m = /^\s*func\s+([A-Za-z_]\w*)\s*\(/.exec(line);
    if (m) {
      out.push({ file: '', name: m[1], kind: 'function', line: i + 1 });
      continue;
    }
    m = /^\s*type\s+([A-Za-z_]\w*)\s+(struct|interface)/.exec(line);
    if (m) {
      out.push({
        file: '',
        name: m[1],
        kind: m[2] === 'struct' ? 'struct' : 'interface',
        line: i + 1
      });
    }
  }
  return out;
}

function extractPy(content: string): SymbolDef[] {
  const out: SymbolDef[] = [];
  const lines = content.split('\n');
  let currentClass: string | undefined;
  let currentClassIndent = -1;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (!line.trim() || line.trim().startsWith('#')) continue;
    const ind = line.length - line.trimStart().length;
    if (ind <= currentClassIndent) {
      currentClass = undefined;
      currentClassIndent = -1;
    }
    let m = /^\s*(?:async\s+)?def\s+([A-Za-z_]\w*)\s*\(/.exec(line);
    if (m) {
      out.push({
        file: '',
        name: m[1],
        kind: currentClass ? 'method' : 'function',
        line: i + 1,
        parent: currentClass
      });
      continue;
    }
    m = /^\s*class\s+([A-Za-z_]\w*)/.exec(line);
    if (m) {
      out.push({ file: '', name: m[1], kind: 'class', line: i + 1 });
      currentClass = m[1];
      currentClassIndent = ind;
    }
  }
  return out;
}

function extractRs(content: string): SymbolDef[] {
  const out: SymbolDef[] = [];
  const lines = content.split('\n');
  let brace = 0;
  const implScopes: Array<{ name: string; depth: number }> = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    for (const ch of line) {
      if (ch === '{') brace++;
      else if (ch === '}') brace--;
    }
    const trimmed = line.trim();
    if (trimmed.startsWith('//') || trimmed.startsWith('/*') || trimmed.startsWith('*')) continue;

    let m = /^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_]\w*)/.exec(line);
    if (m) {
      const parent = implScopes.length > 0 ? implScopes[implScopes.length - 1].name : undefined;
      const inImpl = parent !== undefined && brace > implScopes[implScopes.length - 1].depth;
      out.push({
        file: '',
        name: m[1],
        kind: inImpl ? 'method' : 'function',
        line: i + 1,
        parent
      });
      continue;
    }
    m = /^\s*(?:pub\s+)?(struct|enum|trait)\s+([A-Za-z_]\w*)/.exec(line);
    if (m) {
      out.push({ file: '', name: m[2], kind: m[1] as SymbolKind, line: i + 1 });
      continue;
    }
    // impl Block<T> { / impl Trait for Type {
    m = /^\s*(?:pub\s+)?impl(?:\s*<[^>]*>)?\s+(?:([A-Za-z_]\w*)\s+for\s+)?([A-Za-z_]\w*)/.exec(
      line
    );
    if (m && line.includes('{')) {
      const typeName = m[2];
      implScopes.push({ name: typeName, depth: brace - 1 < 0 ? 0 : brace - 1 });
      continue;
    }
    while (implScopes.length > 0 && brace <= implScopes[implScopes.length - 1].depth) {
      implScopes.pop();
    }
  }
  return out;
}

function extractForExt(ext: string, content: string): SymbolDef[] {
  switch (ext) {
    case '.go':
      return extractGo(content);
    case '.py':
      return extractPy(content);
    case '.rs':
      return extractRs(content);
    default:
      return extractTsLike(content); // ts/tsx/js/jsx/mts/cts
  }
}

function formatMap(result: RepoMapResult): string {
  if (result.symbolCount === 0) return '';
  const header = `repo-map（自动生成；按跨文件引用排序，共 ${result.symbolCount} 符号 / ${result.fileCount} 文件）：`;
  const byFile = new Map<string, SymbolDef[]>();
  for (const s of result.ranked) {
    if (!byFile.has(s.file)) byFile.set(s.file, []);
    byFile.get(s.file)?.push(s);
  }
  const lines: string[] = [header];
  for (const [file, syms] of byFile) {
    lines.push(file);
    for (const s of syms) {
      const tag = s.parent ? `${s.kind}:${s.parent}` : s.kind;
      const refs = result.refCounts.get(s.name) ?? 0;
      lines.push(`  [${tag}] ${s.name}:${s.line}  (refs ${refs})`);
    }
  }
  return lines.join('\n');
}

export async function buildRepoMap(opts: RepoMapOptions): Promise<RepoMapResult> {
  const empty: RepoMapResult = {
    symbols: [],
    ranked: [],
    mapText: '',
    fileCount: 0,
    symbolCount: 0,
    refCounts: new Map()
  };
  try {
    const { paths, contents, globalCounts } = scanFiles(opts.root, opts);
    const symbols: SymbolDef[] = [];
    for (const [rel, content] of contents) {
      const ext = extname(rel).toLowerCase();
      const defs = extractForExt(ext, content);
      for (const d of defs) {
        symbols.push({ ...d, file: rel });
      }
    }
    const refCounts = new Map<string, number>();
    for (const s of symbols) {
      const total = globalCounts.get(s.name) ?? 0;
      refCounts.set(s.name, Math.max(0, total - 1));
    }
    const ranked = [...symbols]
      .sort(
        (a, b) =>
          (refCounts.get(b.name) ?? 0) - (refCounts.get(a.name) ?? 0) ||
          a.file.localeCompare(b.file) ||
          a.line - b.line
      )
      .slice(0, opts.maxSymbols ?? 400);

    const result: RepoMapResult = {
      symbols,
      ranked,
      mapText: '',
      fileCount: paths.length,
      symbolCount: symbols.length,
      refCounts
    };
    result.mapText = formatMap(result);
    return result;
  } catch {
    return empty;
  }
}

export { escapeRegex };
