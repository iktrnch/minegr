<script lang="ts">
	type Tone = 'accent' | 'muted' | 'plain' | 'success';

	type TerminalPart = { text: string; tone: Tone };
	type TerminalLine = { parts: TerminalPart[]; gapAfter?: boolean };
	type Feature = {
		name: string;
		description: string;
		command: string;
		status: 'Planned';
	};

	const product = {
		name: 'minegr',
		description: '— a Minecraft server manager for your terminal',
		status: 'Pre-alpha',
		cta: {
			label: 'Star on GitHub',
			href: 'https://github.com/iktrnch/minegr'
		}
	} as const;

	const terminalLines: TerminalLine[] = [
		{
			parts: [
				{ text: '$ ', tone: 'muted' },
				{ text: 'minegr', tone: 'accent' },
				{ text: ' status', tone: 'plain' }
			]
		},
		{
			parts: [
				{ text: 'Status:       ', tone: 'muted' },
				{ text: 'running', tone: 'success' }
			]
		},
		{
			parts: [
				{ text: 'Memory usage: ', tone: 'muted' },
				{ text: '2048 MiB', tone: 'plain' }
			]
		},
		{
			parts: [
				{ text: 'CPU usage:    ', tone: 'muted' },
				{ text: '125.4%', tone: 'plain' }
			],
			gapAfter: true
		},
		{
			parts: [
				{ text: '$ ', tone: 'muted' },
				{ text: 'minegr', tone: 'accent' },
				{ text: ' logs --follow', tone: 'plain' }
			]
		}
	];

	const features: Feature[] = [
		{
			name: 'Configure',
			description: 'Create a server configuration for Vanilla, Paper, or Fabric.',
			command: 'minegr init',
			status: 'Planned'
		},
		{
			name: 'Start',
			description: 'Launch through a per-server daemon and wait for readiness.',
			command: 'minegr start',
			status: 'Planned'
		},
		{
			name: 'Observe',
			description: 'Read status, process metrics, and the current session log stream.',
			command: 'minegr logs --follow',
			status: 'Planned'
		},
		{
			name: 'Control',
			description: 'Use a focused TUI for live logs and server console input.',
			command: 'minegr console',
			status: 'Planned'
		},
		{
			name: 'Protect',
			description: 'Create consistent backups whether the server is stopped or running.',
			command: 'minegr backup',
			status: 'Planned'
		}
	];

	const footerItems = [
		{ label: 'minegr' },
		{ label: 'Pre-alpha' },
		{ label: 'Source on GitHub', href: product.cta.href }
	] as const;
</script>

<svelte:head>
	<title>minegr — a Minecraft server manager for your terminal</title>
	<meta
		name="description"
		content="Minegr is a pre-alpha Minecraft server manager designed around a fast CLI and TUI for Linux."
	/>
	<meta name="theme-color" content="#030806" />
</svelte:head>

