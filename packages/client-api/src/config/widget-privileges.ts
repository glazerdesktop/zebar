export type WidgetPrivileges = {
  shellCommands: AllowedShellCommand[];
};

export type AllowedShellCommand = {
  program: string;
  argsRegex: string;
};
