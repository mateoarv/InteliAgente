<script lang="ts">
	import { Button, Spinner, Card, Textarea } from 'flowbite-svelte';
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

<div class="flex flex-col h-screen">
	<div class="flex grow justify-center gap-x-2 p-2 min-h-0">
		<Card class="max-w-full min-h-0">
			<h1 class="mb-2 text-center text-2xl font-bold text-gray-900 dark:text-white">
				Transcripción
			</h1>
			<!-- <p class="font-normal p-2 h-full dark:text-gray-200 border border-gray-200 rounded-lg leading-tight overflow-y-auto">Hello, thanks for reaching out.
				I've unfortunately not received the package yet, do you have any way to see the tracking info?
				I made this order to test if it landed in my country (Colombia), but I didn't have high hopes because in my experience the packages that are sent without tracking info are often lost by the local postal service. In the past, the only reliable way for me to receive orders was to use a mail forwarding service, but I noticed that it isn't allowed anymore. 
				However, I was wondering if you could allow me to ship my next order to such forwarding service? I would be okay with losing any reship possibility in the event of no arrival since I know that it is more risky. Or if there's any way to have it shipped to my country with tracking, that could also work.
				Thank you for your kind help.</p> -->
			<Textarea class="resize-none h-full" value="Test"></Textarea>
		</Card>
		<Card class="max-w-full min-h-0">
			<h1 class="mb-2 text-center text-2xl font-bold text-gray-900 dark:text-white">
				Formato
			</h1>
			<Textarea class="resize-none h-full" value="Test"></Textarea>
			<h1 class="mb-2 text-center text-2xl font-bold text-gray-900 dark:text-white">
				Interpretación
			</h1>
			<Textarea class="resize-none h-full" readonly value="Test"></Textarea>
			<!-- <p class="font-normal p-2 h-full text-gray-700 dark:text-gray-200 border border-gray-200 rounded-lg leading-tight overflow-y-auto"></p> -->
		</Card>
	</div>
	<div class="flex justify-center">
		<Button color="red" pill={true} class="!p-4 m-5" on:click={record_btn}>
			{#if recording}
				<PauseSolid class="mr-5 h-7 w-7" />
				<p>{rec_time_text}</p>
			{:else}
				<MicrophoneSolid class="h-7 w-7 " />
			{/if}
		</Button>
	</div>
</div>
