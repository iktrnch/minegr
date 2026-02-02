<script>
    import { fade, fly } from "svelte/transition";
    // Controls modal visibility (Svelte 5 runes)
    let { open = $bindable(false), children } = $props();

</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') open = false; }} />
{#if open}
    <div class="fixed inset-0 z-50 flex items-center justify-center">
        <!-- Backdrop: blur everything behind -->
        <div
            transition:fade
            class="absolute inset-0 bg-black/30 backdrop-blur-sm"
            role="button"
            tabindex="0"
            onclick={() => { open = false; }}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') open=false; }}
        ></div>

        <!-- Modal container: same background as body, scrollable, responsive width -->
        <div
            transition:fly={{ y: -200, duration: 400 }}
            class="relative w-full md:w-[80%] max-h-[85vh] overflow-y-auto rounded-lg shadow-lg bg-surface-100 dark:bg-surface-200 text-on-surface p-6"
            role="dialog" aria-modal="true" tabindex="0"
        >
            {@render children()}
        </div>
    </div>
{/if}

<style>
    /* Optional: avoid scrollbar overlaying rounded corners by clipping */
    :global(.rounded-lg) {
        overflow: hidden;
    }
    /* Ensure iOS bounce doesn't show behind when body is locked */
    :global(html.overflow-hidden), :global(body.overflow-hidden) {
        overscroll-behavior: contain;
    }
    /* Improve backdrop performance */
    :global(.backdrop-blur-sm) {
        will-change: backdrop-filter;
    }
</style>
