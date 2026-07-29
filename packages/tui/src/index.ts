// Main entry
export { startTui } from './tui';

// Stream escape hatch
export { StreamArea } from './stream-escape-hatch';

// Theme
export { palette, fg, bg, reset, themes } from './theme';
export type { Palette, ThemeName } from './theme';

// Render primitives (backward compatible)
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

// Viewport utilities
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

// Scrollback buffer
export { Scrollback } from './scrollback';

// State management (new)
export { Store } from './state/Store';
export type {
  TuiState,
  MessageEntry,
  ViewportLayout,
  LiveStatus,
  ConnectionStatus
} from './state/TuiState';
export type { DirtyFlags } from './state/Store';

// Layout (new)
export { computeLayoutStrategy, getBreakpoint, getBreakpointConfig } from './layout/Layout';
export type { LayoutStrategy, Breakpoint } from './layout/Layout';

// Components (new)
export { TopBar } from './components/TopBar';
export { ChatArea } from './components/ChatArea';
export { Message } from './components/Message';
export { ToolBlock } from './components/ToolBlock';
export { InputPanel } from './components/InputPanel';
export { StatusIndicator } from './components/StatusIndicator';
export { Scrollbar } from './components/Scrollbar';
export { ProgressBar } from './components/ProgressBar';

// Types
export type { MouseEvent, ViewportSize } from './viewport';
