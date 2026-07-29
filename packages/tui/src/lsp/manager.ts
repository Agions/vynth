import { LspClient } from './client';

export class LspManager {
  private clients = new Map<string, LspClient>();
  private activeServerName: string | null = null;

  public async getOrStartClient(filePath: string, cwd: string): Promise<LspClient | null> {
    const ext = filePath.slice(filePath.lastIndexOf('.')).toLowerCase();
    let serverCmd: string | null = null;
    let serverArgs: string[] = [];

    if (ext === '.ts' || ext === '.js' || ext === '.tsx' || ext === '.jsx') {
      serverCmd = 'typescript-language-server';
      serverArgs = ['--stdio'];
    } else if (ext === '.py') {
      serverCmd = 'pyright-langserver';
      serverArgs = ['--stdio'];
    } else if (ext === '.go') {
      serverCmd = 'gopls';
    } else if (ext === '.rs') {
      serverCmd = 'rust-analyzer';
    }

    if (!serverCmd) return null;

    if (this.clients.has(serverCmd)) {
      const existing = this.clients.get(serverCmd);
      if (existing) return existing;
    }

    const client = new LspClient(serverCmd, serverArgs);
    const rootUri = `file://${cwd}`;
    const ok = await client.start(rootUri);
    if (ok) {
      this.clients.set(serverCmd, client);
      this.activeServerName = serverCmd;
      return client;
    }
    return null;
  }

  public getActiveServerName(): string {
    return this.activeServerName || 'none';
  }

  public stopAll(): void {
    for (const client of this.clients.values()) {
      client.stop();
    }
    this.clients.clear();
    this.activeServerName = null;
  }
}

let globalLspManager: LspManager | null = null;
export function getLspManager(): LspManager {
  if (!globalLspManager) {
    globalLspManager = new LspManager();
  }
  return globalLspManager;
}
