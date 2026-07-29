import { highlightCode } from '../render/syntax';
import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';

export interface CodeEditorProps {
  filename: string;
  code: string;
  language: string;
  state: TuiState;
}

export function CodeEditor(props: CodeEditorProps): string {
  const { filename, code, language, state } = props;
  const c = state.palette;
  const highlighted = highlightCode(code, language, c);
  const lines = highlighted.split('\n');

  const formattedLines = lines.map((l, i) => {
    const num = String(i + 1).padStart(4, ' ');
    return `${fg(c.subtext)}${num} │ ${reset}${l}`;
  });

  const header = `${fg(c.mauve)}📄 ${filename} (${language})${reset}`;
  return `${header}\n${formattedLines.join('\n')}`;
}
