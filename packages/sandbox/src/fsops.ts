import { type Dirent, existsSync, statSync } from 'node:fs';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { join, relative, sep } from 'node:path';
import { type ToolResult, audit, formatZenoError } from '@zeno/core';
import { resolveInSandbox } from './sandbox';

const IGNORED_DIRS = new Set([
  'node_modules',
  '.git',
  '.hg',
  '.svn',
  'dist',
  'build',
  'out',
  'coverage',
  '.next',
  '.turbo',
  '.cache',
  '__pycache__',
  '.venv',
  'venv',
  'target'
]);

const SKIP_CONTENT_EXT = new Set([
  '.png',
  '.jpg',
  '.jpeg',
  '.gif',
  '.webp',
  '.ico',
  '.pdf',
  '.zip',
  '.gz',
  '.tar',
  '.woff',
  '.woff2',
  '.ttf',
  '.mp4',
  '.mp3',
  '.lock'
]);

const MAX_FILES = 2000;
const MAX_FILE_BYTES = 256 * 1024;
const MAX_MATCHES = 200;

function ext(p: string): string {
  const i = p.lastIndexOf('.');
  return i === -1 ? '' : p.slice(i).toLowerCase();
}

export async function listFiles(dir: string, cwd: string, maxDepth = 4): Promise<ToolResult> {
  let root: string;
  try {
    root = resolveInSandbox(dir || '.', cwd);
  } catch (err) {
    return { ok: false, output: '', error: formatZenoError(err) };
  }
  if (!existsSync(root)) {
    return { ok: false, output: '', error: `路径不存在: ${dir}` };
  }
  const st = statSync(root);
  if (!st.isDirectory()) {
    return { ok: true, output: dir };
  }

  const lines: string[] = [];
  let count = 0;
  let truncated = false;

  async function walk(abs: string, depth: number, prefix: string): Promise<void> {
    if (depth > maxDepth || truncated) return;
    let entries: Dirent<string>[];
    try {
      entries = await readdir(abs, { withFileTypes: true });
    } catch {
      return;
    }
    entries.sort((a, b) => {
      if (a.isDirectory() !== b.isDirectory()) return a.isDirectory() ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    for (const ent of entries) {
      if (truncated) return;
      if (ent.isDirectory() && IGNORED_DIRS.has(ent.name)) continue;
      if (count >= MAX_FILES) {
        truncated = true;
        return;
      }
      const rel = relative(root, join(abs, ent.name));
      if (ent.isDirectory()) {
        lines.push(`${prefix}📁 ${rel}/`);
        count++;
        await walk(join(abs, ent.name), depth + 1, prefix);
      } else {
        lines.push(`${prefix}  ${rel}`);
        count++;
      }
    }
  }

  await walk(root, 0, '');
  audit().record('file_access', { op: 'list', path: root, ok: true }, true);
  let output = lines.join('\n') || '(empty directory)';
  if (truncated) {
    output += `\n… (truncated at ${MAX_FILES} entries; narrow the path or depth)`;
  }
  output += `\n\n${count} entries under ${relative(cwd, root) || '.'}`;
  return { ok: true, output };
}

export interface GrepOpts {
  regex?: boolean;
  caseSensitive?: boolean;
  include?: string;
}

export async function grepSearch(
  pattern: string,
  cwd: string,
  opts: GrepOpts = {}
): Promise<ToolResult> {
  if (!pattern) return { ok: false, output: '', error: 'pattern 不能为空' };

  let matcher: (line: string) => boolean;
  if (opts.regex) {
    let re: RegExp;
    try {
      re = new RegExp(pattern, opts.caseSensitive ? '' : 'i');
    } catch (err) {
      return { ok: false, output: '', error: `非法正则: ${formatZenoError(err)}` };
    }
    matcher = (line) => re.test(line);
  } else {
    const needle = opts.caseSensitive ? pattern : pattern.toLowerCase();
    matcher = (line) => (opts.caseSensitive ? line : line.toLowerCase()).includes(needle);
  }

  const includeExt = opts.include ? opts.include.toLowerCase() : undefined;
  const matches: string[] = [];
  let filesScanned = 0;
  let done = false;

  async function walk(abs: string): Promise<void> {
    if (done) return;
    let entries: Dirent<string>[];
    try {
      entries = await readdir(abs, { withFileTypes: true });
    } catch {
      return;
    }
    for (const ent of entries) {
      if (done) return;
      const full = join(abs, ent.name);
      if (ent.isDirectory()) {
        if (IGNORED_DIRS.has(ent.name)) continue;
        await walk(full);
      } else if (ent.isFile()) {
        if (includeExt && !full.toLowerCase().endsWith(includeExt)) continue;
        if (SKIP_CONTENT_EXT.has(ext(ent.name))) continue;
        filesScanned++;
        try {
          const stat = statSync(full);
          if (stat.size > MAX_FILE_BYTES) continue;
          const content = await readFile(full, 'utf8');
          const lines = content.split('\n');
          for (let i = 0; i < lines.length; i++) {
            if (matcher(lines[i])) {
              const rel = relative(cwd, full);
              const snippet = lines[i].trim().slice(0, 200);
              matches.push(`${rel}:${i + 1}: ${snippet}`);
              if (matches.length >= MAX_MATCHES) {
                done = true;
                return;
              }
            }
          }
        } catch {}
      }
    }
  }

  const root = resolveInSandbox('.', cwd);
  await walk(root);
  audit().record('file_access', { op: 'grep', pattern, ok: true }, true);

  if (matches.length === 0) {
    return {
      ok: true,
      output: `no matches for "${pattern}" (scanned ${filesScanned} files)`
    };
  }
  let output = matches.join('\n');
  if (matches.length >= MAX_MATCHES) {
    output += `\n… (truncated at ${MAX_MATCHES} matches)`;
  }
  output += `\n\n${matches.length} match(es) across ${filesScanned} files`;
  return { ok: true, output };
}

export async function createFile(path: string, content: string, cwd: string): Promise<ToolResult> {
  try {
    const abs = resolveInSandbox(path, cwd);
    if (existsSync(abs)) {
      return {
        ok: false,
        output: '',
        error: `文件已存在: ${path}（要覆盖请用 write_file）`
      };
    }
    await mkdir(join(abs, '..'), { recursive: true });
    await writeFile(abs, content, 'utf8');
    audit().record('file_access', { op: 'create', path: abs, ok: true }, true);
    return { ok: true, output: `created ${path}` };
  } catch (err) {
    audit().record('file_access', { op: 'create', path, ok: false }, false);
    return { ok: false, output: '', error: formatZenoError(err) };
  }
}
