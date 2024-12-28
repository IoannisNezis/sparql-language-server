<script>
    import Editor from './editor.svelte';
    import Header from './header.svelte';
    import LoadingAnimation from './loadingAnimation.svelte';
    let editorReady = $state(true);
</script>

<Header></Header>
<div class="container m-auto">
    {#if !editorReady}
        <div class="mt-10 grid h-[650px] place-content-center rounded-sm border-2 bg-[#282828] p-2">
            <div class="flex flex-col items-center gap-3">
                <LoadingAnimation></LoadingAnimation>
                <div class="p-2 text-3xl text-white">Loading Monaco Editor ...</div>
            </div>
        </div>
    {/if}
    <div class="mt-10 rounded-sm border-2 border-gray-700 bg-[#282828]" class:hidden={!editorReady}>
        <Editor></Editor>
    </div>
</div>
