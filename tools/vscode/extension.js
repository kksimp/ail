// AIL for VS Code: starts `ail lsp` (ADR 0966, tools/ailsp/ailsp.ail) on stdio.
// The server shells out to `ail check/api`; it finds the binary through the AIL environment
// variable, so this client sets it to the same path it launches.
"use strict";
const fs = require("fs");
const path = require("path");
const vscode = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;
let log;

function resolveAil() {
  const configured = vscode.workspace.getConfiguration("ail").get("path", "").trim();
  if (configured) return configured;
  for (const folder of vscode.workspace.workspaceFolders || []) {
    const script = path.join(folder.uri.fsPath, process.platform === "win32" ? "ail.cmd" : "ail");
    if (fs.existsSync(script)) return script;
  }
  return "ail";
}

async function start() {
  const cfg = vscode.workspace.getConfiguration("ail");
  const ail = resolveAil();
  const env = Object.assign({}, process.env, { AIL: ail });
  if (cfg.get("debug", false)) env.AIL_LSP_DEBUG = "1";
  const run = { command: ail, args: ["lsp"], transport: TransportKind.stdio, options: { env } };
  client = new LanguageClient(
    "ail",
    "AIL",
    { run, debug: run },
    {
      documentSelector: [{ scheme: "file", language: "ail" }],
      outputChannel: log,
      traceOutputChannel: log,
    }
  );
  try {
    await client.start();
  } catch (e) {
    vscode.window.showErrorMessage(
      `AIL: could not start \`${ail} lsp\` (${e.message}). Set "ail.path" to your ail executable.`
    );
  }
}

async function stop() {
  if (client) {
    const c = client;
    client = undefined;
    try { await c.stop(); } catch (_) { /* already gone */ }
  }
}

async function activate(context) {
  log = vscode.window.createOutputChannel("AIL");
  context.subscriptions.push(
    log,
    vscode.commands.registerCommand("ail.restartServer", async () => { await stop(); await start(); }),
    vscode.commands.registerCommand("ail.showLog", () => log.show(true)),
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration("ail.path") || e.affectsConfiguration("ail.debug")) {
        await stop();
        await start();
      }
    })
  );
  await start();
}

module.exports = { activate, deactivate: stop };
