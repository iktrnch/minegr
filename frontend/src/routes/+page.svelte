<script>
    import Server from "./Server.svelte";
    import Modal from "../lib/Modal.svelte";
    import CreateServer from "./CreateServer.svelte";
    import { SquarePlus } from '@lucide/svelte';
    import { LogOut } from '@lucide/svelte';

    let openModal = $state(false);
</script>
<div class="container mx-auto flex flex-col items-center justify-center py-8">
    <Server />
    <Server />
    <Server />
    <div class="mx-auto w-full max-w-5xl mx-4 flex flex-row items-center justify-between gap-[10px] py-2">
        <button onclick={() => {openModal = true}} class="flex-1 px-6 h-14 text-lg bg-blue-600 text-white hover:bg-blue-700 flex items-center justify-center rounded-l-md rounded-r-none">
            <SquarePlus class="h-5 w-5" />
            <span class="ml-2">Add Server</span>
        </button>
        <button aria-label="Log Out" class="bg-red-600 text-white hover:bg-red-700 flex items-center justify-center rounded-r-md rounded-l-none w-14 h-14">
            <LogOut class="h-5 w-5" />
            <span class="sr-only">Log Out</span>
        </button>
    </div>
    <Modal bind:open={openModal}>
        <CreateServer />
    </Modal>
</div>