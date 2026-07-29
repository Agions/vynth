
import type { ConnectionStatus, LiveStatus, TuiState } from './TuiState';

export interface DirtyFlags {
  top: boolean;
  mid: boolean;
  bot: boolean;
  input: boolean;
  cursor: boolean;
}

export type Listener = (state: TuiState) => void;

export class Store {
  private state: TuiState;
  private listeners = new Set<Listener>();
  public dirtyFlags: DirtyFlags = {
    top: true,
    mid: true,
    bot: true,
    input: true,
    cursor: true
  };

  constructor(initial: TuiState) {
    this.state = initial;
  }

  getState(): TuiState {
    return this.state;
  }

  setState(partial: Partial<TuiState>): void {
    const prev = this.state;
    this.state = { ...this.state, ...partial };
    this.notify(prev);
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  markDirty(area: keyof DirtyFlags): void {
    this.dirtyFlags[area] = true;
  }

  clearDirty(area: keyof DirtyFlags): void {
    this.dirtyFlags[area] = false;
  }

  markAllDirty(): void {
    Object.keys(this.dirtyFlags).forEach((k) => {
      this.dirtyFlags[k as keyof DirtyFlags] = true;
    });
  }

  // Convenience actions
  setLiveStatus(status: LiveStatus): void {
    this.setState({ liveStatus: status });
    this.markDirty('bot');
    this.markDirty('top');
  }

  setConnectionStatus(status: ConnectionStatus): void {
    this.setState({ connectionStatus: status });
    this.markDirty('top');
  }

  setInput(input: string): void {
    let cleanInput = input;
    if (/[<;]?\d+;\d+;\d+[mM]?/.test(cleanInput) || /;\d+[mM]/.test(cleanInput)) {
      cleanInput = cleanInput.replace(/[<;]?\d+;\d+;\d+[mM]?/g, '').replace(/;\d+[mM]/g, '');
    }
    this.setState({ input: cleanInput });
    this.markDirty('input');
    this.markDirty('cursor');
  }

  appendMessage(entry: import('./TuiState').MessageEntry): void {
    this.setState({
      transcript: [...this.state.transcript, entry],
      scrollOffset: 0
    });
    this.markDirty('mid');
    this.markDirty('cursor');
  }

  updateMessage(id: string, patch: Partial<import('./TuiState').MessageEntry>): void {
    this.setState({
      transcript: this.state.transcript.map((t) => (t.id === id ? { ...t, ...patch } : t))
    });
    this.markDirty('mid');
    this.markDirty('cursor');
  }

  setScrollOffset(offset: number): void {
    this.setState({
      scrollOffset: offset,
      inputPinned: offset > 0
    });
    this.markDirty('mid');
  }

  toggleToolCollapse(toolId: string): void {
    const next = new Set(this.state.collapsedTools);
    if (next.has(toolId)) {
      next.delete(toolId);
    } else {
      next.add(toolId);
    }
    this.setState({ collapsedTools: next });
    this.markDirty('mid');
  }

  private notify(prev: TuiState): void {
    for (const listener of this.listeners) {
      listener(this.state);
    }
  }
}
