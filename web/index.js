import { Program, default as init } from '../wasm/pkg/chip8_wasm.js';

const rom = fetch('../roms/PONG')
	.then(response => response.arrayBuffer());

const canvas = document.querySelector('canvas');
const context = canvas.getContext('2d');

Promise.all([rom, init()]).then(([rom, wasm]) => {
	console.log('ready', rom, wasm, Program);

	const program = Program.new();
	program.load(new Uint8Array(rom));

	console.log(program);
	window.program = program;

	const draw = () => {
		window.requestAnimationFrame(draw);
		const screen = program.screen();
		context.clearRect(0, 0, canvas.width, canvas.height);

		let x_size = canvas.width / 64;
		let y_size = canvas.height / 32;


		for (let y = 0; y < 32; y += 1) {
			for (let x = 0; x < 64; x += 1) {
				if (screen[y][x]) {
					context.fillStyle = '#00ff00';
				} else {
					context.fillStyle = '#ff0000';
				}
				context.fillRect(x * x_size, y * y_size, x_size, y_size);
			}
		}
	}

	setInterval(() => {
		for (let i = 0; i < 10; i++) {
			program.tick();
		}
		program.decrement_timers();
	}, 1000 / 60);

	draw();
});
