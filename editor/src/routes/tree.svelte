<script lang="ts">
    import { parse } from 'll-sparql-parser';

    let { input } = $props();
    let parseTree = $derived(parse(input));
</script>

{#snippet renderLeave(leave)}
    <div>
        <span>
            {leave.kind}:
        </span>
        <span class="w-min text-red-400">
            {leave.text}
        </span>
    </div>
{/snippet}

{#snippet renderTree(tree)}
    <span>
        {tree.kind}
    </span>
    <div class="ms-2 flex flex-col border-l ps-2">
        {#each tree.children as child}
            {#if child.Tree}
                <span>
                    {@render renderTree(child.Tree)}
                </span>
            {:else}
                {@render renderLeave(child.Token)}
            {/if}
        {/each}
    </div>
{/snippet}

<div
    id="treeContainer"
    style="height: 60vh;"
    class="overflow-y-auto overflow-x-hidden border-l border-gray-700 p-2 text-white"
>
    {@render renderTree(parseTree)}
</div>
