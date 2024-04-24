<script lang="ts">
	import { Button, Spinner, Card } from 'flowbite-svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { MicrophoneSolid, PauseSolid } from 'flowbite-svelte-icons';
	import { emit, listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	let recording = false;
	let rec_time_text = '00:00:00';

	onMount(async () => {
		const unlisten = await listen('rec_time', (event) => {
			console.log(event.payload);
			rec_time_text = event.payload;
		});
	});

	async function record_btn() {
		if (!recording) {
			rec_time_text = '00:00:00';
			await invoke('start_recording');
		} else {
			await invoke('stop_recording');
		}
		recording = !recording;
	}
</script>

<div class="flex h-screen items-center justify-center">
	<Card class="items-center gap-5">
		<h5 class="mb-2 text-center text-2xl font-bold tracking-tight text-gray-900 dark:text-white">
			Grabación
		</h5>

		<Button color="red" pill={true} class="!p-4" on:click={record_btn}>
			{#if recording}
				<PauseSolid class="mr-5 h-7 w-7" />
				<p>{rec_time_text}</p>
			{:else}
				<MicrophoneSolid class="h-7 w-7 " />
			{/if}
		</Button>
		<!-- {#if recording}
			<Spinner />
		{/if} -->
	</Card>
</div>
