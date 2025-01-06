import getTextmateServiceOverride from "@codingame/monaco-vscode-textmate-service-override";
import languageServerWorkerUrl from "./languageServer.worker?worker&url";
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import TextMateWorker from '@codingame/monaco-vscode-textmate-service-override/worker?worker';
import sparqlTextmateGrammar from './sparql.tmLanguage.json?raw';
import sparqlLanguateConfig from './sparql.configuration.json?raw';
import sparqlTheme from './sparql.theme.json?raw';
import type { Logger } from 'monaco-languageclient/tools';
import type { WrapperConfig } from 'monaco-editor-wrapper';
import { LogLevel, Uri } from 'vscode';
import { useWorkerFactory } from 'monaco-editor-wrapper/workerFactory';

export async function buildWrapperConfig(htmlContainer: HTMLElement, initial_text: string): Promise<WrapperConfig> {

        const workerPromise: Promise<Worker> = new Promise((resolve) => {
                const instance = new Worker(new URL(languageServerWorkerUrl, window.location.origin),
                        {
                                name: "Language Server",
                                type: "module"
                        }
                );
                instance.onmessage = (event) => {
                        if (event.data.type === "ready") {
                                resolve(instance);
                        }
                };
        });
        const worker = await workerPromise;



        const extensionFilesOrContents = new Map<string, string | URL>();
        extensionFilesOrContents.set('/sparql-configuration.json', sparqlLanguateConfig);
        extensionFilesOrContents.set('/sparql-grammar.json', sparqlTextmateGrammar);
        extensionFilesOrContents.set('/sparql-theme.json', sparqlTheme);

        return {
                $type: 'extended',
                htmlContainer: htmlContainer,
                logLevel: LogLevel.Debug,
                vscodeApiConfig: {
                        userConfiguration: {
                                json: JSON.stringify({
                                        'workbench.colorTheme': 'SPARQL Custom Theme',
                                        'editor.guides.bracketPairsHorizontal': 'active',
                                        'editor.lightbulb.enabled': 'On',
                                        'editor.wordBasedSuggestions': 'off',
                                        'editor.experimental.asyncTokenization': true
                                })
                        },
                        serviceOverrides: {
                                // ...getConfigurationServiceOverride(),
                                ...getTextmateServiceOverride(),
                                // ...getThemeServiceOverride()
                        }
                },
                editorAppConfig: {
                        codeResources: {
                                modified: {
                                        uri: 'query.rq',
                                        text: initial_text
                                }
                        },
                        monacoWorkerFactory: configureMonacoWorkers,
                        editorOptions: {
                                theme: 'vs-dark',
                                fontSize: 16,
                                fontFamily: 'Source Code Pro',
                                links: false,
                                minimap: {
                                        enabled: false
                                },
                                overviewRulerLanes: 0,
                                scrollBeyondLastLine: false,
                                padding: {
                                        top: 10,
                                        bottom: 10
                                }
                        }
                },

                languageClientConfigs: {
                        sparql: {
                                name: "Qlue-ls",
                                clientOptions: {
                                        documentSelector: [{ language: 'sparql' }],
                                        workspaceFolder: {
                                                index: 0,
                                                name: "workspace",
                                                uri: Uri.file("/"),
                                        }
                                },
                                connection: {
                                        options: {
                                                $type: 'WorkerDirect',
                                                worker: worker
                                        }

                                }
                                ,
                                restartOptions: {
                                        retries: 5,
                                        timeout: 1000,
                                        keepWorker: true
                                }
                        }
                },
                extensions: [{
                        config: {
                                name: 'langium-sparql',
                                publisher: 'Ioannis Nezis',
                                version: '1.0.0',
                                engines: {
                                        vscode: '*'
                                },
                                contributes: {
                                        languages: [{
                                                id: 'sparql',
                                                extensions: ['.rq'],
                                                aliases: ['sparql', 'SPARQL'],
                                                configuration: '/sparql-configuration.json'
                                        }],
                                        themes: [
                                                {
                                                        "label": "SPARQL Custom Theme",
                                                        "uiTheme": "vs-dark",
                                                        "path": "./sparql-theme.json"
                                                }
                                        ],
                                        grammars: [{
                                                language: 'sparql',
                                                scopeName: 'source.sparql',
                                                path: '/sparql-grammar.json'
                                        }]
                                }
                        },
                        filesOrContents: extensionFilesOrContents
                }]
        }
}
export const configureMonacoWorkers = (logger?: Logger) => {
        const textmateWorker = new TextMateWorker({ name: "claris" });
        textmateWorker.onmessage = (event) => {
                console.log(event);

        };
        useWorkerFactory({
                workerOverrides: {
                        ignoreMapping: true,
                        workerLoaders: {
                                TextEditorWorker: () => new EditorWorker(),
                                TextMateWorker: () => textmateWorker
                        }
                },
                logger
        });
};
