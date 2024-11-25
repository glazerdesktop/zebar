import { Command } from '@tauri-apps/plugin-shell';

export interface ShellCommand {
  onStdout: (callback: (stdout: string) => void) => this;
  onStderr: (callback: (stderr: string) => void) => this;
  onExit: (callback: (status: ExitStatus) => void) => this;
  execute: () => Promise<ExitStatus>;
  start: () => Promise<ShellProcess>;
}

export interface ExitStatus {
  exitCode: number;
  stdout: readonly string[];
  stderr: readonly string[];
}

export interface ShellProcess {
  readonly processId: number;
  readonly stdout: string[];
  readonly stderr: string[];
  clearStdout: () => void;
  clearStderr: () => void;
}

const process = await shell('long-running-process', [], {
  onStdout: line => console.log('stdout:', line),
  onClose: result => console.log('Process finished:', result),
}).start();

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
  const stdoutLines: string[] = [];
  const stderrLines: string[] = [];
  let stdoutCallback: ((stdout: string) => void) | undefined;
  let stderrCallback: ((stderr: string) => void) | undefined;

  // Convert encoding option to Tauri's format.
  const tauriOptions = {
    ...options,
    env: options?.env ?? undefined,
    encoding: options?.encoding === 'raw' ? 'raw' : undefined,
  };

  const command = Command.create(program, args, tauriOptions);

  // Set up stdout listener
  command.stdout.on('data', (line: string) => {
    stdoutLines.push(line);
    if (stdoutCallback) {
      stdoutCallback(line);
    }
  });

  // Set up stderr listener
  command.stderr.on('data', (line: string) => {
    stderrLines.push(line);
    if (stderrCallback) {
      stderrCallback(line);
    }
  });

  const shellCommand: ShellCommand = {
    onStdout(callback: (stdout: string) => void) {
      stdoutCallback = callback;
      return shellCommand;
    },

    onStderr(callback: (stderr: string) => void) {
      stderrCallback = callback;
      return shellCommand;
    },

    async execute(): Promise<ExitStatus> {
      const child = await command.spawn();

      return new Promise((resolve, reject) => {
        command.on('error', reject);
        command.on('close', ({ code }) => {
          resolve({
            exitCode: code ?? 0,
            stdout: [...stdoutLines],
            stderr: [...stderrLines],
          });
        });
      });
    },

    async start(): Promise<ShellProcess> {
      const child = await command.spawn();

      return {
        processId: child.pid,
        get stdout() {
          return [...stdoutLines];
        },
        get stderr() {
          return [...stderrLines];
        },
        clearStdout: () => {
          stdoutLines.length = 0;
        },
        clearStderr: () => {
          stderrLines.length = 0;
        },
      };
    },
  };

  return shellCommand;
}
