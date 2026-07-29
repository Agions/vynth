import { appendFileSync, existsSync, mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

export type AuditKind =
  | 'tool_exec'
  | 'file_access'
  | 'network_egress'
  | 'config_change'
  | 'plugin_load';

export interface AuditRecord {
  ts: string; // ISO 8601
  kind: AuditKind;
  ok: boolean;
  detail: Record<string, unknown>;
}

export class AuditLog {
  private readonly file: string;
  private readonly enabledFlag: boolean;

  constructor(dataDir: string, enabled: boolean) {
    this.enabledFlag = enabled;
    this.file = join(dataDir, 'audit.log');
  }

  get enabled(): boolean {
    return this.enabledFlag;
  }

  record(kind: AuditKind, detail: Record<string, unknown>, ok = true): void {
    if (!this.enabledFlag) return;
    try {
      const dir = this.file.slice(0, this.file.lastIndexOf('/'));
      if (dir && !existsSync(dir)) mkdirSync(dir, { recursive: true });
      const rec: AuditRecord = { ts: new Date().toISOString(), kind, ok, detail };
      appendFileSync(this.file, `${JSON.stringify(rec)}\n`, 'utf8');
    } catch {}
  }

  readAllSync(): AuditRecord[] {
    if (!existsSync(this.file)) return [];
    const text = readFileSync(this.file, 'utf8');
    return text
      .split('\n')
      .filter((l) => l.trim().length > 0)
      .map((l) => JSON.parse(l) as AuditRecord);
  }
}

const NOOP = new AuditLog(process.cwd(), false);
let active: AuditLog = NOOP;

export function initAudit(config: { dataDir: string; audit: boolean }): AuditLog {
  active = new AuditLog(config.dataDir, config.audit);
  return active;
}

export function audit(): AuditLog {
  return active;
}

export function resetAudit(): void {
  active = NOOP;
}
