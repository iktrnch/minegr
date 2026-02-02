<script>
    import { SlidersHorizontal } from '@lucide/svelte'
    // Placeholder data; wire up to real API later
    let serverName = "BastionMC #1";
    let serverStatus = "Online"; // or "Offline"
    let playerCount = 12;
    let maxPlayers = 100;
    let serverImageUrl = "https://placehold.co/160x160/minecraft/white?text=Server";

    $effect(() => {
        // noop placeholder for future reactive bindings
    });

    const statusBadge = () =>
        serverStatus === "Online"
            ? "bg-green-600/15 text-green-600 ring-1 ring-green-600/30"
            : "bg-red-600/15 text-red-600 ring-1 ring-red-600/30";
</script>

<!-- Container to center the long card -->
<section class="mx-auto w-full max-w-5xl py-2">
    <!-- Long server card -->
    <div class="card card--soft w-full bg-surface-100/70 dark:bg-surface-200/40">
        <div class="p-4 lg:p-5">
            <div class="flex items-center gap-4 lg:gap-6">
                <!-- Server image (left) -->
                <div class="group relative h-20 w-20 shrink-0 lg:h-24 lg:w-24">
                    <img
                        src={serverImageUrl}
                        alt="Server banner"
                        class="h-full w-full rounded-lg object-cover ring-1 ring-surface-300"
                    />
                    <div class="absolute inset-0 flex items-center justify-center rounded-lg bg-blue-500/50 opacity-0 transition-opacity group-hover:opacity-100">
                        <SlidersHorizontal class="h-8 w-8 text-white lg:h-10 lg:w-10" />
                    </div>
                </div>

                <!-- Server name + status (middle) -->
                <div class="flex min-w-0 flex-1 items-center">
                    <div class="min-w-0">
                        <h2 class="text-xl font-bold tracking-tight text-on-surface lg:text-2xl">
                            {serverName}
                        </h2>
                        <div class="mt-1 flex items-center gap-2">
                            <span class={`badge rounded-full px-3 py-1 text-sm ${statusBadge()}`}>
                                {serverStatus}
                            </span>
                            <span class="text-sm text-muted-500">
                                MOTD: Adventure Awaits!
                            </span>
                        </div>
                    </div>
                </div>

                <!-- Player count (far right) -->
                <div class="min-w-[6rem] text-right">
                    <div class="text-2xl font-semibold tracking-tight text-on-surface">
                        {playerCount}/{maxPlayers}
                    </div>
                    <div class="text-xs text-muted-500">players online</div>
                </div>
            </div>
        </div>
    </div>
    
</section>