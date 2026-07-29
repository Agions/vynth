
import type { Store } from './Store';
import type { MessageEntry, TuiState } from './TuiState';

export type Action =
  | { type: 'SUBMIT'; input: string }
  | { type: 'APPEND_MESSAGE'; entry: MessageEntry }
  | { type: 'SCROLL'; delta: number }
  | { type: 'SCROLL_TO_BOTTOM' }
  | { type: 'TOGGLE_TOOL'; toolId: string }
  | { type: 'SELECT_TOOL'; toolId: string | null }
  | { type: 'SET_INPUT'; input: string }
  | { type: 'SET_STATUS'; status: TuiState['liveStatus'] }
  | { type: 'SET_THEME'; theme: string }
  | { type: 'SET_CONNECTION'; status: TuiState['connectionStatus'] }
  | { type: 'START_SEARCH'; query: string }
  | { type: 'NEXT_SEARCH_RESULT' }
  | { type: 'PREV_SEARCH_RESULT' }
  | { type: 'RESIZE'; cols: number; rows: number };

export function dispatch(store: Store, action: Action): void {
  switch (action.type) {
    case 'SUBMIT':
      store.setState({ input: '' });
      store.markDirty('input');
      store.markDirty('cursor');
      break;
    case 'APPEND_MESSAGE':
      store.appendMessage(action.entry);
      break;
    case 'SCROLL': {
      const state = store.getState();
      const newOffset = Math.max(0, state.scrollOffset + action.delta);
      store.setScrollOffset(newOffset);
      break;
    }
    case 'SCROLL_TO_BOTTOM':
      store.setScrollOffset(0);
      break;
    case 'TOGGLE_TOOL':
      store.toggleToolCollapse(action.toolId);
      break;
    case 'SELECT_TOOL':
      store.setState({ selectedToolId: action.toolId });
      store.markDirty('mid');
      break;
    case 'SET_INPUT':
      store.setInput(action.input);
      break;
    case 'SET_STATUS':
      store.setLiveStatus(action.status);
      break;
    case 'SET_THEME':
      // Theme change requires palette update - caller should provide full state
      break;
    case 'SET_CONNECTION':
      store.setConnectionStatus(action.status);
      break;
    case 'START_SEARCH':
      store.setState({
        searchQuery: action.query,
        searchResults: [],
        currentSearchIndex: -1
      });
      store.markDirty('mid');
      break;
    case 'NEXT_SEARCH_RESULT':
    case 'PREV_SEARCH_RESULT':
      // Search navigation - implementation depends on search results
      store.markDirty('mid');
      break;
    case 'RESIZE':
      store.setState({ cols: action.cols, rows: action.rows });
      store.markAllDirty();
      break;
  }
}
