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

export class ShellCommandImpl {
  private readonly command: Command<string>;
  private readonly stdoutLines: string[] = [];
  private readonly stderrLines: string[] = [];
  private stdoutCallback?: (stdout: string) => void;
  private stderrCallback?: (stderr: string) => void;

  constructor(
    program: string,
    args?: string | string[],
    options?: ShellCommandOptions,
  ) {
    // Convert encoding option to Tauri's format
    const tauriOptions = {
      ...options,
      env: options?.env ?? undefined,
      encoding: options?.encoding === 'raw' ? 'raw' : undefined,
    };

    this.command = Command.create(program, args, tauriOptions);

    // Set up stdout listener
    this.command.stdout.on('data', (line: string) => {
      this.stdoutLines.push(line);
      if (this.stdoutCallback) {
        this.stdoutCallback(line);
      }
    });

    // Set up stderr listener
    this.command.stderr.on('data', (line: string) => {
      this.stderrLines.push(line);
      if (this.stderrCallback) {
        this.stderrCallback(line);
      }
    });
  }

  onStdout(callback: (stdout: string) => void): this {
    this.stdoutCallback = callback;
    return this;
  }

  onStderr(callback: (stderr: string) => void): this {
    this.stderrCallback = callback;
    return this;
  }

  async execute(): Promise<ShellResult> {
    const result = await this.command.execute();
    return {
      exitCode: result.code ?? 0,
      stdout: this.stdoutLines,
      stderr: this.stderrLines,
    };
  }

  async start(): Promise<ShellProcess> {
    const child = await this.command.spawn();
    const stdoutLines = this.stdoutLines;
    const stderrLines = this.stderrLines;

    return {
      processId: child.pid,
      get stdout() {
        return [...stdoutLines];
      },
      get stderr() {
        return [...stderrLines];
      },
      clearStdout: () => {
        this.stdoutLines.length = 0;
      },
      clearStderr: () => {
        this.stderrLines.length = 0;
      },
    };
  }
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
  // ) {
  // const command = Command.create(program, args, options as any);
  return new ShellCommandImpl(program, args, options);
}
