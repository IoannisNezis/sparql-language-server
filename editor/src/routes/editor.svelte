<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import Statusbar from './statusbar.svelte';
    import type { MonacoEditorLanguageClientWrapper } from 'monaco-editor-wrapper';

    let editorContainer: HTMLElement;
    let wrapper: MonacoEditorLanguageClientWrapper | undefined;

    onMount(async () => {
        const { MonacoEditorLanguageClientWrapper } = await import('monaco-editor-wrapper');
        const { buildWrapperConfig } = await import('$lib/config');

        wrapper = new MonacoEditorLanguageClientWrapper();
        let wrapperConfig = buildWrapperConfig(editorContainer);
        await wrapper.initAndStart(wrapperConfig);

        let editor = wrapper.getEditor();
    });

    onDestroy(() => {
        wrapper?.dispose(true);
        console.log('destroy');
    });
</script>

<div>
    <div id="editor" class="container" bind:this={editorContainer}></div>
    <Statusbar></Statusbar>
</div>

<style>
    #editor {
        height: 60vh;
    }
</style>
