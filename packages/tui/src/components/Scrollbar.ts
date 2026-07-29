import type { TuiState } from '../state/TuiState';

export interface ScrollbarProps {
  state: TuiState;
}

export function Scrollbar(props: ScrollbarProps): string {
  const { state } = props;
  if (state.cols <= 120) return '';

  const totalLines = state.transcript.reduce((acc, t) => {
    return acc + t.content.split('\n').length;
  }, 0);
  const viewportLines = state.layout.midEnd - state.layout.midStart + 1;
  const maxScroll = Math.max(0, totalLines - viewportLines);
  if (maxScroll <= 0) return '';

  const thumbHeight = Math.max(1, Math.floor((viewportLines / totalLines) * viewportLines));
  const thumbPos = Math.floor((state.scrollOffset / maxScroll) * (viewportLines - thumbHeight));

  const track = '│'.repeat(viewportLines);
  const thumb = '█'.repeat(thumbHeight);
  const scrollbar = track.slice(0, thumbPos) + thumb + track.slice(thumbPos + thumbHeight);

  return ` ${scrollbar}`;
}
