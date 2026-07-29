import { spawn, type ChildProcess } from 'node:child_process';

export interface LspDiagnostic {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  severity?: number;
  message: string;
}

export class LspClient {
  private process: ChildProcess | null = null;
  private buffer = '';
  private nextId = 1;
  private pendingRequests = new Map<number, (res: any) => void>();
  private diagnosticsListeners: Array<(uri: string, diagnostics: LspDiagnostic[]) => void> = [];

  constructor(private command: string, private args: string[] = []) {}

  public async start(rootUri: string): Promise<boolean> {
    try {
      this.process = spawn(this.command, this.args, { stdio: ['pipe', 'pipe', 'ignore'] });
      if (!this.process.stdout || !this.process.stdin) return false;

      this.process.stdout.on('data', (chunk: Buffer) => {
        this.buffer += chunk.toString('utf8');
        this.processBuffer();
      });

      this.process.on('error', () => {
        this.process = null;
      });

      // Send initialize
      const res = await this.sendRequest('initialize', {
        processId: process.pid,
        rootUri,
        capabilities: {}
      });
      if (res) {
        this.sendNotification('initialized', {});
        return true;
      }
      return false;
    } catch {
      return false;
    }
  }

  public notifyDidOpen(uri: string, languageId: string, version: number, text: string): void {
    this.sendNotification('textDocument/didOpen', {
      textDocument: { uri, languageId, version, text }
    });
  }

  public async getHover(uri: string, line: number, character: number): Promise<string | null> {
    try {
      const res = await this.sendRequest('textDocument/hover', {
        textDocument: { uri },
        position: { line, character }
      });
      if (res && res.contents) {
        if (typeof res.contents === 'string') return res.contents;
        if (res.contents.value) return res.contents.value;
      }
      return null;
    } catch {
      return null;
    }
  }

  public onDiagnostics(listener: (uri: string, diagnostics: LspDiagnostic[]) => void): void {
    this.diagnosticsListeners.push(listener);
  }

  public stop(): void {
    if (this.process) {
      try {
        this.process.kill();
      } catch {}
      this.process = null;
    }
  }

  private sendRequest(method: string, params: any): Promise<any> {
    return new Promise((resolve) => {
      if (!this.process || !this.process.stdin) return resolve(null);
      const id = this.nextId++;
      this.pendingRequests.set(id, resolve);
      const msg = JSON.stringify({ jsonrpc: '2.0', id, method, params });
      const header = `Content-Length: ${Buffer.byteLength(msg, 'utf8')}\r\n\r\n`;
      this.process.stdin.write(header + msg);
    });
  }

  private sendNotification(method: string, params: any): void {
    if (!this.process || !this.process.stdin) return;
    const msg = JSON.stringify({ jsonrpc: '2.0', method, params });
    const header = `Content-Length: ${Buffer.byteLength(msg, 'utf8')}\r\n\r\n`;
    this.process.stdin.write(header + msg);
  }

  private processBuffer(): void {
    while (true) {
      const match = this.buffer.match(/Content-Length: (\d+)\r\n\r\n/);
      if (!match) break;
      const contentLength = parseInt(match[1], 10);
      const headerLength = match[0].length;
      if (this.buffer.length < headerLength + contentLength) break;

      const bodyStr = this.buffer.slice(headerLength, headerLength + contentLength);
      this.buffer = this.buffer.slice(headerLength + contentLength);

      try {
        const body = JSON.parse(bodyStr);
        if (body.id !== undefined && this.pendingRequests.has(body.id)) {
          const resolver = this.pendingRequests.get(body.id)!;
          this.pendingRequests.delete(body.id);
          resolver(body.result);
        } else if (body.method === 'textDocument/publishDiagnostics') {
          for (const l of this.diagnosticsListeners) {
            l(body.params.uri, body.params.diagnostics || []);
          }
        }
      } catch {}
    }
  }
}
