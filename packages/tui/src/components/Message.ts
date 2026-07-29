import { detectLanguage, highlightCode } from '../render/syntax';
import type { TuiState } from '../state/TuiState';
import { renderMessage } from '../utils/text';

export interface MessageProps {
  state: TuiState;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  collapsed?: boolean;
  selected?: boolean;
}

function highlightCodeBlocks(content: string, state: TuiState): string {
  const CODE_BLOCK_RE = /```(\w+)?\n?([\s\S]*?)```/g;
  return content.replace(CODE_BLOCK_RE, (_, lang, code) => {
    const language = detectLanguage(code, lang);
    const highlighted = highlightCode(code.trim(), language, state.palette);
    return highlighted;
  });
}

export function Message(props: MessageProps): string {
  const { state, role, content } = props;
  const innerW = Math.max(10, state.cols - 4);

  // Apply syntax highlighting to code blocks
  const highlightedContent = highlightCodeBlocks(content, state);

  const block = renderMessage({
    role,
    content: highlightedContent,
    palette: state.palette,
    width: innerW,
    background: role !== 'system'
  });
  return block;
}
