export interface ProgressBarProps {
  progress: number; // 0-1
  label?: string;
  width?: number;
}

export function ProgressBar(props: ProgressBarProps): string {
  const { progress, label, width = 20 } = props;
  const filled = Math.round(width * Math.max(0, Math.min(1, progress)));
  const bar = `[${'█'.repeat(filled)}${'░'.repeat(width - filled)}]`;
  const percent = `${Math.round(progress * 100)}%`;
  const labelStr = label ? `${label} ` : '';
  return `${labelStr}${bar} ${percent}`;
}
