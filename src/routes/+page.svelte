<script lang="ts">
	import { Button, Spinner, Card, Textarea, Modal, Label, Select, Input } from 'flowbite-svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import {
		MicrophoneSolid,
		PauseSolid,
		ExclamationCircleOutline,
		EnvelopeSolid
	} from 'flowbite-svelte-icons';
	import { emit, listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	//document.addEventListener('contextmenu', event => event.preventDefault());

	let rec_time_text = '00:00:00';
	let rec_state = 0;
	let trans_text = '';
	let format_text = `- Datos socio demográficos del paciente : sexo , edad , ocupación y lugar de residencia.
- Motivo de consulta y enfermedad actual y evolución,  para describir los síntomas , detalla tiempo de evolución de los síntomas , características de los mismos y manejo recibido hasta el momento.
- Antecedentes personales : patológicos , farmacológicos , alérgicos , quirúrgicos , traumáticos y tóxicos
- Resultados paraclínicos (detallar la fecha de cada uno y si no está poner “falta fecha” y que se ordenen de más antiguo a más nuevo. La tasa filtración glomerular debe ir en mililitros.
- Examen físico (la respuesta después de paraclínicos), incluye frecuencia cardíaca, órganos de los sentidos , cuello, cardiopulmonar, abdomen y extremidades  con base única y exclusivamente en la conversación suministrada.
- Diagnóstico (Codifica cada cosa con el código de clasificación CIE10 y Define si es principal o secundario y si es nuevo o repetido)
- Análisis y plan contemplando medicamentos Y conducta a seguir`;
	let result_text = '';
	let devices = [];
	let selected_device = undefined;

	let error_modal = false;
	let error_text = '';
	let dbg_modal = false;
	let dbg_text = '';

	let welcome_modal = false;
	let email_invalid = true;
	let email = '';

	onMount(async () => {
		update_devices();

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
		const unlisten_4 = await listen('panic', (event) => {
			error_text += event.payload + '\n\n';
			error_modal = true;
			console.log(event.payload);
		});
		const unlisten_5 = await listen('dbg_msg', (event) => {
			dbg_text += event.payload;
			dbg_text += '----\n';
			dbg_modal = true;
			console.log(event.payload);
		});

		welcome_modal = await invoke('get_welcome');

		//emit('front_ready');
		//await invoke('front_ready');
	});

	async function update_devices() {
		let _devices = await invoke('get_devices');
		console.log(_devices);
		devices = [];
		for (const dev of _devices) {
			devices.push({ value: dev, name: dev });
		}
		selected_device = devices[0].value;
	}

	async function record_btn() {
		if (rec_state == 0) {
			console.log('Rec');
			rec_time_text = '00:00:00';
			await invoke('start_recording', { device: selected_device });
			trans_text = '';
			result_text = '';
			rec_state = 1;
		} else if (rec_state == 1) {
			console.log('Stop');
			rec_state = 2;
			await invoke('stop_recording', { formatText: format_text });
			rec_state = 0;
		}
	}

	function validate_email() {
		const emailInput = document.getElementById('email');
		email_invalid = !(emailInput && emailInput.checkValidity() && email != '');
	}

	async function enter_btn() {
		console.log('click');
		await invoke('set_email', { email: email });
		welcome_modal = false;
	}
</script>

<div class="flex h-screen flex-col">
	<div class="flex min-h-0 justify-center gap-x-2 p-2">
		<Card class="min-h-0 max-w-full">
			<Label>
				Micrófono:
				<Select
					class="mt-2"
					items={devices}
					placeholder="Seleccione una opción:"
					bind:value={selected_device}
				/>
			</Label>
		</Card>
	</div>
	<div class="flex min-h-0 grow justify-center gap-x-2 p-2">
		<Card class="min-h-0 max-w-full">
			<h1 class="mb-2 text-center text-2xl font-bold text-gray-900 dark:text-white">
				Transcripción
			</h1>
			<!-- <p class="font-normal p-2 h-full dark:text-gray-200 border border-gray-200 rounded-lg leading-tight overflow-y-auto">Hello, thanks for reaching out.
				I've unfortunately not received the package yet, do you have any way to see the tracking info?
				I made this order to test if it landed in my country (Colombia), but I didn't have high hopes because in my experience the packages that are sent without tracking info are often lost by the local postal service. In the past, the only reliable way for me to receive orders was to use a mail forwarding service, but I noticed that it isn't allowed anymore. 
				However, I was wondering if you could allow me to ship my next order to such forwarding service? I would be okay with losing any reship possibility in the event of no arrival since I know that it is more risky. Or if there's any way to have it shipped to my country with tracking, that could also work.
				Thank you for your kind help.</p> -->
			<Textarea class="h-full resize-none" value={trans_text} spellcheck="false"></Textarea>
		</Card>
		<Card class="min-h-0 max-w-full">
			<h1 class="mb-2 text-center text-2xl font-bold text-gray-900 dark:text-white">Formato</h1>
			<Textarea class="h-full resize-none" bind:value={format_text} spellcheck="false"></Textarea>
			<h1 class="mb-2 text-center text-2xl font-bold text-gray-900 dark:text-white">
				Interpretación
			</h1>
			<Textarea class="h-full resize-none" readonly value={result_text} spellcheck="false"
			></Textarea>
			<!-- <p class="font-normal p-2 h-full text-gray-700 dark:text-gray-200 border border-gray-200 rounded-lg leading-tight overflow-y-auto"></p> -->
		</Card>
	</div>
	<div class="flex justify-center">
		<Button color="red" pill={true} class="m-5 !p-4" on:click={record_btn}>
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

<Modal title="Ha ocurrido un error" bind:open={error_modal} size="lg" autoclose>
	<div class="text-center">
		<ExclamationCircleOutline class="mx-auto mb-4 h-12 w-12 text-gray-400 dark:text-gray-200" />
		<p class="whitespace-pre-wrap text-base leading-relaxed dark:text-white">{error_text}</p>
		<Button color="red" class="me-2">Cerrar</Button>
	</div>
</Modal>

<Modal title="Mensaje debug" bind:open={dbg_modal} size="lg" autoclose>
	<div class="text-center">
		<p class="whitespace-pre-wrap text-base leading-relaxed dark:text-white">{dbg_text}</p>
		<Button color="red" class="me-2">Cerrar</Button>
	</div>
</Modal>

<Modal title="Bienvenido a InteliAgente" bind:open={welcome_modal} size="md" dismissable={false}>
	<div class="text-center">
		<p class="whitespace-pre-wrap text-base leading-relaxed dark:text-white">
			Por favor ingrese su correo electrónico, este será usado como su usuario de InteliAgente:
		</p>
		<br />
		<!-- <Label class="mb-2 block">Correo electrónico:</Label> -->
		<Input
			id="email"
			type="email"
			placeholder="correo@gmail.com"
			on:input={validate_email}
			bind:value={email}
		>
			<EnvelopeSolid slot="left" class="h-5 w-5 text-gray-500 dark:text-gray-400" />
		</Input><br />
		<Button color="green" class="me-2" bind:disabled={email_invalid} on:click={enter_btn}
			>Ingresar</Button
		>
	</div>
</Modal>
