import { type Palette, fg, palette, reset } from '../theme';

const KEYWORDS_SET: Record<string, Set<string>> = {
  python: new Set([
    'def',
    'class',
    'import',
    'from',
    'return',
    'if',
    'elif',
    'else',
    'for',
    'while',
    'try',
    'except',
    'finally',
    'with',
    'as',
    'lambda',
    'async',
    'await',
    'raise',
    'yield',
    'pass',
    'break',
    'continue',
    'and',
    'or',
    'not',
    'is',
    'in',
    'True',
    'False',
    'None'
  ]),
  javascript: new Set([
    'const',
    'let',
    'var',
    'function',
    'return',
    'if',
    'else',
    'for',
    'while',
    'switch',
    'case',
    'break',
    'continue',
    'import',
    'export',
    'default',
    'from',
    'async',
    'await',
    'try',
    'catch',
    'finally',
    'throw',
    'new',
    'this',
    'class',
    'extends',
    'super',
    'typeof',
    'instanceof',
    'void',
    'delete',
    'null',
    'undefined',
    'true',
    'false'
  ]),
  typescript: new Set([
    'const',
    'let',
    'var',
    'function',
    'return',
    'if',
    'else',
    'for',
    'while',
    'switch',
    'case',
    'break',
    'continue',
    'import',
    'export',
    'default',
    'from',
    'async',
    'await',
    'try',
    'catch',
    'finally',
    'throw',
    'new',
    'this',
    'class',
    'extends',
    'super',
    'interface',
    'type',
    'enum',
    'namespace',
    'public',
    'private',
    'protected',
    'readonly',
    'implements',
    'declare',
    'abstract',
    'as',
    'is',
    'keyof',
    'never',
    'any',
    'unknown',
    'void',
    'null',
    'undefined',
    'true',
    'false'
  ]),
  go: new Set([
    'package',
    'import',
    'func',
    'return',
    'var',
    'const',
    'type',
    'struct',
    'interface',
    'if',
    'else',
    'for',
    'range',
    'switch',
    'case',
    'default',
    'select',
    'go',
    'defer',
    'chan',
    'map',
    'make',
    'new',
    'nil',
    'true',
    'false',
    'break',
    'continue'
  ]),
  rust: new Set([
    'fn',
    'let',
    'mut',
    'const',
    'static',
    'pub',
    'struct',
    'enum',
    'trait',
    'impl',
    'use',
    'mod',
    'crate',
    'self',
    'Super',
    'where',
    'for',
    'loop',
    'while',
    'if',
    'else',
    'match',
    'return',
    'break',
    'continue',
    'async',
    'await',
    'unsafe',
    'dyn',
    'ref',
    'move',
    'true',
    'false',
    'Some',
    'None',
    'Ok',
    'Err'
  ]),
  c: new Set([
    'int',
    'char',
    'float',
    'double',
    'void',
    'long',
    'short',
    'signed',
    'unsigned',
    'struct',
    'union',
    'enum',
    'typedef',
    'sizeof',
    'if',
    'else',
    'for',
    'while',
    'do',
    'switch',
    'case',
    'default',
    'return',
    'break',
    'continue',
    'goto',
    'static',
    'extern',
    'const',
    'volatile',
    'auto',
    'register',
    'NULL'
  ]),
  cpp: new Set([
    'int',
    'char',
    'float',
    'double',
    'void',
    'bool',
    'class',
    'struct',
    'union',
    'enum',
    'typedef',
    'template',
    'typename',
    'namespace',
    'using',
    'public',
    'private',
    'protected',
    'virtual',
    'override',
    'final',
    'new',
    'delete',
    'this',
    'auto',
    'const',
    'constexpr',
    'if',
    'else',
    'for',
    'while',
    'do',
    'switch',
    'case',
    'default',
    'return',
    'break',
    'continue',
    'try',
    'catch',
    'throw',
    'true',
    'false',
    'nullptr',
    'std'
  ]),
  bash: new Set([
    'if',
    'then',
    'else',
    'elif',
    'fi',
    'for',
    'while',
    'do',
    'done',
    'case',
    'esac',
    'function',
    'return',
    'exit',
    'export',
    'local',
    'unset',
    'echo',
    'cd',
    'pwd',
    'mkdir',
    'rm',
    'cp',
    'mv'
  ])
};

