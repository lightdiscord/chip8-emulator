import Vue from 'vue';
import App from '@/components/app.vue';

Vue.config.productionTip = false

export const instance = new Vue({
    el: '#root',
    render: h => h(App),
});