<main class="min-h-screen overflow-hidden bg-[#030806] text-zinc-50">
	<section class="hero relative isolate border-b border-zinc-800/80" aria-labelledby="hero-title">
		<div class="signal-bleed" aria-hidden="true"></div>

		<div class="relative mx-auto max-w-[96rem] px-5 pt-7 sm:px-8 lg:px-14 lg:pt-9">
			<div class="flex justify-end">
				<p
					class="status-label flex items-center gap-2 text-xs font-semibold tracking-[0.14em] text-emerald-100/70 uppercase"
				>
					<span class="h-2 w-2 rounded-full bg-emerald-500" aria-hidden="true"></span>
					{product.status}
				</p>
			</div>

			<div class="hero-copy pt-12 sm:pt-14 lg:pt-12">
				<h1 id="hero-title" class="max-w-[68rem] text-balance">
					<span class="wordmark block font-black text-emerald-500">{product.name}</span>
					<span class="descriptor mt-6 block max-w-[23ch] font-normal text-zinc-50 sm:mt-8">
						{product.description}
					</span>
				</h1>

				<a
					class="primary-action mt-9 inline-flex min-h-13 items-center justify-center rounded-xl bg-emerald-500 px-7 py-3.5 text-base font-bold text-emerald-950 transition-[background-color,transform,box-shadow] duration-200 ease-out hover:-translate-y-0.5 hover:bg-emerald-400 hover:shadow-[0_14px_36px_rgba(16,185,129,0.18)] focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-emerald-300 active:translate-y-0"
					href={product.cta.href}
					target="_blank"
					rel="external noreferrer"
				>
					{product.cta.label}
				</a>
			</div>

			<figure
				class="terminal relative z-10 mt-16 ml-auto w-full max-w-[53rem] overflow-hidden rounded-[14px] bg-[#090d0b] shadow-[0_28px_80px_rgba(0,0,0,0.48)] lg:mt-[-5.4rem] lg:-mb-24 lg:w-[62%]"
			>
				<figcaption
					class="flex min-h-13 items-center justify-between border-b border-zinc-700/80 bg-zinc-900/80 px-4 sm:px-6"
				>
					<div class="flex items-center gap-2" aria-hidden="true">
						<span class="h-2.5 w-2.5 rounded-full bg-zinc-600"></span>
						<span class="h-2.5 w-2.5 rounded-full bg-zinc-700"></span>
						<span class="h-2.5 w-2.5 rounded-full bg-emerald-500"></span>
					</div>
					<span
						class="font-mono text-[0.68rem] tracking-[0.08em] text-zinc-400 uppercase sm:text-xs"
						>Design preview</span
					>
					<span
						class="font-mono text-[0.68rem] tracking-[0.08em] text-emerald-400 uppercase sm:text-xs"
						>Planned</span
					>
				</figcaption>

				<div
					class="terminal-screen p-5 font-mono text-[clamp(0.72rem,1.45vw,1.05rem)] leading-[1.7] sm:p-7 lg:min-h-72"
				>
					{#each terminalLines as line, lineIndex (lineIndex)}
						<div class:mb-4={line.gapAfter} class="whitespace-pre-wrap">
							{#each line.parts as part, partIndex (partIndex)}
								<span
									class:text-emerald-400={part.tone === 'accent'}
									class:text-emerald-300={part.tone === 'success'}
									class:text-zinc-500={part.tone === 'muted'}
									class:text-zinc-100={part.tone === 'plain'}>{part.text}</span
								>
							{/each}
						</div>
					{/each}
					<span
						class="terminal-cursor mt-1 inline-block h-[1.2em] w-[0.62em] bg-emerald-500 align-middle"
						aria-hidden="true"
					></span>
				</div>
			</figure>
		</div>
	</section>

	<section
		class="lifecycle relative mx-auto max-w-[96rem] px-5 pt-20 pb-24 sm:px-8 lg:px-14 lg:pt-40 lg:pb-36"
		aria-labelledby="features-title"
	>
		<h2 id="features-title" class="sr-only">Planned server lifecycle features</h2>

		<div
			class="lifecycle-line absolute top-[11.25rem] right-14 left-14 hidden h-px bg-zinc-700 lg:block"
			aria-hidden="true"
		>
			<span class="signal-trace block h-px bg-emerald-500"></span>
		</div>

		<ol class="relative grid gap-0 lg:grid-cols-5">
			{#each features as feature, index (feature.name)}
				<li
					class="feature-stage group relative border-t border-zinc-700 py-8 lg:border-t-0 lg:px-5 lg:pt-0 lg:pb-4 first:lg:pl-0 last:lg:pr-0"
				>
					<div class="mb-6 flex items-center justify-between lg:mb-9 lg:block">
						<span
							class="stage-marker relative z-10 flex h-10 w-10 items-center justify-center rounded-full border border-emerald-500 bg-[#030806] font-mono text-sm text-emerald-300 transition-colors duration-200 group-hover:bg-emerald-500 group-hover:text-emerald-950"
						>
							{index + 1}
						</span>
						<span
							class="font-mono text-[0.65rem] tracking-[0.12em] text-zinc-500 uppercase lg:mt-5 lg:block"
							>{feature.status}</span
						>
					</div>

					<h3 class="text-2xl font-semibold tracking-[-0.025em] text-emerald-400">
						{feature.name}
					</h3>
					<p class="mt-3 max-w-[28rem] text-sm leading-6 text-zinc-400 lg:min-h-24">
						{feature.description}
					</p>
					<code class="mt-5 block font-mono text-xs text-zinc-200">$ {feature.command}</code>
				</li>
			{/each}
		</ol>
	</section>

	<footer class="border-t border-zinc-800/80">
		<div
			class="mx-auto flex max-w-[96rem] flex-wrap items-center gap-x-3 gap-y-2 px-5 py-7 font-mono text-xs text-zinc-500 sm:px-8 lg:px-14"
		>
			{#each footerItems as item, index (item.label)}
				{#if index > 0}<span class="text-zinc-700" aria-hidden="true">·</span>{/if}
				{#if 'href' in item}
					<a
						class="underline decoration-zinc-700 underline-offset-4 transition-colors hover:text-emerald-400 focus-visible:rounded-sm focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-emerald-400"
						href={item.href}
						target="_blank"
						rel="external noreferrer">{item.label}</a
					>
				{:else}
					<span>{item.label}</span>
				{/if}
			{/each}
		</div>
	</footer>
</main>
