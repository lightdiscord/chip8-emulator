<template>
    <div>
        <div>
            <game-view v-if="program" :program="program" />
        </div>

        <div>
            <div>
            <label for="select-rom">Select a rom</label>

            <select id="select-rom" v-model="rom">
                <option disabled value="">Please select one</option>
                <option value="other">Other</option>
            </select>
            </div>

            <div v-if="rom === 'other'">
                <label for="select-file">Select file</label>

                <input type="file" ref="file" id="select-file">
            </div>

            <div>
                <button @click="load">Load rom</button>
            </div>
        </div>
    </div>
</template>

<script lang="ts">
import Vue from 'vue';

export default Vue.extend({
    data: () => ({
        rom: '',
        program: null
    }),
    methods: {
        load() {
            if (this.rom == 'other') {
                Promise.all([
                    import('@/../../wasm/pkg'),
                    this.$refs.file.files[0].arrayBuffer()
                ])
                    .then(([wasm, file]) => {
                        const program = wasm.Program.new();
                        program.load(new Uint8Array(file));
                        console.log(program, file);
                        this.program = program;
                        return program;
                    })
            }
        }
    },
    components: {
        gameView: () => import('./view.vue')
    }
});
</script>

