import { createRouter, createWebHistory } from "vue-router";

import ChannelView from "../views/ChannelView.vue";
import HomeView from "../views/HomeView.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      component: HomeView,
    },
    {
      path: "/channels",
      component: HomeView,
    },
    {
      path: "/channels/:id",
      component: ChannelView,
    },
  ],
});

export default router;
