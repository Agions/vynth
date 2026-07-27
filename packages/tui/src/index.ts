export { startTui } from './tui';
export { StreamArea } from './stream-escape-hatch';
export { palette, fg, bg, reset } from './theme';
export {
  renderBadge,
  renderFrameStart,
  renderInline,
  renderInputArea,
  renderMessage,
  renderPanel,
  renderSection,
  renderStatusBar,
  renderToolBlock,
  visibleWidth,
  wrapLine,
  clipHistory
} from './render';
export {
  computeLayout,
  enterAltScreen,
  leaveAltScreen,
  enableMouseTracking,
  disableMouseTracking,
  resetScrollRegion,
  parseMouse,
  setScrollRegion,
  cursorTo,
  eraseLine,
  eraseDown
} from './viewport';
export { Scrollback } from './scrollback';
export type { Palette } from './theme';
export type { MouseEvent, ViewportLayout, ViewportSize } from './viewport';
