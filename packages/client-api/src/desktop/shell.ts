import { Command } from '@tauri-apps/plugin-shell';

export interface ShellCommand {
  onStdout: (callback: (stdout: string) => void) => this;
  onStderr: (callback: (stderr: string) => void) => this;
  execute: () => Promise<ShellResult>;
  start: () => Promise<ShellProcess>;
}

export interface ShellResult {
  exitCode: number;
  stdout: string[];
  stderr: string[];
}

export interface ShellProcess {
  processId: number;
  stdout: string[];
  stderr: string[];
  clearStdout: () => void;
  clearStderr: () => void;
}

export interface ShellCommandOptions {
  /**
   * Current working directory.
   */
  cwd?: string;

  /**
   * Environment variables.
   *
   * Set to `null` to clear the process env and prevent inheriting the
   * parent env.
   */
  env?: Record<string, string> | null;

  /**
   * Character encoding for stdout/stderr.
   *
   * - `text` (default): stdout/stderr are returned as UTF-8 encoded strings.
   * - `raw`: stdout/stderr are returned as raw bytes (`Uint8Array`).
   */
  encoding?: 'text' | 'raw';
}

export function shell(
  program: string,
  args?: string | string[],
  options?: ShellCommandOptions,
): ShellCommand {
  const command = Command.create(program, args, options as any);

  return {
    onStdout: callback => {
      command.stdout.on('data', callback);
      return this;
    },
    onStderr: callback => {
      command.stderr.on('data', callback);
      return this;
    },
  };
}
