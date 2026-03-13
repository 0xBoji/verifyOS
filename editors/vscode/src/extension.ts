import * as childProcess from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { promisify } from "node:util";
import * as vscode from "vscode";
import * as lc from "vscode-languageclient/node";

const execFile = promisify(childProcess.execFile);

let client: lc.LanguageClient | undefined;
let outputChannel: vscode.OutputChannel | undefined;
let explorer: VerifyOSView | undefined;
let serverStatus: ServerStatus = {
  running: false,
  source: "unknown",
  command: "",
  profile: "basic",
};

type VerifyOSCommand = {
  command: string;
  args: string[];
  source: string;
  profile: string;
};

type ServerStatus = {
  running: boolean;
  source: string;
  command: string;
  profile: string;
  lastError?: string;
};

type VerifyOSItemKind = "status" | "meta" | "action";

class VerifyOSTreeItem extends vscode.TreeItem {
  constructor(
    public readonly kind: VerifyOSItemKind,
    label: string,
    description?: string,
    command?: vscode.Command,
    iconPath?: vscode.ThemeIcon,
  ) {
    super(label, vscode.TreeItemCollapsibleState.None);
    this.description = description;
    this.command = command;
    this.contextValue = kind;
    this.iconPath = iconPath;
  }
}

class VerifyOSView implements vscode.TreeDataProvider<VerifyOSTreeItem> {
  private readonly emitter = new vscode.EventEmitter<VerifyOSTreeItem | undefined | void>();

  readonly onDidChangeTreeData = this.emitter.event;

  refresh(): void {
    this.emitter.fire();
  }

  getTreeItem(element: VerifyOSTreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(): VerifyOSTreeItem[] {
    const statusLabel = serverStatus.running ? "Language server" : "Language server";
    const statusDetail = serverStatus.running ? "running" : "waiting";
    const sourceDetail = serverStatus.source === "bundled" ? "bundled binary" : "configured path";
    const profileDetail = `profile ${serverStatus.profile}`;

    const items = [
      new VerifyOSTreeItem(
        "status",
        statusLabel,
        `${statusDetail} · ${sourceDetail}`,
        undefined,
        new vscode.ThemeIcon(serverStatus.running ? "pass-filled" : "clock"),
      ),
      new VerifyOSTreeItem("meta", "Current profile", profileDetail, undefined, new vscode.ThemeIcon("settings-gear")),
      new VerifyOSTreeItem(
        "action",
        "Scan current bundle",
        "Run voc with the active file's bundle",
        {
          command: "verifyOS.scanCurrentBundle",
          title: "Scan current bundle",
        },
        new vscode.ThemeIcon("search"),
      ),
      new VerifyOSTreeItem(
        "action",
        "Generate handoff bundle",
        "Create AGENTS.md, agent-pack, and repair docs",
        {
          command: "verifyOS.generateHandoff",
          title: "Generate handoff bundle",
        },
        new vscode.ThemeIcon("package"),
      ),
      new VerifyOSTreeItem(
        "action",
        "Open Problems",
        "Show editor diagnostics",
        {
          command: "verifyOS.openProblems",
          title: "Open Problems",
        },
        new vscode.ThemeIcon("warning"),
      ),
      new VerifyOSTreeItem(
        "action",
        "Show Output",
        "Open the verifyOS log",
        {
          command: "verifyOS.showOutput",
          title: "Show Output",
        },
        new vscode.ThemeIcon("output"),
      ),
      new VerifyOSTreeItem(
        "action",
        "Restart language server",
        "Reload voc lsp",
        {
          command: "verifyOS.restartLanguageServer",
          title: "Restart language server",
        },
        new vscode.ThemeIcon("refresh"),
      ),
    ];

    if (serverStatus.lastError) {
      items.splice(
        1,
        0,
        new VerifyOSTreeItem(
          "meta",
          "Last startup issue",
          serverStatus.lastError,
          undefined,
          new vscode.ThemeIcon("error"),
        ),
      );
    }

    return items;
  }
}

function output(): vscode.OutputChannel {
  if (!outputChannel) {
    outputChannel = vscode.window.createOutputChannel("verifyOS");
  }
  return outputChannel;
}

function bundledBinaryName(): string | undefined {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "darwin" && arch === "arm64") {
    return path.join("bin", "darwin-arm64", "voc");
  }

  if (platform === "darwin" && arch === "x64") {
    return path.join("bin", "darwin-x64", "voc");
  }

  if (platform === "linux" && arch === "x64") {
    return path.join("bin", "linux-x64", "voc");
  }

  if (platform === "win32" && arch === "x64") {
    return path.join("bin", "win32-x64", "voc.exe");
  }

  return undefined;
}

function resolveBundledBinary(context: vscode.ExtensionContext): string | undefined {
  const relative = bundledBinaryName();
  if (!relative) {
    return undefined;
  }

  const absolute = context.asAbsolutePath(relative);
  return fs.existsSync(absolute) ? absolute : undefined;
}

function serverCommand(context: vscode.ExtensionContext): VerifyOSCommand {
  const config = vscode.workspace.getConfiguration("verifyOS");
  const profile = config.get<string>("profile", "basic");
  const useBundledBinary = config.get<boolean>("useBundledBinary", true);
  const bundledBinary = useBundledBinary ? resolveBundledBinary(context) : undefined;
  const command = bundledBinary ?? config.get<string>("path", "voc");

  return {
    command,
    args: ["lsp", "--profile", profile],
    source: bundledBinary ? "bundled" : "configured",
    profile,
  };
}

function updateServerStatus(command: VerifyOSCommand, running: boolean, lastError?: string): void {
  serverStatus = {
    running,
    source: command.source,
    command: command.command,
    profile: command.profile,
    lastError,
  };
  explorer?.refresh();
}

