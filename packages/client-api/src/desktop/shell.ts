import { Command } from '@tauri-apps/plugin-shell';

export interface ShellCommandOptions {
  /**
   * Current working directory.
   */
  cwd?: string;

  /**
   * Environment variables.
   *
   * Set to `null` to clear the process env and prevent inheritance from
   * the parent process.
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

export interface ShellProcess {
  processId: number;
  onStdout: (callback: (line: string) => void) => void;
  onStderr: (callback: (line: string) => void) => void;
  onExit: (
    callback: (status: Omit<ShellExitStatus, 'stdout' | 'stderr'>) => void,
  ) => void;
  kill: () => void;
  write: (data: string | Uint8Array) => void;
}

export interface ShellExitStatus {
  exitCode: number;
  stdout: string;
  stderr: string;
}

/**
 * Executes a shell command and waits for completion
 *
 * @param {string} command - Path to program executable, or program name
 * (if in $PATH).
 * @param {string | string[]} args - Arguments to pass to the program.
 * @param {Object} options - Spawn options (optional).
 */
export async function shellExec(
  program: string,
  args?: string | string[],
  options?: ShellCommandOptions,
): Promise<ShellExitStatus> {
  const output = await createCommand(program, args, options).execute();

  return {
    exitCode: output.code ?? 0,
    stdout: output.stdout,
    stderr: output.stderr,
  };
}

/**
 * Starts a shell command without waiting for completion.
 *
 * @param {string} command - Path to program executable, or program name
 * (if in $PATH).
 * @param {string | string[]} args - Arguments to pass to the program.
 * @param {Object} options - Spawn options (optional).
 */
export async function shellSpawn(
  program: string,
  args?: string | string[],
  options?: ShellCommandOptions,
): Promise<ShellProcess> {
  const command = createCommand(program, args, options);
  const process = await command.spawn();

  return {
    processId: process.pid,
    onStdout: callback => command.stdout.on('data', callback),
    onStderr: callback => command.stderr.on('data', callback),
    onExit: callback =>
      command.on('close', status =>
        callback({ exitCode: status.code ?? 0 }),
      ),
    kill: () => process.kill(),
    write: data => process.write(data),
  };
}

/**
 * Creates a shell command via Tauri's shell plugin.
 */
function createCommand(
  program: string,
  args?: string | string[],
  options?: ShellCommandOptions,
): Command<Uint8Array | string> {
  return Command.create(program, args, {
    ...options,
    // Tauri's `SpawnOptions` type is not explicit about allowing `env` to
    // be `null`.
    env: options?.env ?? undefined,
  });
}
