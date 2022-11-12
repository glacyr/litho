/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as path from "path";
import * as url from "url";
import * as vscode from "vscode";

import {
  Disposable,
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;
let textDocumentContentProvider: Disposable;

export function activate(context: vscode.ExtensionContext) {
  // The server is implemented in node
  const serverModule = context.asAbsolutePath(
    path.join(process.platform === "win32" ? "litho-lsp.exe" : "litho-lsp")
  );
  // The debug options for the server
  // --inspect=6009: runs the server in Node's Inspector mode so VS Code can attach to the server for debugging
  const debugOptions = { execArgv: ["--nolazy", "--inspect=6009"] };

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  const serverOptions: ServerOptions = {
    run: {
      command: serverModule,
      transport: TransportKind.stdio,
    } as Executable,
    debug: {
      command: serverModule,
      transport: TransportKind.stdio,
      options: debugOptions,
    } as Executable,
  };

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "graphql" }],
    uriConverters: {
      code2Protocol: (uri) => {
        const result = new url.URL(uri.toString(true));
        result.search = new URLSearchParams(result.searchParams).toString();
        return result.toString();
      },
      protocol2Code: (str) => vscode.Uri.parse(str),
    },
  };

  const provider = new (class implements vscode.TextDocumentContentProvider {
    provideTextDocumentContent(
      uri: vscode.Uri,
      token: vscode.CancellationToken
    ): vscode.ProviderResult<string> {
      return client.sendRequest(
        "textDocument/content",
        { url: clientOptions.uriConverters.code2Protocol(uri) },
        token
      );
    }
  })();

  textDocumentContentProvider =
    vscode.workspace.registerTextDocumentContentProvider("litho", provider);

  // Create the language client and start the client.
  client = new LanguageClient("litho", "Litho", serverOptions, clientOptions);

  // Start the client. This will also launch the server
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  textDocumentContentProvider?.dispose();
  if (!client) {
    return undefined;
  }
  return client.stop();
}
