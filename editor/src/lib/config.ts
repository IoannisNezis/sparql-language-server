import languageServerWorkerUrl from "./languageServer.worker?worker&url";
import sparqlTextmateGrammar from './sparql.tmLanguage.json?raw';
import sparqlLanguateConfig from './sparql.configuration.json?raw';
import sparqlTheme from './sparql.theme.json?raw';
import type { WrapperConfig } from 'monaco-editor-wrapper';
import { LogLevel } from 'vscode';
import { useWorkerFactory } from 'monaco-editor-wrapper/workerFactory';

export function buildWrapperConfig(htmlContainer: HTMLElement): WrapperConfig {

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
                                        'workbench.colorTheme': 'Default Dark Modern',
                                        'editor.guides.bracketPairsHorizontal': 'active',
                                        'editor.lightbulb.enabled': 'On',
                                        'editor.wordBasedSuggestions': 'off',
                                        'editor.experimental.asyncTokenization': true
                                })
                        }
                },
                editorAppConfig: {
                        codeResources: {
                                modified: {
                                        fileExt: 'rq',
                                        text: 'Select * where {}'
                                }
                        },
                        monacoWorkerFactory: configureMonacoWorkers,
                        editorOptions: {
                                language: 'sparql',
                                value: 'SELECT * WHERE {}',
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
                                        documentSelector: [{ language: 'sparql' }]
                                },
                                connection: {
                                        options: {
                                                $type: 'WorkerConfig',
                                                url: new URL(languageServerWorkerUrl, window.location.origin),
                                                type: 'module',
                                        }

                                }
                        }
                }
                // extensions: [
                // 	{
                // 		config: {
                // 			name: 'langium-sparql',
                // 			publisher: 'Ioannis Nezis',
                // 			version: '1.0.0',
                // 			engines: {
                // 				vscode: '*'
                // 			},
                // 			contributes: {
                // 				languages: [{
                // 					id: 'sparql',
                // 					extensions: ['.rq'],
                // 					configuration: './sparql-configuration.json'
                // 				}],
                // 				themes: [
                // 					{
                // 						"label": "SPARQL Custom Theme",
                // 						"uiTheme": "vs-dark",
                // 						"path": "./sparql-theme.json"
                // 					}
                // 				],
                // 				grammars: [{
                // 					language: 'sparql',
                // 					scopeName: 'source.sparql',
                // 					path: './sparql-grammar.json'
                // 				}]
                // 			}
                // 		},
                // 		filesOrContents: extensionFilesOrContents
                // 	}
                // ]
        }
}
// TODO: add logger monaco languageclient logger
export const configureMonacoWorkers = () => {
        useWorkerFactory({
                workerOverrides: {
                        ignoreMapping: true,
                        workerLoaders: {
                                TextEditorWorker: () => new Worker(new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url), { type: 'module' }),
                                TextMateWorker: () => new Worker(new URL('@codingame/monaco-vscode-textmate-service-override/worker', import.meta.url), { type: 'module' })
                        }
                },
                // logger
        });
};
