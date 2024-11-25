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
}

export interface ShellExitStatus {
  exitCode: number;
  stdout: string;
  stderr: string;
}

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
 * @param {string} command - The command to execute.
 * @param {string | string[]} args - Array of command arguments.
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
    onStdout: callback =>
      command.stdout.on('data', data => callback(data.toString())),
    onStderr: callback =>
      command.stderr.on('data', data => callback(data.toString())),
    onExit: callback =>
      command.on('close', status =>
        callback({
          exitCode: status.code ?? 0,
        }),
      ),
  };
}

/**
 * Creates a Tauri command via its shell plugin.
 */
function createCommand(
  program: string,
  args?: string | string[],
  options?: ShellCommandOptions,
): Command<Uint8Array | string> {
  // Convert encoding option to Tauri's format.
  const tauriOptions = {
    ...options,
    env: options?.env ?? undefined,
    encoding: options?.encoding === 'raw' ? 'raw' : undefined,
  };

  return Command.create(program, args, tauriOptions);
}
