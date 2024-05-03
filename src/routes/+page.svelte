<script lang="ts">
	import { Button, Spinner, Card, Textarea } from 'flowbite-svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { MicrophoneSolid, PauseSolid } from 'flowbite-svelte-icons';
	import { emit, listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	document.addEventListener('contextmenu', event => event.preventDefault());

	let rec_time_text = '00:00:00';
	let rec_state = 0;
	let trans_text = "";
	let format_text = `- Datos socio demográficos del paciente : sexo , edad , ocupación y lugar de residencia.
- Motivo de consulta y enfermedad actual y evolución,  para describir los síntomas , detalla tiempo de evolución de los síntomas , características de los mismos y manejo recibido hasta el momento.
- Antecedentes personales : patológicos , farmacológicos , alérgicos , quirúrgicos , traumáticos y tóxicos
- Resultados paraclínicos (detallar la fecha de cada uno y si no está poner “falta fecha” y que se ordenen de más antiguo a más nuevo. La tasa filtración glomerular debe ir en mililitros.
- Examen físico (la respuesta después de paraclínicos), incluye frecuencia cardíaca, órganos de los sentidos , cuello, cardiopulmonar, abdomen y extremidades  con base única y exclusivamente en la conversación suministrada.
- Diagnóstico (Codifica cada cosa con el código de clasificación CIE10 y Define si es principal o secundario y si es nuevo o repetido)
- Análisis y plan contemplando medicamentos Y conducta a seguir`;
	let result_text = "";

	onMount(async () => {
		const unlisten_1 = await listen('rec_time', (event) => {
			//console.log(event.payload);
			rec_time_text = event.payload;
		});
		const unlisten_2 = await listen('trans_text', (event) => {
			//console.log(event.payload);
			trans_text = event.payload;
		});
		const unlisten_3 = await listen('result_text', (event) => {
			//console.log(event.payload);
			result_text = event.payload;
		});
	});

	async function record_btn() {
		if(rec_state == 0) {
			console.log("Rec");
			rec_time_text = '00:00:00';
			await invoke('start_recording');
			rec_state = 1;
		} else if(rec_state == 1) {
			console.log("Stop");
			rec_state = 2;
			await invoke('stop_recording',{formatText: format_text});
			rec_state = 0;
		}

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
			<Textarea class="resize-none h-full" value={trans_text} spellcheck="false"></Textarea>
		</Card>
		<Card class="max-w-full min-h-0">
			<h1 class="mb-2 text-center text-2xl font-bold text-gray-900 dark:text-white">
				Formato
			</h1>
			<Textarea class="resize-none h-full" value={format_text} spellcheck="false"></Textarea>
			<h1 class="mb-2 text-center text-2xl font-bold text-gray-900 dark:text-white">
				Interpretación
			</h1>
			<Textarea class="resize-none h-full" readonly value={result_text} spellcheck="false"></Textarea>
			<!-- <p class="font-normal p-2 h-full text-gray-700 dark:text-gray-200 border border-gray-200 rounded-lg leading-tight overflow-y-auto"></p> -->
		</Card>
	</div>
	<div class="flex justify-center">
		<Button color="red" pill={true} class="!p-4 m-5" on:click={record_btn}>
			{#if rec_state == 1}
				<PauseSolid class="mr-5 h-7 w-7" />
				<p>{rec_time_text}</p>
			{:else if rec_state == 2}
				<Spinner class="" size="7" color="white" />
			{:else}
				<MicrophoneSolid class="h-7 w-7 " /> 
			{/if}
		</Button>
	</div>
</div>