function supportedBundleFile(uri: vscode.Uri | undefined): vscode.Uri | undefined {
  if (!uri || uri.scheme !== "file") {
    return undefined;
  }

  const name = path.basename(uri.fsPath);
  if (name === "Info.plist" || name === "PrivacyInfo.xcprivacy") {
    return uri;
  }

  return undefined;
}

function activeBundleRoot(): string | undefined {
  const uri = supportedBundleFile(vscode.window.activeTextEditor?.document.uri);
  return uri ? path.dirname(uri.fsPath) : undefined;
}

function workspaceOutputDir(): string | undefined {
  const workspace = vscode.workspace.workspaceFolders?.[0];
  if (!workspace) {
    return undefined;
  }

  const configured = vscode.workspace
    .getConfiguration("verifyOS")
    .get<string>("outputDir", ".verifyos");
  return path.join(workspace.uri.fsPath, configured);
}

async function runVocCommand(
  context: vscode.ExtensionContext,
  title: string,
  args: string[],
): Promise<void> {
  const channel = output();
  const server = serverCommand(context);

  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title,
    },
    async () => {
      channel.appendLine(`Running ${server.command} ${args.join(" ")}`);
      try {
        const result = await execFile(server.command, args, {
          cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
        });
        if (result.stdout) {
          channel.appendLine(result.stdout.trim());
        }
        if (result.stderr) {
          channel.appendLine(result.stderr.trim());
        }
      } catch (error) {
        channel.appendLine(String(error));
        channel.show(true);
        throw error;
      }
    },
  );
}

async function scanCurrentBundle(context: vscode.ExtensionContext): Promise<void> {
  const bundleRoot = activeBundleRoot();
  if (!bundleRoot) {
    void vscode.window.showInformationMessage(
      "Open Info.plist or PrivacyInfo.xcprivacy inside an .app bundle first.",
    );
    return;
  }

  const profile = vscode.workspace.getConfiguration("verifyOS").get<string>("profile", "basic");
  await runVocCommand(context, "verifyOS: scanning current bundle", [
    "--app",
    bundleRoot,
    "--profile",
    profile,
  ]);
  await vscode.commands.executeCommand("workbench.actions.view.problems");
}

async function generateHandoff(context: vscode.ExtensionContext): Promise<void> {
  const bundleRoot = activeBundleRoot();
  const outputDir = workspaceOutputDir();
  if (!bundleRoot || !outputDir) {
    void vscode.window.showInformationMessage(
      "Open Info.plist or PrivacyInfo.xcprivacy in a workspace before generating a handoff bundle.",
    );
    return;
  }

  const profile = vscode.workspace.getConfiguration("verifyOS").get<string>("profile", "basic");
  await runVocCommand(context, "verifyOS: generating handoff bundle", [
    "handoff",
    "--output-dir",
    outputDir,
    "--from-scan",
    bundleRoot,
    "--profile",
    profile,
  ]);
  void vscode.window.showInformationMessage(`verifyOS handoff bundle updated in ${outputDir}`);
}

async function startClient(context: vscode.ExtensionContext): Promise<void> {
  if (client) {
    return;
  }

  const server = serverCommand(context);
  const channel = output();
  channel.appendLine(`Starting verifyOS language server via ${server.source} binary: ${server.command}`);
  const clientOptions: lc.LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", pattern: "**/Info.plist" },
      { scheme: "file", pattern: "**/*.plist" },
      { scheme: "file", pattern: "**/*.xcprivacy" },
    ],
    outputChannel: channel,
  };

  client = new lc.LanguageClient(
    "verifyOS",
    "verifyOS",
    {
      command: server.command,
      args: server.args,
    },
    clientOptions,
  );

  client.onDidChangeState(({ newState }) => {
    if (newState === lc.State.Stopped) {
      channel.appendLine("verifyOS language server stopped.");
      updateServerStatus(server, false, serverStatus.lastError);
    }
  });

  try {
    await client.start();
    updateServerStatus(server, true);
    context.subscriptions.push(channel);
  } catch (error) {
    client = undefined;
    const message = String(error);
    channel.appendLine(message);
    updateServerStatus(server, false, message);
    const selection = "Install verifyOS-cli";
    const picked = await vscode.window.showErrorMessage(
      "verifyOS could not start `voc lsp`. Open the verifyOS sidebar or Output panel for details.",
      selection,
    );
    if (picked === selection) {
      void vscode.env.openExternal(vscode.Uri.parse("https://crates.io/crates/verifyos-cli"));
    }
  }
}

async function restartClient(context: vscode.ExtensionContext): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
  await startClient(context);
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  explorer = new VerifyOSView();
  const treeView = vscode.window.createTreeView("verifyOS.explorer", {
    treeDataProvider: explorer,
  });

  context.subscriptions.push(
    treeView,
    vscode.commands.registerCommand("verifyOS.scanCurrentBundle", async () => {
      await scanCurrentBundle(context);
    }),
    vscode.commands.registerCommand("verifyOS.generateHandoff", async () => {
      await generateHandoff(context);
    }),
    vscode.commands.registerCommand("verifyOS.openProblems", async () => {
      await vscode.commands.executeCommand("workbench.actions.view.problems");
    }),
    vscode.commands.registerCommand("verifyOS.restartLanguageServer", async () => {
      await restartClient(context);
    }),
    vscode.commands.registerCommand("verifyOS.showOutput", () => {
      output().show(true);
    }),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (
        event.affectsConfiguration("verifyOS.path")
        || event.affectsConfiguration("verifyOS.profile")
        || event.affectsConfiguration("verifyOS.useBundledBinary")
      ) {
        await restartClient(context);
      }
    }),
  );

  await startClient(context);
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}