export function detectLanguage(code: string, hint?: string): string {
  if (hint) {
    const h = hint.toLowerCase();
    if (h.includes('ts') || h.includes('typescript')) return 'typescript';
    if (h.includes('js') || h.includes('javascript')) return 'javascript';
    if (h.includes('py') || h.includes('python')) return 'python';
    if (h.includes('json')) return 'json';
    if (h.includes('bash') || h.includes('sh') || h.includes('shell')) return 'bash';
    if (h.includes('md') || h.includes('markdown')) return 'markdown';
  }

  const trimmed = code.trim();
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) return 'json';
  if (trimmed.startsWith('#!/bin/bash') || trimmed.startsWith('#!/bin/sh')) return 'bash';
  if (
    trimmed.startsWith('import ') ||
    trimmed.startsWith('from ') ||
    trimmed.startsWith('def ') ||
    trimmed.startsWith('class ')
  ) {
    if (trimmed.includes(':') && !trimmed.includes('{')) return 'python';
  }
  if (
    trimmed.includes('=>') ||
    trimmed.includes('function') ||
    trimmed.includes('const ') ||
    trimmed.includes('let ')
  ) {
    return 'typescript';
  }

  return 'text';
}

export function highlightCode(code: string, language?: string, p?: Palette): string {
  const activePalette = p ?? palette('mocha');
  const lang = (language || 'typescript')
    .toLowerCase()
    .replace(/^js$/, 'javascript')
    .replace(/^ts$/, 'typescript')
    .replace(/^py$/, 'python')
    .replace(/^rs$/, 'rust')
    .replace(/^sh$/, 'bash');

  if (lang === 'text') return code;

  if (lang === 'json') {
    return highlightJson(code, activePalette);
  }
  if (lang === 'markdown' || lang === 'md') {
    return highlightMarkdown(code, activePalette);
  }

  const keywords = KEYWORDS_SET[lang] || KEYWORDS_SET.typescript;
  const lines = code.split('\n');

  const highlightedLines = lines.map((line) => {
    let commentIdx = -1;
    if (lang === 'python' || lang === 'bash') {
      commentIdx = line.indexOf('#');
    } else {
      commentIdx = line.indexOf('//');
    }

    let codePart = line;
    let commentPart = '';
    if (commentIdx !== -1) {
      codePart = line.slice(0, commentIdx);
      commentPart = line.slice(commentIdx);
    }

    let result = '';
    let i = 0;
    while (i < codePart.length) {
      const ch = codePart[i];

      if (ch === '"' || ch === "'" || ch === '`') {
        const quote = ch;
        let strVal = quote;
        i++;
        while (i < codePart.length) {
          strVal += codePart[i];
          if (codePart[i] === quote && codePart[i - 1] !== '\\') {
            i++;
            break;
          }
          i++;
        }
        result += `${fg(activePalette.green)}${strVal}${reset}`;
        continue;
      }

      if (/\d/.test(ch) && (i === 0 || !/[a-zA-Z0-9_]/.test(codePart[i - 1]))) {
        let numStr = '';
        while (i < codePart.length && /[0-9.xX-a-fA-F]/.test(codePart[i])) {
          numStr += codePart[i];
          i++;
        }
        result += `${fg(activePalette.teal || activePalette.blue)}${numStr}${reset}`;
        continue;
      }

      if (/[a-zA-Z_$]/.test(ch)) {
        let word = '';
        while (i < codePart.length && /[a-zA-Z0-9_$]/.test(codePart[i])) {
          word += codePart[i];
          i++;
        }
        if (keywords.has(word)) {
          result += `${fg(activePalette.mauve)}${word}${reset}`;
        } else if (i < codePart.length && codePart[i] === '(') {
          result += `${fg(activePalette.blue)}${word}${reset}`;
        } else if (/^[A-Z]/.test(word)) {
          result += `${fg(activePalette.yellow)}${word}${reset}`;
        } else {
          result += word;
        }
        continue;
      }

      result += ch;
      i++;
    }

    if (commentPart) {
      result += `${fg(activePalette.subtext)}${commentPart}${reset}`;
    }

    return result;
  });

  return highlightedLines.join('\n');
}

function highlightJson(code: string, p: Palette): string {
  return code.replace(
    /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g,
    (match) => {
      let cls = p.blue;
      if (/^"/.test(match)) {
        if (/:$/.test(match)) {
          cls = p.mauve;
        } else {
          cls = p.green;
        }
      } else if (/true|false/.test(match)) {
        cls = p.teal;
      } else if (/null/.test(match)) {
        cls = p.red;
      }
      return `${fg(cls)}${match}${reset}`;
    }
  );
}

function highlightMarkdown(code: string, p: Palette): string {
  const lines = code.split('\n');
  return lines
    .map((line) => {
      if (line.startsWith('#')) {
        return `${fg(p.mauve)}${line}${reset}`;
      }
      if (line.startsWith('```')) {
        return `${fg(p.subtext)}${line}${reset}`;
      }
      if (line.startsWith('- ') || line.startsWith('* ') || /^\d+\./.test(line)) {
        return `${fg(p.yellow)}${line}${reset}`;
      }
      return line.replace(/`([^`]+)`/g, `${fg(p.teal)}\`$1\`${reset}`);
    })
    .join('\n');
}
