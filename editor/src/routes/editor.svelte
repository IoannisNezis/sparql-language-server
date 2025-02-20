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

    let showTree = $state(false);
</script>

<div class="relative grid grid-cols-3">
    <div
        id="editor"
        class="container transition-all {showTree ? 'col-span-2' : 'col-span-3'}"
        bind:this={editorContainer}
    ></div>
    {#if showTree}
        <Tree input={content}></Tree>
    {/if}

    <button
        onclick={() => (showTree = !showTree)}
        class="absolute right-2 top-2 rounded bg-gray-700 px-2 py-2 font-bold text-white hover:bg-gray-600"
    >
        <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-5 transition duration-200 {showTree ? 'rotate-180' : ''}"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="m18.75 4.5-7.5 7.5 7.5 7.5m-6-15L5.25 12l7.5 7.5"
            />
        </svg>
    </button>
</div>
<Statusbar {markers}></Statusbar>

<style>
    #editor {
        height: 60vh;
    }
</style>
