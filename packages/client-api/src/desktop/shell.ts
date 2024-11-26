import { Command } from '@tauri-apps/plugin-shell';

export interface ShellSpawnOptions {
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
   * - `text` (default): stdout/stderr are returned as a UTF-8 encoded
   * `string`.
   * - `raw`: stdout/stderr are returned as raw bytes (`Uint8Array`).
   */
  encoding?: 'text' | 'raw';
}

export interface ShellProcess<
  TOutput extends string | Uint8Array = string,
> {
  processId: number;
  onStdout: (callback: (line: TOutput) => void) => void;
  onStderr: (callback: (line: TOutput) => void) => void;
  onExit: (
    callback: (status: Omit<ShellExitStatus, 'stdout' | 'stderr'>) => void,
  ) => void;
  kill: () => void;
  write: (data: string | Uint8Array) => void;
}

export interface ShellExitStatus<
  TOutput extends string | Uint8Array = string,
> {
  exitCode: number;
  stdout: TOutput;
  stderr: TOutput;
}

/**
 * Executes a shell command and waits for completion.
 *
 * @param {string} command - Path to program executable, or program name
 * (if in $PATH).
 * @param {string | string[]} args - Arguments to pass to the program.
 * @param {Object} options - Spawn options (optional).
 * @throws - If the command fails to execute.
 */
export async function shellExec<
  TOutput extends string | Uint8Array = string,
>(
  program: string,
  args?: string | string[],
  options?: ShellSpawnOptions,
): Promise<ShellExitStatus<TOutput>> {
  const output = await createCommand<TOutput>(
    program,
    args,
    options,
  ).execute();

  return {
    exitCode: output.code ?? 0,
    stdout: output.stdout,
    stderr: output.stderr,
  };
}

/**
 * Starts a shell command without waiting for completion. Allows for
 * interaction with the spawned process, such as sending input and killing
 * the process.
 *
 * @param {string} command - Path to program executable, or program name
 * (if in $PATH).
 * @param {string | string[]} args - Arguments to pass to the program.
 * @param {Object} options - Spawn options (optional).
 */
export async function shellSpawn<
  TOutput extends string | Uint8Array = string,
>(
  program: string,
  args?: string | string[],
  options?: ShellSpawnOptions,
): Promise<ShellProcess<TOutput>> {
  const command = createCommand<TOutput>(program, args, options);
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
function createCommand<TOutput extends string | Uint8Array = string>(
  program: string,
  args?: string | string[],
  options?: ShellSpawnOptions,
): Command<TOutput> {
  return (Command as any).create(program, args, {
    ...options,
    // Tauri's `SpawnOptions` type is not explicit about allowing `env` to
    // be `null`.
    env: options?.env ?? undefined,
  });
}
