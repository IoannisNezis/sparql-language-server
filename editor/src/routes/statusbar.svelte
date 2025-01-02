<script lang="ts">
    import { type editor } from 'monaco-editor';
    interface Props {
        markers: editor.IMarker[];
    }
    let { markers }: Props = $props();
    // const Hint = 1;
    const Info = 2;
    const Warning = 4;
    const Error = 8;
    let errorCount = $derived(
        markers.filter((item: editor.IMarker) => item.severity == Error).length
    );
    let infoCount = $derived(
        markers.filter((item: editor.IMarker) => item.severity == Info).length
    );
    let warningCount = $derived(
        markers.filter((item: editor.IMarker) => item.severity == Warning).length
    );
</script>

<div class="flex w-full flex-row-reverse border-t border-gray-700 px-5 py-1">
    <div id="diagnosticOverview" class="grid shrink grid-cols-3 gap-2 text-white">
        <div class="flex flex-row">
            <img class="w-6 text-white" src="error.svg" alt="Errors:" />
            {errorCount}
        </div>
        <div class="flex flex-row">
            <img class="w-6 text-white" src="warning.svg" alt="Warnings:" />
            {warningCount}
        </div>
        <div class="flex flex-row">
            <img class="w-6 text-white" src="info.svg" alt="Infos:" />
            {infoCount}
        </div>
    </div>
</div>
