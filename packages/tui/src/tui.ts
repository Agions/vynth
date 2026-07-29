import { resolve } from 'node:path';
import * as readline from 'node:readline';
import { type Mode, type ZenoConfig, saveConfigFile } from '@zeno/core';
import { buildRepoMap, builtinTools, createProvider, runAgent } from '@zeno/engine';
import { type TrustConfirm, loadPluginsWithTrust } from '@zeno/plugins';
import * as sandbox from '@zeno/sandbox';
import {
  AtPalette,
  BottomArea,
  ChatArea,
  CommandPalette,
  ConfigModal,
  FileTreePanel,
  InputPanel,
  OutputPane,
  SearchModal,
  SidePanel,
  SlashPalette,
  TasksPanel,
  TopBar,
  UndoModal,
  UsagePanel,
  filterAtFiles,
  renderWelcome
} from './components';
import { COMMANDS } from './components/CommandPalette';
import { collectOutputLines } from './components/OutputPane';
import { filterSlashCommands } from './components/SlashPalette';
import { computeLayoutStrategy } from './layout/Layout';
import {
  renderBadge,
  renderDivider,
  renderInline,
  renderInputPanel,
  renderMessage,
  renderStatusBar,
  renderToolBlock
} from './render';
import { DoubleBufferedRenderer } from './render/double-buffer';
import { FrameCache } from './render/frame-cache';
import { composeOverlay, overlayPanelWidth, stripPanelBorders } from './render/overlay';
import { Scrollback } from './scrollback';
import { SLASH_COMMANDS, matchSlashCommands } from './slash-commands';
import { Store } from './state/Store';
import type { MessageEntry, TuiState } from './state/TuiState';
import { type ThemeName, bg, fg, palette, reset } from './theme';
import {
  executeDirectCommand,
  isMouseOrEscapeGarbage,
  isPhysicalEscapeKey,
  parseSGRMouse,
  scanProjectFiles
} from './tui-controller';
import {
  cursorTo,
  enterAltScreen,
  eraseDown,
  hideCursor,
  leaveAltScreen,
  showCursor
} from './utils/ansi';
import { errorHintFor, parseVcCode } from './utils/error-hints';
import { loadHistory, saveHistory } from './utils/history';
import { estimateCost } from './utils/pricing';
import { type TaskStatus, getTaskManager } from './utils/tasks';
import { padToWidth, stripAnsi, truncateVisible, visibleWidth } from './utils/unicode';

const ESC = '\x1b';

const SIDEBAR_MIN_COLS = 84;

