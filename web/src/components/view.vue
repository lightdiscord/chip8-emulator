<template>
    <canvas ref="canvas"></canvas>
</template>

<script lang="ts">
import Vue from 'vue';

const WIDTH = 64;
const HEIGHT = 32;

export default Vue.extend({
    props: ['program'],
    data: () => ({
        afId: undefined,
        intervalId: null
    }),

    mounted() {
        if (!this.program) return;

        const { canvas } = this.$refs;
        const context = canvas.getContext('2d');

        const draw = () => {
            this.afId = window.requestAnimationFrame(draw);

            const screen = this.program.screen();
            const x_size = canvas.width / WIDTH;
            const y_size = canvas.height / HEIGHT;

            context.clearRect(0, 0, canvas.width, canvas.height);
            for (let y = 0; y < HEIGHT; y += 1) {
                for (let x = 0; x < WIDTH; x += 1) {
                    context.fillStyle = screen[y][x] ? '#ffffff' : '#000000';
                    context.fillRect(x * x_size, y * y_size, x_size, y_size);
                }
            }
        };

        this.afId = window.requestAnimationFrame(draw);
        this.intervalId = setInterval(() => {
            for (let i = 0; i < 10; i++) {
                this.program.tick();
            }
            this.program.decrement_timers();
        }, 1000 / 60);
    },

    destroyed() {
        if (!this.program) return;

        window.cancelAnimationFrame(this.afId);
        clearInterval(this.intervalId);
    }
});
</script>
