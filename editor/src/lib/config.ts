import languageServerWorkerUrl from "./languageServer.worker?worker&url";
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
// import TextMateWorker from '@codingame/monaco-vscode-textmate-service-override/worker?worker';
import sparqlTextmateGrammar from './sparql.tmLanguage.json?raw';
import sparqlLanguateConfig from './sparql.configuration.json?raw';
import sparqlTheme from './sparql.theme.json?raw';
import type { WrapperConfig } from 'monaco-editor-wrapper';
import { LogLevel, Uri } from 'vscode';


export async function buildWrapperConfig(container: HTMLElement, initial: string): Promise<WrapperConfig> {
        window.MonacoEnvironment = {
                getWorker: (_moduleId, _label) => new editorWorker()
        }


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

        const wrapperConfig: WrapperConfig = {
                $type: 'extended',
                htmlContainer: container,
                logLevel: LogLevel.Debug,
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
                editorAppConfig: {
                        codeResources: {
                                modified: {
                                        uri: 'query.rq',
                                        text: initial
                                }
                        },
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
                                        // grammars: [{
                                        //         language: 'sparql',
                                        //         scopeName: 'source.sparql',
                                        //         path: '/sparql-grammar.json'
                                        // }]
                                }
                        },
                        filesOrContents: extensionFilesOrContents
                }]
        };
        return wrapperConfig;
}