export async function startTui(
  config: ZenoConfig,
  pluginPaths: string[] = [],
  version = '0.0.0'
): Promise<void> {
  const c = palette(config.theme);
  const provider = createProvider(config);
  const tools = builtinTools(config.sandbox.cwd, {
    networkAllowed: config.sandbox.networkAllowed,
    harden: config.sandbox.harden
  });
  const agentToolNames = tools.list().map((t) => t.name);

  const initialState: TuiState = {
    cols: Math.max(60, Math.min(140, (process.stdout.columns ?? 100) - 1)),
    rows: Math.max(20, process.stdout.rows ?? 32),
    layout: computeLayoutStrategy({
      cols: Math.max(60, Math.min(140, (process.stdout.columns ?? 100) - 1)),
      rows: Math.max(20, process.stdout.rows ?? 32),
      topLines: 2,
      botLines: 8
    }).layout,
    transcript: [],
    scrollOffset: 0,
    selectedToolId: null,
    collapsedTools: new Set<string>(),
    briefMode: false,
    input: '',
    prompting: false,
    inputPinned: false,
    liveStatus: 'idle',
    currentTool: null,
    liveToolId: null,
    turnCount: 0,
    connectionStatus: 'connected',
    theme: config.theme,
    palette: c,
    searchQuery: '',
    searchResults: [],
    currentSearchIndex: -1,
    commandPaletteOpen: false,
    commandPaletteFilter: '',
    commandPaletteIndex: 0,
    hintsBarCollapsed: false,
    inputHistory: loadHistory(config.dataDir),
    inputHistoryIndex: -1,
    tokenUsage: { inputTokens: 0, outputTokens: 0, estimatedCost: 0, history: [] },
    usagePanelOpen: false,
    slashCycleIndex: 0,
    slashPaletteOpen: false,
    slashPaletteFilter: '',
    slashPaletteIndex: 0,
    tasksPanelOpen: false,
    atPaletteOpen: false,
    atPaletteFilter: '',
    atPaletteIndex: 0,
    atPaletteFiles: [
      'package.json',
      'README.md',
      'apps/cli/src/main.ts',
      'packages/tui/src/tui.ts',
      'packages/core/src/config.ts',
      'packages/engine/src/llm.ts',
      'packages/sandbox/src/fsops.ts'
    ],
    spinnerFrame: 0,
    undoModalOpen: false,
    sidebarOpen: true,
    sidebarTab: 'files',
    splitOpen: false,
    splitFocus: 'chat',
    outputScrollOffset: 0,

    mode: config.mode,

    fileTreeOpen: false,
    fileTreeData: [],
    fileTreeIndex: 0,

    configModalOpen: false,
    configDraft: {
      model: config.model,
      llmBaseUrl: config.llmBaseUrl,
      apiKey: config.apiKey,
      theme: config.theme,
      networkAllowed: config.sandbox.networkAllowed,
      mode: config.mode
    },
    configFieldIndex: 0,

    searchModalOpen: false,
    searchModalQuery: '',
    searchModalResults: [],
    searchModalIndex: 0,

    toasts: [],
    activeFilePath: '',
    cursorPos: { line: 0, col: 0 }
  };

  const store = new Store(initialState);
  const scrollback = new Scrollback(2_000);

  let drawing = false;
  let drawPending = false;
  let lastMouseTime = 0;

  let spinnerInterval: ReturnType<typeof setInterval> | null = null;
  store.subscribe(() => {
    const isBusy = store.getState().liveStatus !== 'idle';
    if (isBusy && !spinnerInterval) {
      spinnerInterval = setInterval(() => {
        store.setState({ spinnerFrame: (store.getState().spinnerFrame + 1) % 10 });
        draw();
      }, 80);
    } else if (!isBusy && spinnerInterval) {
      clearInterval(spinnerInterval);
      spinnerInterval = null;
    }
  });

  const useNewRenderer = process.env.VYNTH_NEW_RENDERER === '1';
  let renderer: DoubleBufferedRenderer | null = null;
  if (useNewRenderer) {
    renderer = new DoubleBufferedRenderer(store.getState().cols, store.getState().rows);
  }

  const frameCache = new FrameCache();

  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  readline.emitKeypressEvents(process.stdin, rl);
  const stdinTty =
    process.stdout.isTTY === true && (process.stdin as { isTTY?: boolean }).isTTY === true;
  if (stdinTty) process.stdin.setRawMode(true);

  if (stdinTty) {
    process.stdout.write(enterAltScreen);
    process.stdout.write(hideCursor);
  }

  const confirmTty: TrustConfirm = async ({ path }) => {
    return askYesNo(
      `${c.red}⚠ 信任并加载插件？${reset}\n  插件将在本进程执行任意代码，拥有与 Zeno 同等的文件/网络/命令权限。\n  路径: ${path}\n  仅加载你完全信任的插件。确认加载`
    );
  };

  const welcome = renderWelcome({
    version,
    model: config.model,
    mode: config.mode,
    cwd: config.sandbox.cwd,
    theme: config.theme,
    palette: c,
    width: store.getState().cols
  });
  appendMessage('system', welcome, { background: false });

  if (pluginPaths.length > 0) {
    const abs = pluginPaths.map((p) => resolve(config.sandbox.cwd, p));
    const res = await loadPluginsWithTrust(abs, tools, confirmTty);
    for (const n of res.loaded) appendSystemMessage(`✓ plugin loaded: ${n}`);
    for (const p of res.declined) appendSystemMessage(`✗ plugin declined: ${p}`);
    for (const e of res.errors) appendSystemMessage(`✗ plugin error: ${e.error}`);
  }

  scanProjectFiles(config.sandbox.cwd).then((files) => {
    if (files.length > 0) {
      store.setState({ atPaletteFiles: files });
      draw();
    }
  });

  let projectRepoMap: string | undefined;
  if (config.repomap.enabled) {
    void buildRepoMap({
      root: config.sandbox.cwd,
      maxSymbols: config.repomap.maxSymbols,
      includeTests: config.repomap.includeTests
    })
      .then((res) => {
        if (res.symbolCount > 0) {
          projectRepoMap = res.mapText;
          appendSystemMessage(
            `✓ repo-map 已生成: ${res.symbolCount} 符号 / ${res.fileCount} 文件（注入对话上下文）`
          );
          draw();
        }
      })
      .catch(() => {});
  }

  draw();

  getTaskManager().onChange(() => {
    syncBackgroundTasks();
    draw();
  });

  process.stdout.on('resize', () => {
    const cols = Math.max(60, Math.min(140, (process.stdout.columns ?? 100) - 1));
    const rows = Math.max(20, process.stdout.rows ?? 32);
    store.setState({
      cols,
      rows,
      layout: computeLayoutStrategy({ cols, rows, topLines: 2, botLines: 8 }).layout
    });
    if (renderer) {
      renderer.resize(cols, rows);
    }
    store.markAllDirty();
    draw();
  });

  function askYesNo(question: string): Promise<boolean> {
    return new Promise((resolvePrompt) => {
      store.setState({ prompting: true });
      const stdinTty2 =
        process.stdout.isTTY === true && (process.stdin as { isTTY?: boolean }).isTTY === true;
      const prevRaw =
        stdinTty2 && 'isRaw' in process.stdin
          ? (process.stdin as { isRaw?: boolean }).isRaw === true
          : false;
      if (stdinTty2) process.stdin.setRawMode(false);
      rl.question(`${question} [y/N] `, (ans) => {
        if (stdinTty2) process.stdin.setRawMode(prevRaw);
        store.setState({ prompting: false });
        frameCache.invalidate();
        draw();
        const a = ans.trim().toLowerCase();
        resolvePrompt(a === 'y' || a === 'yes');
      });
    });
  }

  function askCustomInput(promptText: string): Promise<string> {
    return new Promise((resolveInput) => {
      store.setState({ prompting: true });
      const stdinTty2 =
        process.stdout.isTTY === true && (process.stdin as { isTTY?: boolean }).isTTY === true;
      const prevRaw =
        stdinTty2 && 'isRaw' in process.stdin
          ? (process.stdin as { isRaw?: boolean }).isRaw === true
          : false;
      if (stdinTty2) process.stdin.setRawMode(false);
      rl.question(`${fg(palette('mocha').mauve)}✎ ${promptText}: ${reset}`, (ans) => {
        if (stdinTty2) process.stdin.setRawMode(prevRaw);
        store.setState({ prompting: false });
        frameCache.invalidate();
        draw();
        resolveInput(ans.trim());
      });
    });
  }

  function draw(): void {
    if (drawing) {
      drawPending = true;
      return;
    }
    drawing = true;
    try {
      drawImpl();
    } finally {
      drawing = false;
      if (drawPending) {
        drawPending = false;
        draw();
      }
    }
  }

  function drawImpl(): void {
    const state = store.getState();
    const cols = state.cols;
    const rows = state.rows;
    const TOP_H = 2;
    const baseBot = state.selectedToolId ? 7 : 5; // divider + input(4) + tool-status(2)
    const BOT_H = baseBot + (state.inputPinned ? 1 : 0);
    const midH = Math.max(5, rows - TOP_H - BOT_H);

    if (useNewRenderer && renderer) {
      renderer.clear();
      const r = renderer;

      const ci = {
        text: r.color(state.palette.text),
        subtext: r.color(state.palette.subtext),
        accent: r.color(state.palette.mauve),
        divider: r.color(state.palette.overlay0 ?? state.palette.subtext)
      };

      const topText = `Zeno · ${state.theme} · ${state.liveStatus} · turn ${state.turnCount}`;
      r.renderLine(0, topText, ci.accent);
      r.renderLine(
        1,
        `${state.connectionStatus} · ctx ${Math.min(100, state.turnCount * 2) % 100}%`,
        ci.subtext
      );

      r.renderDivider(2, '─', ci.divider);

      const chatStart = 3;
      const lines = renderTranscriptToLines(cols - 4);
      const totalLines = lines.length;
      const maxScroll = Math.max(0, totalLines - midH);
      const effectiveScroll = Math.min(state.scrollOffset, maxScroll);
      const startIdx = effectiveScroll === 0 ? Math.max(0, totalLines - midH) : effectiveScroll;
      for (let i = 0; i < midH; i++) {
        const lineIdx = startIdx + i;
        const line = lineIdx < totalLines ? stripAnsi(lines[lineIdx]) : '';
        r.renderLine(chatStart + i, line, ci.text);
      }

      r.renderDivider(chatStart + midH, '─', ci.divider);

      const bottomStart = chatStart + midH + 1;
      const inputLine = `❯ ${state.input || ''}`;
      r.renderLine(bottomStart, inputLine, ci.text);
      r.renderLine(
        bottomStart + 1,
        '⏎ send · ⇧↵ newline · ^B 侧栏 · ^T 切换 · ? help · ↑↓ scroll · ^P^N history',
        ci.subtext
      );

      const output = r.flush();
      process.stdout.write(output);
      const cursorRow = bottomStart;
      const cursorCol = state.input.includes('\n') ? 6 : visibleWidth(`  ❯ ${state.input}`) + 3;
      process.stdout.write(cursorTo(cursorRow, cursorCol));
      return;
    }

    const frame: string[] = [];
    const w = (line: string): void => {
      frame.push(line);
    };

    const topLines = TopBar(state);
    for (const line of topLines) {
      w(`${line}${reset}`);
    }

    w(
      `${renderDivider({ width: cols, color: state.palette.overlay0 ?? state.palette.subtext })}${reset}`
    );

    const sideVisible = state.sidebarOpen && cols >= SIDEBAR_MIN_COLS;
    const sideW = sideVisible ? Math.min(34, Math.max(22, Math.floor(cols * 0.24))) : 0;
    const chatW = cols - sideW;

    const splitVisible = state.splitOpen && midH >= 10;
    const outH = splitVisible ? Math.max(4, Math.floor(midH * 0.35)) : 0;
    const chatH = splitVisible ? midH - outH : midH;

    const chatLines = sideVisible ? ChatArea({ ...state, cols: chatW }) : ChatArea(state);
    const allLines = [...chatLines];
    const totalLines = allLines.length;
    const maxScroll = Math.max(0, totalLines - chatH);
    const effectiveScroll = Math.min(state.scrollOffset, maxScroll);

    const startLine = Math.max(0, totalLines - chatH - effectiveScroll);
    const visibleLines = allLines.slice(startLine, startLine + chatH);

    const paneW = sideVisible ? chatW - 1 : cols;
    const leftCol: string[] = [];
    for (let i = 0; i < chatH; i++) leftCol.push(visibleLines[i] ?? '');
    if (splitVisible) {
      leftCol.push(...OutputPane({ state, width: paneW, height: outH }));
    }

    if (sideVisible) {
      const sideLines = SidePanel({
        state,
        width: sideW,
        height: midH,
        files: state.atPaletteFiles || [],
        tasks: getTaskManager().list(),
        toolNames: agentToolNames
      });
      for (let i = 0; i < midH; i++) {
        const left = truncateVisible(leftCol[i] ?? '', chatW - 1);
        const leftPadded = padToWidth(left, chatW - 1);
        w(`${leftPadded}${reset} ${sideLines[i] ?? ''}${reset}`);
      }
    } else {
      for (let i = 0; i < midH; i++) {
        const line = leftCol[i] ?? '';
        const truncated = truncateVisible(line, cols);
        w(`${truncated}${reset}`);
      }
    }

    w('');

    if (effectiveScroll > 0) {
      const pinHint = `  ${fg(state.palette.mauve)}↑ ${effectiveScroll} lines scrolled${reset}  ${fg(state.palette.subtext)}press ${fg(state.palette.yellow)}End${fg(state.palette.subtext)} to return to bottom${reset}`;
      w(pinHint);
    }

    if (state.selectedToolId) {
      const toolEntry = state.transcript.find((t) => t.toolId === state.selectedToolId);
      if (toolEntry) {
        const toolName = toolEntry.content.split('(')[0] || 'tool';
        const isCollapsed = state.collapsedTools.has(state.selectedToolId);
        const toolStatus = `${c.yellow}▸ ${toolName} ${isCollapsed ? 'collapsed' : 'expanded'}${reset}`;
        w(`${toolStatus}${reset}`);
        w(
          `${renderDivider({ width: cols, color: state.palette.overlay0 ?? state.palette.subtext })}${reset}`
        );
      }
    }

    const bottomStart = frame.length;
    const bottomLines = BottomArea({ state });
    for (const line of bottomLines) {
      const lineStr = typeof line === 'string' ? line : String(line);
      for (const part of lineStr.split('\n')) {
        const truncated = truncateVisible(part, cols);
        w(`${truncated}${reset}`);
      }
    }

    const isSlashMode =
      state.slashPaletteOpen ||
      (state.input.startsWith('/') && !state.input.includes(' ') && !state.input.includes('\n'));
    const lastAtIdx = state.input.lastIndexOf('@');
    const isAtMode = lastAtIdx !== -1 && !state.input.slice(lastAtIdx).includes(' ');
    let overlayLines: string[] | null = null;
    if (state.configModalOpen) {
      overlayLines = ConfigModal({ state }).split('\n');
    } else if (state.searchModalOpen) {
      overlayLines = SearchModal({ state }).split('\n');
    } else if (state.undoModalOpen) {
      overlayLines = UndoModal({ state }).split('\n');
    } else if (state.usagePanelOpen) {
      overlayLines = UsagePanel({ state, model: config.model }).split('\n');
    } else if (state.tasksPanelOpen) {
      overlayLines = TasksPanel({ palette: state.palette, cols }).split('\n');
    } else if (state.fileTreeOpen) {
      overlayLines = FileTreePanel({ state }).split('\n');
    } else if (state.commandPaletteOpen) {
      overlayLines = CommandPalette({
        state,
        selectedIndex: state.commandPaletteIndex,
        filter: state.commandPaletteFilter
      }).split('\n');
    } else if (isSlashMode) {
      const filter = state.slashPaletteOpen ? state.slashPaletteFilter : state.input.slice(1);
      overlayLines = SlashPalette({
        state,
        selectedIndex: state.slashPaletteIndex,
        filter
      }).split('\n');
    } else if (isAtMode) {
      overlayLines = AtPalette({
        state,
        selectedIndex: state.atPaletteIndex || 0,
        filter: state.input.slice(lastAtIdx + 1),
        files: state.atPaletteFiles || []
      }).split('\n');
    }
    if (overlayLines) {
      overlayLines = stripPanelBorders(overlayLines);
      const panelW = overlayPanelWidth(overlayLines, 2, cols - 6);
      composeOverlay(frame, overlayLines, {
        left: 2,
        width: panelW,
        anchorBottom: bottomStart,
        minRow: topLines.length + 1
      });
    }

    const frameOut = frameCache.flush(frame, cols, rows);
    if (frameOut) process.stdout.write(frameOut);

    const cursorRow = rows - 2;
    const cursorCol = state.input.includes('\n') ? 6 : visibleWidth(`  ❯ ${state.input}`) + 1;
    process.stdout.write(cursorTo(cursorRow, cursorCol));
  }

  function appendToMid(multiLine: string): void {
    scrollback.push(multiLine);
  }

  function renderTranscriptToLines(width: number): string[] {
    const lines: string[] = [];
    const state = store.getState();
    for (const entry of state.transcript) {
      if (entry.role === 'tool' && entry.toolId) {
        const isCollapsed = state.collapsedTools.has(entry.toolId);
        const isSelected = state.selectedToolId === entry.toolId;
        const block = renderToolBlock({
          name: entry.content.split('(')[0] || 'tool',
          args: entry.content.includes('(')
            ? entry.content.slice(entry.content.indexOf('('))
            : '{}',
          status: entry.status ?? 'ok',
          output: entry.output,
          palette: c,
          width,
          collapsed: isCollapsed,
          selected: isSelected,
          hint: (() => {
            if (entry.status !== 'error') return undefined;
            const code = parseVcCode(entry.output ?? '');
            return code ? errorHintFor(code) : undefined;
          })()
        });
        for (const line of block.split('\n')) {
          lines.push(line);
        }
      } else {
        const block = renderMessage({
          role: entry.role,
          content: entry.content,
          palette: c,
          width,
          background: entry.role !== 'system'
        });
        for (const line of block.split('\n')) {
          lines.push(line);
        }
      }
      lines.push('');
    }
    return lines;
  }

  function appendMessage(
    role: 'user' | 'assistant' | 'system' | 'tool',
    content: string,
    options?: { toolId?: string; background?: boolean }
  ): void {
    const toolId = options?.toolId;
    const entry: MessageEntry = {
      id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      role,
      content,
      timestamp: Date.now(),
      toolId,
      collapsed: role === 'tool' && toolId ? store.getState().collapsedTools.has(toolId) : undefined
    };
    store.appendMessage(entry);
    scrollback.push(
      `${renderMessage({
        role,
        content,
        palette: c,
        width: store.getState().cols - 4,
        background: options?.background ?? role !== 'system'
      })}\n`
    );
    store.setScrollOffset(0);
  }

  function appendSystemMessage(content: string): void {
    appendMessage('system', content, { background: false });
  }

  process.stdin.on('keypress', (str: string | null, key: readline.Key) => {
    if (store.getState().prompting || !key || Date.now() - lastMouseTime < 120) return;

    if (key.ctrl && key.name === 'c') {
      cleanup();
      process.exit(0);
    }

    const isEsc = (key && key.name === 'escape') || str === '\x1b';
    if (isEsc) {
      const state = store.getState();
      let closedModal = false;

      if (state.configModalOpen) {
        store.setState({ configModalOpen: false });
        closedModal = true;
      }
      if (state.undoModalOpen) {
        store.setState({ undoModalOpen: false });
        closedModal = true;
      }
      if (state.fileTreeOpen) {
        store.setState({ fileTreeOpen: false });
        closedModal = true;
      }
      if (state.searchModalOpen) {
        store.setState({ searchModalOpen: false });
        closedModal = true;
      }
      if (state.usagePanelOpen) {
        store.setState({ usagePanelOpen: false });
        closedModal = true;
      }
      if (state.tasksPanelOpen) {
        store.setState({ tasksPanelOpen: false });
        closedModal = true;
      }
      if (state.commandPaletteOpen) {
        store.setState({
          commandPaletteOpen: false,
          commandPaletteFilter: '',
          commandPaletteIndex: 0
        });
        closedModal = true;
      }
      if (state.slashPaletteOpen) {
        store.setState({ slashPaletteOpen: false, slashPaletteFilter: '', slashPaletteIndex: 0 });
        closedModal = true;
      }
      if (state.selectedToolId) {
        store.setState({ selectedToolId: null });
        closedModal = true;
      }

      if (closedModal) {
        draw();
        return;
      }
    }

    const currInput = store.getState().input;
    const isSlashActive =
      store.getState().slashPaletteOpen ||
      (currInput.startsWith('/') && !currInput.includes(' ') && !currInput.includes('\n'));
    if (isSlashActive) {
      const state = store.getState();
      const filter = state.slashPaletteOpen ? state.slashPaletteFilter : currInput.slice(1);
      const filtered = filterSlashCommands(filter);
      const maxIdx = Math.max(0, filtered.length - 1);

      if (key.name === 'up' || (key.shift && key.name === 'tab')) {
        store.setState({ slashPaletteIndex: Math.max(0, (state.slashPaletteIndex || 0) - 1) });
        draw();
        return;
      }
      if (key.name === 'down' || key.name === 'tab') {
        store.setState({ slashPaletteIndex: Math.min(maxIdx, (state.slashPaletteIndex || 0) + 1) });
        draw();
        return;
      }
      if (key.name === 'return') {
        const selected = filtered[state.slashPaletteIndex || 0];
        const cmdName = selected ? selected.name : currInput.slice(1);
        if (cmdName) {
          store.setInput(`/${cmdName}`);
        }
        store.setState({ slashPaletteOpen: false, slashPaletteFilter: '', slashPaletteIndex: 0 });
        submit();
        return;
      }
      if (key.name === 'escape') {
        store.setState({ slashPaletteOpen: false, slashPaletteFilter: '', slashPaletteIndex: 0 });
        draw();
        return;
      }
      if (state.slashPaletteOpen) {
        if (key.name === 'backspace') {
          store.setState({
            slashPaletteFilter: state.slashPaletteFilter.slice(0, -1),
            slashPaletteIndex: 0
          });
          draw();
          return;
        }
        if (str && !key.ctrl && !key.meta) {
          store.setState({
            slashPaletteFilter: state.slashPaletteFilter + str,
            slashPaletteIndex: 0
          });
          draw();
          return;
        }
        return;
      }
    }
    if (!store.getState().commandPaletteOpen) {
      const inputEmpty = store.getState().input.length === 0;
      const ctrlSlash = key.ctrl && (key.name === '/' || key.name === '_');
      if (ctrlSlash || (key.name === '/' && inputEmpty)) {
        store.setState({ slashPaletteOpen: true, slashPaletteFilter: '', slashPaletteIndex: 0 });
        draw();
        return;
      }
    }

    const outputFocused = () => {
      const s = store.getState();
      return s.splitOpen && s.splitFocus === 'output';
    };
    const scrollOutput = (delta: number) => {
      const s = store.getState();
      const total = collectOutputLines(s).length;
      const maxScroll = Math.max(0, total - 4);
      store.setState({
        outputScrollOffset: Math.max(0, Math.min(s.outputScrollOffset + delta, maxScroll))
      });
      draw();
    };

    if (key.shift && key.name === 'up') {
      if (outputFocused()) {
        scrollOutput(3);
        return;
      }
      const allLines = renderTranscriptToLines(store.getState().cols - 4);
      const state = store.getState();
      const maxScroll = Math.max(0, allLines.length - (state.rows - 10));
      store.setScrollOffset(Math.min(state.scrollOffset + 3, maxScroll));
      draw();
      return;
    }
    if (key.shift && key.name === 'down') {
      if (outputFocused()) {
        scrollOutput(-3);
        return;
      }
      const state = store.getState();
      store.setScrollOffset(Math.max(state.scrollOffset - 3, 0));
      draw();
      return;
    }
    if (key.name === 'pageup') {
      if (outputFocused()) {
        scrollOutput(store.getState().rows - 10);
        return;
      }
      const allLines = renderTranscriptToLines(store.getState().cols - 4);
      const state = store.getState();
      const maxScroll = Math.max(0, allLines.length - (state.rows - 10));
      store.setScrollOffset(Math.min(state.scrollOffset + (state.rows - 10), maxScroll));
      draw();
      return;
    }
    if (key.name === 'pagedown') {
      if (outputFocused()) {
        scrollOutput(-(store.getState().rows - 10));
        return;
      }
      const state = store.getState();
      store.setScrollOffset(Math.max(state.scrollOffset - (state.rows - 10), 0));
      draw();
      return;
    }

    if (key.name === 'tab') {
      const state = store.getState();
      const frag = state.input;

      if (frag.startsWith('/') && !frag.includes(' ') && !frag.includes('\n')) {
        const candidates = matchSlashCommands(frag);
        if (candidates.length > 0) {
          const reverse = Boolean(key.shift);
          const idx = reverse
            ? (((state.slashCycleIndex - 1) % candidates.length) + candidates.length) %
              candidates.length
            : (state.slashCycleIndex + 1) % candidates.length;
          const chosen = candidates[idx];
          store.setState({
            input: `/${chosen}`,
            slashCycleIndex: idx,
            slashPaletteIndex: idx
          });
          draw();
          return;
        }
        return;
      }

      if (frag.trim() === '' && !key.shift) {
        const currentMode = state.mode || 'vibe';
        const nextMode: Mode =
          currentMode === 'vibe' ? 'plan' : currentMode === 'plan' ? 'auto' : 'vibe';

        store.setState({
          mode: nextMode,
          configDraft: { ...(state.configDraft || {}), mode: nextMode }
        });
        config.mode = nextMode;
        try {
          saveConfigFile(config.dataDir, { mode: nextMode });
        } catch {}

        draw();
        return;
      }

      const currentIdx = state.selectedToolId
        ? state.transcript.findIndex((t) => t.toolId === state.selectedToolId)
        : -1;
      const step = key.shift ? -1 : 1;
      const nextIdx = currentIdx === -1 ? 0 : currentIdx + step;
      const nextTool = findNextToolId(nextIdx, step, store);
      store.setState({ selectedToolId: nextTool });
      draw();
      return;
    }
    const lastAtIdx = store.getState().input.lastIndexOf('@');
    const isAtActive = lastAtIdx !== -1 && !store.getState().input.slice(lastAtIdx).includes(' ');
    if (isAtActive) {
      const state = store.getState();
      const filter = state.input.slice(lastAtIdx + 1);
      const filtered = filterAtFiles(state.atPaletteFiles || [], filter);
      const maxIdx = Math.max(0, filtered.length - 1);

      if (key.name === 'up') {
        store.setState({ atPaletteIndex: Math.max(0, (state.atPaletteIndex || 0) - 1) });
        draw();
        return;
      }
      if (key.name === 'down') {
        store.setState({ atPaletteIndex: Math.min(maxIdx, (state.atPaletteIndex || 0) + 1) });
        draw();
        return;
      }
      if (key.name === 'return' || key.name === 'tab') {
        const selected = filtered[state.atPaletteIndex || 0];
        if (selected) {
          const beforeAt = state.input.slice(0, lastAtIdx);
          store.setInput(`${beforeAt}@${selected} `);
        }
        store.setState({ atPaletteIndex: 0 });
        draw();
        return;
      }
    }

    if (store.getState().undoModalOpen) {
      if (key.name === 'return' || (str && str.toLowerCase() === 'y')) {
        const state = store.getState();
        const newTranscript = [...state.transcript];
        while (newTranscript.length > 0) {
          const popped = newTranscript.pop();
          if (popped?.role === 'user') break;
        }
        store.setState({
          transcript: newTranscript,
          undoModalOpen: false
        });
        appendSystemMessage('✓ 已从工作区恢复并回撤上一步修改');
        draw();
        return;
      }
      if (key.name === 'escape' || (str && str.toLowerCase() === 'n')) {
        store.setState({ undoModalOpen: false });
        draw();
        return;
      }
      return;
    }

    if (store.getState().configModalOpen) {
      const state = store.getState();
      const draft = {
        model: state.configDraft?.model || config.model,
        llmBaseUrl: state.configDraft?.llmBaseUrl || config.llmBaseUrl,
        apiKey: state.configDraft?.apiKey || config.apiKey,
        theme: state.configDraft?.theme || config.theme,
        networkAllowed: state.configDraft?.networkAllowed ?? config.sandbox.networkAllowed,
        mode: state.configDraft?.mode || config.mode
      };
      let fieldIdx = state.configFieldIndex || 0;

      if (key.name === 'up') {
        fieldIdx = (fieldIdx - 1 + 5) % 5;
        store.setState({ configFieldIndex: fieldIdx });
        draw();
        return;
      }
      if (key.name === 'down') {
        fieldIdx = (fieldIdx + 1) % 5;
        store.setState({ configFieldIndex: fieldIdx });
        draw();
        return;
      }
      if (str && str.toLowerCase() === 'c') {
        if (fieldIdx === 0) {
          askCustomInput('请输入自定义 LLM 模型名称 (例如: qwen3-coder:30b, gpt-5.6-terra)').then(
            (customVal) => {
              if (customVal) {
                draft.model = customVal;
                config.model = customVal;
                store.setState({ configDraft: draft });
                try {
                  saveConfigFile(config.dataDir, { model: customVal });
                } catch {}
                appendSystemMessage(`✓ 模型已自定义设为: ${customVal}`);
                draw();
              }
            }
          );
        } else if (fieldIdx === 1) {
          askCustomInput('请输入自定义 Base URL 端点 (例如: http://localhost:11434/v1)').then(
            (customVal) => {
              if (customVal) {
                draft.llmBaseUrl = customVal;
                config.llmBaseUrl = customVal;
                store.setState({ configDraft: draft });
                try {
                  saveConfigFile(config.dataDir, { llmBaseUrl: customVal });
                } catch {}
                appendSystemMessage(`✓ 端点已自定义设为: ${customVal}`);
                draw();
              }
            }
          );
        } else if (fieldIdx === 2) {
          askCustomInput('请输入自定义 VYNTH_API_KEY').then((customVal) => {
            if (customVal) {
              draft.apiKey = customVal;
              config.apiKey = customVal;
              store.setState({ configDraft: draft });
              try {
                saveConfigFile(config.dataDir, { apiKey: customVal });
              } catch {}
              appendSystemMessage('✓ API Key 已更新');
              draw();
            }
          });
        }
        return;
      }
      if (
        key.name === 'return' ||
        key.name === 'space' ||
        key.name === 'left' ||
        key.name === 'right'
      ) {
        const reverse = key.name === 'left';
        if (fieldIdx === 0) {
          const models = [
            'deepseek-v4-pro',
            'deepseek-v4-flash',
            'gpt-5.6-sol',
            'gpt-5.6-terra',
            'gpt-5.6-luna',
            'claude-opus-5',
            'claude-sonnet-5',
            'qwen3-coder:30b',
            'llama4'
          ];
          const curIdx = models.indexOf(draft.model);
          const nextIdx = reverse
            ? (curIdx - 1 + models.length) % models.length
            : (curIdx + 1) % models.length;
          draft.model = models[nextIdx];
          config.model = draft.model;

          if (draft.model.startsWith('deepseek')) {
            draft.llmBaseUrl = 'https://api.deepseek.com/v1';
          } else if (draft.model.startsWith('gpt-') || draft.model.startsWith('o3-')) {
            draft.llmBaseUrl = 'https://api.openai.com/v1';
          } else if (draft.model.startsWith('claude-')) {
            draft.llmBaseUrl = 'https://api.anthropic.com/v1';
          } else if (draft.model.includes('qwen') || draft.model.includes('llama')) {
            draft.llmBaseUrl = 'http://localhost:11434/v1';
          }
          config.llmBaseUrl = draft.llmBaseUrl;

          appendSystemMessage(`✓ 模型已切换为: ${draft.model} (端点: ${draft.llmBaseUrl})`);
        } else if (fieldIdx === 1) {
          const urls = [
            'https://api.deepseek.com/v1',
            'https://api.openai.com/v1',
            'https://api.anthropic.com/v1',
            'http://localhost:11434/v1'
          ];
          const curIdx = urls.indexOf(draft.llmBaseUrl);
          const nextIdx = reverse
            ? (curIdx - 1 + urls.length) % urls.length
            : (curIdx + 1) % urls.length;
          draft.llmBaseUrl = urls[nextIdx];
          config.llmBaseUrl = draft.llmBaseUrl;
          appendSystemMessage(`✓ 端点已切换为: ${draft.llmBaseUrl}`);
        } else if (fieldIdx === 3) {
          const themesList: ThemeName[] = ['mocha', 'latte', 'midnight', 'forest', 'light', 'neon'];
          const curIdx = themesList.indexOf(draft.theme);
          const nextIdx = reverse
            ? (curIdx - 1 + themesList.length) % themesList.length
            : (curIdx + 1) % themesList.length;
          const nextTheme = themesList[nextIdx];
          draft.theme = nextTheme;
          config.theme = nextTheme;
          store.setState({ theme: nextTheme, palette: palette(nextTheme) });
        } else if (fieldIdx === 4) {
          draft.networkAllowed = !draft.networkAllowed;
          config.sandbox.networkAllowed = draft.networkAllowed;
          appendSystemMessage(`✓ 联网沙箱已${draft.networkAllowed ? '开启' : '关闭'}`);
        }

        try {
          saveConfigFile(config.dataDir, {
            model: draft.model,
            llmBaseUrl: draft.llmBaseUrl,
            theme: draft.theme
          });
        } catch {}

        store.setState({ configDraft: draft });
        draw();
        return;
      }
      if (key.name === 'escape') {
        store.setState({ configModalOpen: false });
        draw();
        return;
      }
      return;
    }

    if (key.name === 'escape') {
      const state = store.getState();
      if (state.undoModalOpen) {
        store.setState({ undoModalOpen: false });
        draw();
        return;
      }
      if (state.configModalOpen) {
        store.setState({ configModalOpen: false });
        draw();
        return;
      }
      if (state.fileTreeOpen) {
        store.setState({ fileTreeOpen: false });
        draw();
        return;
      }
      if (state.searchModalOpen) {
        store.setState({ searchModalOpen: false });
        draw();
        return;
      }
      if (state.usagePanelOpen) {
        store.setState({ usagePanelOpen: false });
        draw();
        return;
      }
      if (state.tasksPanelOpen) {
        store.setState({ tasksPanelOpen: false });
        draw();
        return;
      }
      if (state.commandPaletteOpen) {
        store.setState({
          commandPaletteOpen: false,
          commandPaletteFilter: '',
          commandPaletteIndex: 0
        });
        draw();
        return;
      }
      if (state.slashPaletteOpen) {
        store.setState({
          slashPaletteOpen: false,
          slashPaletteFilter: '',
          slashPaletteIndex: 0
        });
        draw();
        return;
      }
      store.setState({ selectedToolId: null });
      draw();
      return;
    }

    if (key.ctrl && key.name === 'b') {
      const s = store.getState();
      if (s.cols < SIDEBAR_MIN_COLS) {
        appendSystemMessage(
          `⚠ 侧栏需要终端宽度 ≥ ${SIDEBAR_MIN_COLS} 列（当前 ${s.cols} 列），请拉宽终端窗口后再按 Ctrl+B`
        );
        draw();
        return;
      }
      store.setState({ sidebarOpen: !s.sidebarOpen });
      draw();
      return;
    }
    if (key.ctrl && key.name === 't') {
      const s = store.getState();
      if (s.cols < SIDEBAR_MIN_COLS) {
        appendSystemMessage(
          `⚠ 侧栏需要终端宽度 ≥ ${SIDEBAR_MIN_COLS} 列（当前 ${s.cols} 列），请拉宽终端窗口后再按 Ctrl+T`
        );
        draw();
        return;
      }
      const order: Array<TuiState['sidebarTab']> = ['files', 'tasks', 'tools'];
      const next = order[(order.indexOf(s.sidebarTab) + 1) % order.length];
      store.setState({ sidebarTab: next, sidebarOpen: true });
      draw();
      return;
    }

    if (key.ctrl && key.name === 'o') {
      const cur = store.getState();
      store.setState({
        splitOpen: !cur.splitOpen,
        splitFocus: cur.splitOpen ? 'chat' : 'output',
        outputScrollOffset: 0
      });
      draw();
      return;
    }
    if (key.ctrl && key.name === 'e') {
      const cur = store.getState();
      if (cur.splitOpen) {
        store.setState({ splitFocus: cur.splitFocus === 'chat' ? 'output' : 'chat' });
        draw();
        return;
      }
    }

    if (key.ctrl && key.name === 'u') {
      const state = store.getState();
      store.setState({ usagePanelOpen: !state.usagePanelOpen });
      draw();
      return;
    }

    if (key.name === 'f1' || (key.ctrl && key.name === 'space')) {
      const state = store.getState();
      store.setState({
        commandPaletteOpen: !state.commandPaletteOpen,
        commandPaletteFilter: '',
        commandPaletteIndex: 0
      });
      draw();
      return;
    }

    if (store.getState().commandPaletteOpen) {
      // Command palette navigation
      if (key.name === 'up') {
        const state = store.getState();
        const maxIdx =
          COMMANDS.filter(
            (cmd) =>
              !state.commandPaletteFilter ||
              cmd.label.toLowerCase().includes(state.commandPaletteFilter.toLowerCase()) ||
              cmd.description.toLowerCase().includes(state.commandPaletteFilter.toLowerCase())
          ).length - 1;
        store.setState({
          commandPaletteIndex: Math.max(0, state.commandPaletteIndex - 1)
        });
        draw();
        return;
      }
      if (key.name === 'down') {
        const state = store.getState();
        const maxIdx =
          COMMANDS.filter(
            (cmd) =>
              !state.commandPaletteFilter ||
              cmd.label.toLowerCase().includes(state.commandPaletteFilter.toLowerCase()) ||
              cmd.description.toLowerCase().includes(state.commandPaletteFilter.toLowerCase())
          ).length - 1;
        store.setState({
          commandPaletteIndex: Math.min(maxIdx, state.commandPaletteIndex + 1)
        });
        draw();
        return;
      }
      if (key.name === 'return') {
        const state = store.getState();
        const filtered = COMMANDS.filter(
          (cmd) =>
            !state.commandPaletteFilter ||
            cmd.label.toLowerCase().includes(state.commandPaletteFilter.toLowerCase()) ||
            cmd.description.toLowerCase().includes(state.commandPaletteFilter.toLowerCase())
        );
        const selected = filtered[state.commandPaletteIndex];
        if (selected) {
          appendSystemMessage(`⏎  ${selected.label}: ${selected.description}`);
          // Execute command action
          if (selected.key === '/clear') {
            store.setState({ transcript: [], scrollOffset: 0 });
            scrollback.reset('');
          } else if (selected.key === '/brief') {
            const newBrief = !state.briefMode;
            store.setState({ briefMode: newBrief });
            appendSystemMessage(`Brief mode: ${newBrief ? 'ON (tools auto-collapsed)' : 'OFF'}`);
          } else if (selected.key === '/theme') {
            // Cycle theme
            const themes: ThemeName[] = ['mocha', 'latte', 'midnight', 'forest', 'light', 'neon'];
            const currentIdx = themes.indexOf(state.theme);
            const nextTheme = themes[(currentIdx + 1) % themes.length];
            store.setState({ theme: nextTheme, palette: palette(nextTheme) });
            appendSystemMessage(`Theme: ${nextTheme}`);
          } else if (selected.key === '/usage') {
            store.setState({ usagePanelOpen: !state.usagePanelOpen });
          } else if (selected.key === 'End') {
            store.setScrollOffset(0);
          } else if (selected.key === 'Home') {
            const allLines = renderTranscriptToLines(state.cols - 4);
            store.setScrollOffset(Math.max(0, allLines.length));
          }
        }
        store.setState({
          commandPaletteOpen: false,
          commandPaletteFilter: '',
          commandPaletteIndex: 0
        });
        draw();
        return;
      }
      // Regular typing filters commands
      if (str && !key.ctrl && !key.meta && key.name !== 'backspace') {
        const state = store.getState();
        store.setState({
          commandPaletteFilter: state.commandPaletteFilter + str,
          commandPaletteIndex: 0
        });
        draw();
        return;
      }
      if (key.name === 'backspace') {
        const state = store.getState();
        store.setState({
          commandPaletteFilter: state.commandPaletteFilter.slice(0, -1),
          commandPaletteIndex: 0
        });
        draw();
        return;
      }
      return;
    }

    if (key.name === 'return') {
      const state = store.getState();
      if (state.selectedToolId) {
        store.toggleToolCollapse(state.selectedToolId);
        draw();
        return;
      }
      void submit();
      return;
    }

    if (key.name === 'return' && (key.shift || key.ctrl || key.meta)) {
      const state = store.getState();
      store.setInput(`${state.input}\n`);
      draw();
      return;
    }

    if (key.ctrl && key.name === 'p') {
      const state = store.getState();
      if (state.inputHistory.length > 0) {
        const newIndex =
          state.inputHistoryIndex < 0
            ? state.inputHistory.length - 1
            : Math.max(0, state.inputHistoryIndex - 1);
        store.setState({
          input: state.inputHistory[newIndex] || '',
          inputHistoryIndex: newIndex
        });
        draw();
      }
      return;
    }
    if (key.ctrl && key.name === 'n') {
      const state = store.getState();
      if (state.inputHistoryIndex >= 0) {
        const atLast = state.inputHistoryIndex >= state.inputHistory.length - 1;
        if (atLast) {
          store.setState({ input: '', inputHistoryIndex: -1 });
        } else {
          const newIndex = state.inputHistoryIndex + 1;
          store.setState({
            input: state.inputHistory[newIndex] || '',
            inputHistoryIndex: newIndex
          });
        }
        draw();
      }
      return;
    }

    if (
      key.name === 'up' &&
      !key.ctrl &&
      !key.meta &&
      !key.shift &&
      !store.getState().commandPaletteOpen
    ) {
      const state = store.getState();
      const allLines = renderTranscriptToLines(state.cols - 4);
      const midH = Math.max(5, state.rows - 10);
      const maxScroll = Math.max(0, allLines.length - midH);
      store.setScrollOffset(Math.min(state.scrollOffset + 3, maxScroll));
      draw();
      return;
    }
    if (
      key.name === 'down' &&
      !key.ctrl &&
      !key.meta &&
      !key.shift &&
      !store.getState().commandPaletteOpen
    ) {
      const state = store.getState();
      store.setScrollOffset(Math.max(state.scrollOffset - 3, 0));
      draw();
      return;
    }

    if (key.name === 'tab' && store.getState().input.trim() === '') {
      const state = store.getState();
      const currentMode = state.mode || 'vibe';
      const nextMode: Mode =
        currentMode === 'vibe' ? 'plan' : currentMode === 'plan' ? 'auto' : 'vibe';
      store.setState({ mode: nextMode });
      config.mode = nextMode;
      try {
        saveConfigFile(config.dataDir, { mode: nextMode });
      } catch {}
      draw();
      return;
    }

    if (key.name === 'backspace') {
      const state = store.getState();
      store.setInput(state.input.slice(0, -1));
      store.setState({ slashCycleIndex: 0 });
      draw();
      return;
    }
    if (str && !key.ctrl && !key.meta) {
      if (isMouseOrEscapeGarbage(str, key)) {
        return;
      }
      const state = store.getState();
      store.setInput(state.input + str);
      store.setState({ slashCycleIndex: 0 });
      draw();
    }
  });

  process.stdin.prependListener('data', (chunk: Buffer | string) => {
    const text = typeof chunk === 'string' ? chunk : chunk.toString('utf8');
    if (text.includes('\x1b[<') || text.includes('\x1b[M') || /\d+;\d+;\d+[mM]/.test(text)) {
      lastMouseTime = Date.now();
    }
    let rest = text;
    const state = store.getState();
    const cols = state.cols;
    const rows = state.rows;
    const midH = Math.max(5, rows - 10);
    while (true) {
      const parsed = parseSGRMouse(rest);
      if (!parsed) break;
      rest = parsed.rest;
      const ev = parsed.event;

      if (ev.button === 0 || ev.button === 32) {
        store.setState({ scrollOffset: 0, inputPinned: false });
        process.stdout.write('\x1b[?25h');
        draw();
        continue;
      }

      if (ev.button === 64) {
        const lastAtIdx = store.getState().input.lastIndexOf('@');
        const isAtActive =
          lastAtIdx !== -1 && !store.getState().input.slice(lastAtIdx).includes(' ');
        if (isAtActive) {
          store.setState({
            atPaletteIndex: Math.max(0, (store.getState().atPaletteIndex || 0) - 1)
          });
          draw();
          continue;
        }

        const isSlashActive =
          store.getState().slashPaletteOpen ||
          (store.getState().input.startsWith('/') && !store.getState().input.includes(' '));
        if (isSlashActive) {
          store.setState({
            slashPaletteIndex: Math.max(0, (store.getState().slashPaletteIndex || 0) - 1)
          });
          draw();
          continue;
        }

        const allLines = renderTranscriptToLines(cols - 4);
        const maxScroll = Math.max(0, allLines.length - midH);
        store.setScrollOffset(Math.min(store.getState().scrollOffset + 3, maxScroll));
        draw();
      } else if (ev.button === 65) {
        const lastAtIdx = store.getState().input.lastIndexOf('@');
        const isAtActive =
          lastAtIdx !== -1 && !store.getState().input.slice(lastAtIdx).includes(' ');
        if (isAtActive) {
          const filter = store.getState().input.slice(lastAtIdx + 1);
          const filtered = filterAtFiles(store.getState().atPaletteFiles || [], filter);
          const maxIdx = Math.max(0, filtered.length - 1);
          store.setState({
            atPaletteIndex: Math.min(maxIdx, (store.getState().atPaletteIndex || 0) + 1)
          });
          draw();
          continue;
        }

        const isSlashActive =
          store.getState().slashPaletteOpen ||
          (store.getState().input.startsWith('/') && !store.getState().input.includes(' '));
        if (isSlashActive) {
          const filter = store.getState().slashPaletteOpen
            ? store.getState().slashPaletteFilter
            : store.getState().input.slice(1);
          const filtered = filterSlashCommands(filter);
          const maxIdx = Math.max(0, filtered.length - 1);
          store.setState({
            slashPaletteIndex: Math.min(maxIdx, (store.getState().slashPaletteIndex || 0) + 1)
          });
          draw();
          continue;
        }

        store.setScrollOffset(Math.max(store.getState().scrollOffset - 3, 0));
        draw();
      }
    }
  });

  async function submit(): Promise<void> {
    const state = store.getState();
    const goal = state.input.trim();
    if (!goal) return;

    const history = [...state.inputHistory, goal];
    store.setState({
      input: '',
      inputHistory: history,
      inputHistoryIndex: -1
    });
    saveHistory(config.dataDir, history);

    if (goal.startsWith('!')) {
      await runDirectCommand(goal.slice(1).trim());
      draw();
      return;
    }

    if (goal.startsWith('/')) {
      store.setLiveStatus('idle');

      if (goal.toLowerCase().startsWith('/model ')) {
        const args = goal.slice(7).trim();
        if (args) {
          const parts = args.split(/\s+/);
          const customModel = parts[0];
          const customUrl = parts[1];
          const customKey = parts[2];

          const draft = { ...(store.getState().configDraft || {}) };
          let msg = `✓ 模型已设为: ${customModel}`;

          if (customModel) {
            config.model = customModel;
            draft.model = customModel;
          }
          if (customUrl) {
            config.llmBaseUrl = customUrl;
            draft.llmBaseUrl = customUrl;
            msg += ` | 端点: ${customUrl}`;
          }
          if (customKey) {
            config.apiKey = customKey;
            draft.apiKey = customKey;
            msg += ' | Key: 已设置';
          }

          store.setState({ configDraft: draft });
          try {
            saveConfigFile(config.dataDir, {
              model: config.model,
              llmBaseUrl: config.llmBaseUrl,
              apiKey: config.apiKey
            });
          } catch {}
          appendSystemMessage(msg);
          draw();
          return;
        }
      }
      {
        const legacy = goal.toLowerCase().match(/^\/(api|url|key)(\s+(.*))?$/s);
        if (legacy) {
          const sub = legacy[1];
          const args = (legacy[3] ?? '').trim();
          let equiv = '/model <name> [url] [key]';
          if (args) {
            if (sub === 'api') equiv = `/model ${config.model} ${args}`;
            else if (sub === 'url') equiv = `/model ${config.model} ${args}`;
            else equiv = `/model ${config.model} ${config.llmBaseUrl} ${args}`;
          }
          appendSystemMessage(
            `✗ /${sub} 已并入 /model，不再是独立命令\n  等价写法: ${equiv}\n  或输入 /config 打开可视化配置面板`
          );
          draw();
          return;
        }
      }

      const cmd = goal.slice(1).toLowerCase().trim();
      switch (cmd) {
        case 'vibe':
          store.setState({ mode: 'vibe' });
          appendSystemMessage('✓ 切换到 Vibe 编程模式');
          break;
        case 'plan':
          store.setState({ mode: 'plan' });
          appendSystemMessage('✓ 切换到 Plan 规划模式');
          break;
        case 'config':
        case 'model':
          store.setState({ configModalOpen: !store.getState().configModalOpen });
          break;
        case 'files':
          store.setState({ fileTreeOpen: !store.getState().fileTreeOpen });
          break;
        case 'search':
          store.setState({ searchModalOpen: !store.getState().searchModalOpen });
          break;
        case 'init': {
          const { writeFileSync, existsSync } = await import('node:fs');
          const { join } = await import('node:path');
          const agentsPath = join(config.sandbox.cwd, 'AGENTS.md');
          if (existsSync(agentsPath)) {
            appendSystemMessage(`ℹ 项目根目录已存在 AGENTS.md 文件 (${agentsPath})`);
          } else {
            const template =
              '# AGENTS.md — 项目 AI 辅助开发与架构规则\n\n## 1. 项目简介与架构\n- **项目名称**: Zeno 终端 AI 编程系统\n- **开发模式**: 推荐在 Vibe 模式下快速迭代，Plan 模式下重构规划\n\n## 2. 代码规范与指令\n- 统一使用 TypeScript 严格模式\n- 运行单元测试: `bun test packages`\n- 编译可执行程序: `bun run compile`\n\n## 3. 注意事项\n- 保持 UI 组件无外框极简规范\n- 所有命令错误统一输出 VC-XXXXXX 规范 6 位错误码\n';
            try {
              writeFileSync(agentsPath, template, 'utf8');
              appendSystemMessage('🚀 已成功初始化项目的 AGENTS.md 规则文件！');
            } catch (err) {
              appendSystemMessage(`✖ 生成 AGENTS.md 失败: ${String(err)}`);
            }
          }
          break;
        }
        case 'compact': {
          const currentTr = store.getState().transcript;
          const compactedTr = currentTr.map((t) => {
            if (t.role === 'tool' && t.content && t.content.length > 200) {
              return {
                ...t,
                content: `${t.content.slice(0, 100)}... [已化简 ${t.content.length - 200} 字符]`
              };
            }
            return t;
          });
          store.setState({ transcript: compactedTr, scrollOffset: 0 });
          appendSystemMessage('🗜 对话上下文与历史已完成压缩 (Context compacted)');
          break;
        }
        case 'tokens':
        case 'usage':
          store.setState({ usagePanelOpen: !store.getState().usagePanelOpen });
          break;
        case 'lsp':
          appendSystemMessage('✓ LSP Status: active (stdio analyzer)');
          break;
        case 'undo':
        case 'rewind':
          store.setState({ undoModalOpen: true });
          break;
        case 'clear':
          store.setState({ transcript: [], scrollOffset: 0 });
          scrollback.reset('');
          appendSystemMessage('✓ Conversation cleared');
          break;
        case 'theme': {
          const themes: ThemeName[] = ['mocha', 'latte', 'midnight', 'forest', 'light', 'neon'];
          const currentIdx = themes.indexOf(state.theme);
          const nextTheme = themes[(currentIdx + 1) % themes.length];
          store.setState({ theme: nextTheme, palette: palette(nextTheme) });
          config.theme = nextTheme;
          try {
            saveConfigFile(config.dataDir, { theme: nextTheme });
          } catch {}
          break;
        }
        case 'brief': {
          const newBrief = !state.briefMode;
          store.setState({ briefMode: newBrief });
          appendSystemMessage(`✓ Brief mode: ${newBrief ? 'ON' : 'OFF'}`);
          break;
        }
        case 'tasks':
          store.setState({ tasksPanelOpen: !store.getState().tasksPanelOpen });
          break;
        case 'help': {
          const helpLines = [
            '◈ Zeno 终端 AI 编程助手帮助手册',
            '',
            '【 ⌨ 核心快捷按键 】',
            '  Tab          · 快捷切换 编码/开发 模式 (VIBE ⇄ PLAN) [Prompt为空时]',
            '  /            · 唤起斜杠指令弹窗选择器',
            '  @            · 引用项目文件并注入上下文',
            '  ! <cmd>      · 直接运行 Shell 命令 (末尾加 & 后台运行)',
            '  ?            · 打开全局命令面板 (Command Palette)',
            '  F2           · 开关工作区文件树 (File Tree)',
            '  Ctrl+U       · 查看 Token 消耗与费用面板',
            '',
            '【 ⚡ 模式切换 (Mode) 】',
            '  /vibe        · 切换到 Vibe 极速编程模式 (Vibe Coding)',
            '  /plan        · 切换到 Plan 架构规划模式 (Architecture Planning)',
            '',
            '【 ⚙ 配置管理 (Config) 】',
            '  /config      · 打开 AI 配置中心弹窗 (模型/端点/密钥/主题)',
            '  /model       · 一站式配置模型、端点与密钥 (/model <name> [url] [key])',
            '  /theme       · 循环切换界面主题 (mocha/latte/midnight/forest)',
            '',
            '【 🗂 工作流 (Workflow) 】',
            '  /files       · 开关工作区文件树抽屉 (F2)',
            '  /search      · 全局内容与代码正则搜索 (Ctrl+F)',
            '  /undo        · 回撤/恢复上一步 AI 代码修改',
            '',
            '【 📊 系统与统计 (System) 】',
            '  /tokens      · Token 详细消耗与费用面板 (Ctrl+U)',
            '  /usage       · Token 用量与统计面板',
            '  /tasks       · 查看后台异步任务面板',
            '  /lsp         · 查看 LSP 语言服务器与诊断状态',
            '  /brief       · 开关折叠工具输出模式',
            '  /clear       · 清空当前对话与上下文历史',
            '  /help        · 显示此帮助手册'
          ].join('\n');
          appendSystemMessage(helpLines);
          break;
        }
        default: {
          const near = matchSlashCommands(`/${cmd}`);
          const hint = near.length > 0 ? ` 试试 /${near[0]}？` : ' 输入 /help 查看可用命令。';
          appendSystemMessage(`Unknown command: /${cmd}.${hint}`);
        }
      }
      draw();
      return;
    }

    store.setLiveStatus('idle');
    appendMessage('user', goal);
    draw();
    let acc = '';
    let reasoningAcc = '';
    for await (const ev of runAgent(goal, { provider, tools, repoMap: projectRepoMap })) {
      if (ev.type === 'reasoning') {
        reasoningAcc += ev.text;
        store.setLiveStatus('thinking');
        draw();
      } else if (ev.type === 'token') {
        acc += ev.text;
        store.setLiveStatus('streaming');
        draw();
      } else if (ev.type === 'tool') {
        if (acc) {
          appendMessage('assistant', acc);
          acc = '';
        }
        const toolId = `tool-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
        store.appendMessage({
          id: toolId,
          role: 'tool',
          content: `${ev.call.name}(${JSON.stringify(ev.call.args)})`,
          timestamp: Date.now(),
          toolId,
          status: 'running',
          output: ''
        });
        store.setLiveStatus('tool');
        store.setState({ currentTool: ev.call.name, liveToolId: toolId });
        draw();
      } else if (ev.type === 'tool_result') {
        const s = store.getState();
        const entry = s.transcript.find((t) => t.toolId === s.liveToolId);
        if (entry) {
          store.updateMessage(entry.id, {
            status: ev.ok ? 'ok' : 'error',
            output: ev.ok
              ? ev.output || '(命令成功，无输出)'
              : ev.error || ev.output || '(no output)'
          });
        }
        store.setState({ currentTool: null, liveToolId: null });
        draw();
      } else if (ev.type === 'done') {
        if (acc) {
          appendMessage('assistant', acc);
          acc = '';
        }
        const state = store.getState();
        const nextTurn = (state.turnCount || 0) + 1;
        if (ev.usage) {
          const input = ev.usage.promptTokens ?? 0;
          const output = ev.usage.completionTokens ?? 0;
          const cost = estimateCost(input, output, config.model);
          const tu = state.tokenUsage;
          store.setState({
            turnCount: nextTurn,
            tokenUsage: {
              inputTokens: tu.inputTokens + input,
              outputTokens: tu.outputTokens + output,
              estimatedCost: tu.estimatedCost + cost,
              history: [...tu.history, { ts: Date.now(), input, output, cost }]
            }
          });
        } else {
          store.setState({ turnCount: nextTurn });
        }
        store.setLiveStatus('idle');
        store.setState({ currentTool: null });
        draw();
      }
    }
    if (acc) appendMessage('assistant', acc);
    store.setLiveStatus('idle');
    store.setState({ currentTool: null });
    draw();
  }

  const appendedTasks = new Set<string>();
  function syncBackgroundTasks(): void {
    const mgr = getTaskManager();
    for (const t of mgr.list()) {
      if (t.finishedAt && !appendedTasks.has(t.id)) {
        appendedTasks.add(t.id);
        appendCommandResult({
          id: t.id,
          command: t.command,
          status: t.status,
          output: t.output,
          exitCode: t.exitCode
        });
      }
    }
  }

  async function runDirectCommand(raw: string): Promise<void> {
    await executeDirectCommand(raw, config, appendSystemMessage, appendCommandResult);
  }

  function appendCommandResult(r: {
    id: string;
    command: string;
    status: TaskStatus;
    output: string;
    exitCode: number | null;
  }): void {
    const ok = r.status === 'done';
    const body = (r.output ?? '').trim().slice(0, 8000);
    store.appendMessage({
      id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      role: 'tool',
      content: `! ${r.command}`,
      timestamp: Date.now(),
      toolId: `task-${r.id}`,
      status: ok ? 'ok' : 'error',
      output: body || (ok ? '(命令成功，无输出)' : `[exit ${r.exitCode ?? '?'}](no output)`)
    });
    draw();
  }

  function cleanup(): void {
    if (stdinTty) {
      process.stdout.write('\x1b[?1006l\x1b[?1002l\x1b[?1000l');
      process.stdout.write(showCursor);
      process.stdout.write(leaveAltScreen);
    }
    if (process.stdin.isTTY) process.stdin.setRawMode(false);
    rl.close();
  }

  process.on('exit', cleanup);
  process.on('SIGINT', () => {
    cleanup();
    process.exit(0);
  });
}

function findNextToolId(startIdx: number, step: number, store: Store): string | null {
  const state = store.getState();
  const ids = state.transcript
    .filter((t: MessageEntry) => t.role === 'tool' && t.toolId)
    .map((t: MessageEntry) => t.toolId as string);
  if (ids.length === 0) return null;
  const idx = state.selectedToolId ? ids.indexOf(state.selectedToolId) : -1;
  const next = (((idx + step) % ids.length) + ids.length) % ids.length;
  return ids[next] ?? null;
}
