<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import Statusbar from './statusbar.svelte';
    import type { MonacoEditorLanguageClientWrapper } from 'monaco-editor-wrapper';
    import type { editor } from 'monaco-editor';
    import Tree from './tree.svelte';

    let editorContainer: HTMLElement;
    let wrapper: MonacoEditorLanguageClientWrapper | undefined;
    let markers: editor.IMarker[] = $state([]);
    let content = $state('SELECT * {}');

    onMount(async () => {
        const { MonacoEditorLanguageClientWrapper } = await import('monaco-editor-wrapper');
        const { buildWrapperConfig } = await import('$lib/config');
        const monaco = await import('monaco-editor');

        wrapper = new MonacoEditorLanguageClientWrapper();
        let wrapperConfig = await buildWrapperConfig(editorContainer, content);
        await wrapper.initAndStart(wrapperConfig);
        monaco.editor.onDidChangeMarkers(() => {
            markers = monaco.editor.getModelMarkers({});
        });
        wrapper
            .getEditor()!
            .getModel()!
            .onDidChangeContent(() => {
                content = wrapper?.getEditor()!.getModel()!.getValue();
            });
    });

    onDestroy(() => {
        wrapper?.dispose(true);
    });
</script>

<div class="grid grid-cols-2">
    <div>
        <div id="editor" class="container" bind:this={editorContainer}></div>
        <Statusbar {markers}></Statusbar>
    </div>
    <Tree input={content}></Tree>
</div>

<style>
    #editor {
        height: 60vh;
    }
</style>
