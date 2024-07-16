<script lang="ts">
	import {
		Button,
		Spinner,
		Card,
		Textarea,
		Modal,
		Label,
		Select,
		Input,
		Tabs,
		TabItem
	} from 'flowbite-svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import {
		MicrophoneSolid,
		PauseSolid,
		ExclamationCircleOutline,
		EnvelopeSolid,
		PenSolid,
		ClipboardCheckSolid,
		AdjustmentsVerticalSolid
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

	let result_open = false;

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

		const tab_content = document
			.querySelector('.tabs')
			.children[2];

		tab_content.style.display = 'flex';
		tab_content.style.flexGrow = '1';
		tab_content.children[0].style.display = 'flex';
		tab_content.children[0].style.flexGrow = '1';

		const observer_config = { attributes: false, childList: true, subtree: false };
		const observer = new MutationObserver(tab_change);
		observer.observe(tab_content, observer_config);
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
			result_open = true;
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

	function tab_change(mutationsList, observer) {
		const tab_content = document
			.querySelector('.tabs')
			.children[2];

		tab_content.children[0].style.display = 'flex';
		tab_content.children[0].style.flexGrow = '1';
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

	<div class="tabs flex flex-col grow gap-x-2 p-2">
		<Tabs>
			<TabItem open>
				<div slot="title" class="flex items-center gap-2">
					<AdjustmentsVerticalSolid size="md" />
					Formato
				</div>
				<Textarea class="h-full resize-none" bind:value={format_text} spellcheck="false"></Textarea>
			</TabItem>
			<TabItem>
				<div slot="title" class="flex items-center gap-2">
					<PenSolid size="md" />
					Transcripción
				</div>
				<Textarea class="h-full resize-none" readonly value={trans_text} spellcheck="false"></Textarea>
			</TabItem>
			<TabItem bind:open={result_open}>
				<div slot="title" class="flex items-center gap-2">
					<ClipboardCheckSolid size="md" />
					Resultado
				</div>
				<Textarea class="h-full resize-none" readonly value={result_text} spellcheck="false"
				></Textarea>
			</TabItem>
		</Tabs>
	</div>
	<div class="flex justify-center">
		<Button color="red" pill={true} class="m-3 !p-4" on:click={record_btn}>
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
		>Ingresar
		</Button
		>
	</div>
</Modal>

<!--<style>-->
<!--    #abc:global(.group) {-->
<!--				background-color: #0e9f6e;-->
<!--    }-->

<!--</style>-->
