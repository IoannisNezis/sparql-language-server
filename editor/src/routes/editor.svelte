<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import Statusbar from './statusbar.svelte';
    import type { MonacoEditorLanguageClientWrapper } from 'monaco-editor-wrapper';
    import type { editor } from 'monaco-editor';

    let editorContainer: HTMLElement;
    let wrapper: MonacoEditorLanguageClientWrapper | undefined;
    let markers: editor.IMarker[] = $state([]);

    onMount(async () => {
        const { MonacoEditorLanguageClientWrapper } = await import('monaco-editor-wrapper');
        const { buildWrapperConfig } = await import('$lib/config');
        const monaco = await import('monaco-editor');

        wrapper = new MonacoEditorLanguageClientWrapper();
        let wrapperConfig = await buildWrapperConfig(editorContainer, '');
        await wrapper.initAndStart(wrapperConfig);
        let editor = wrapper.getEditor()!;
        // editor.getModel()?.setValue('SELECT * WHERE {\n  ?sub schema:name ?name\n}');
        console.log(editor);
        editor.getModel()?.setValue(
            `SELECT * WHERE {
  ?x <http://www.w3.org/2000/01/rdf-schema#label> ?label
}`
        );
        monaco.editor.onDidChangeMarkers(() => {
            markers = monaco.editor.getModelMarkers({});
        });
    });

    onDestroy(() => {
        wrapper?.dispose(true);
    });
</script>

<div>
    <div id="editor" class="container" bind:this={editorContainer}></div>
    <Statusbar {markers}></Statusbar>
</div>

<style>
    #editor {
        height: 60vh;
    }
</style>
