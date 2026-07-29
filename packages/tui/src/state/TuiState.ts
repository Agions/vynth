import type { Mode } from '@zeno/core';
import type { ThemeName } from '../theme';

export interface MessageEntry {
  id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  timestamp: number;
  toolId?: string;
  collapsed?: boolean;
  status?: 'running' | 'ok' | 'error';
  output?: string;
}

export interface ViewportLayout {
  topStart: number;
  topEnd: number;
  midStart: number;
  midEnd: number;
  botStart: number;
  botEnd: number;
}

export type LiveStatus = 'idle' | 'streaming' | 'tool' | 'thinking';
export type ConnectionStatus = 'connected' | 'disconnected' | 'error';

export interface TokenUsage {
  inputTokens: number;
  outputTokens: number;
  estimatedCost: number;
  history: Array<{ ts: number; input: number; output: number; cost: number }>;
}

export interface TuiState {
  cols: number;
  rows: number;
  layout: ViewportLayout;

  transcript: MessageEntry[];
  scrollOffset: number;
  selectedToolId: string | null;
  collapsedTools: Set<string>;

  briefMode: boolean;

  input: string;
  prompting: boolean;

  inputPinned: boolean;

  mode: Mode;

  liveStatus: LiveStatus;
  currentTool: string | null;
  liveToolId: string | null;
  turnCount: number;
  connectionStatus: ConnectionStatus;

  theme: ThemeName;
  palette: ReturnType<typeof import('../theme').palette>;

  searchQuery: string;
  searchResults: number[];
  currentSearchIndex: number;

  commandPaletteOpen: boolean;
  commandPaletteFilter: string;
  commandPaletteIndex: number;

  hintsBarCollapsed: boolean;
  inputHistory: string[];
  inputHistoryIndex: number;

  tokenUsage: TokenUsage;
  usagePanelOpen: boolean;

  slashCycleIndex: number;

  slashPaletteOpen: boolean;
  slashPaletteFilter: string;
  slashPaletteIndex: number;

  tasksPanelOpen: boolean;

  fileTreeOpen: boolean;
  fileTreeData: string[];
  fileTreeIndex: number;

  configModalOpen: boolean;
  configDraft: {
    model: string;
    llmBaseUrl: string;
    apiKey: string;
    theme: ThemeName;
    networkAllowed: boolean;
    mode: Mode;
  };
  configFieldIndex: number;

  searchModalOpen: boolean;
  searchModalQuery: string;
  searchModalResults: string[];
  searchModalIndex: number;

  toasts: Array<{
    id: string;
    type: 'info' | 'warning' | 'error' | 'success';
    text: string;
    expiresAt: number;
  }>;

  activeFilePath: string;
  cursorPos: { line: number; col: number };
  atPaletteOpen: boolean;
  atPaletteFilter: string;
  atPaletteIndex: number;
  atPaletteFiles: string[];
  spinnerFrame: number;
  undoModalOpen: boolean;

  sidebarOpen: boolean;
  sidebarTab: 'files' | 'tasks' | 'tools';

  splitOpen: boolean;
  splitFocus: 'chat' | 'output';
  outputScrollOffset: number;
}
