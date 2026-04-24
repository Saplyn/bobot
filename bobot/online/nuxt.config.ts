// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  ssr: false,
  compatibilityDate: "2025-07-15",
  devtools: { enabled: true },
  modules: ["@nuxt/icon", "@nuxt/ui", "@nuxtjs/supabase"],
  css: ["~/assets/css/main.css"],
  supabase: {
    useSsrCookies: false,
    redirectOptions: {
      login: "/signin",
      callback: "/signin/confirm",
    },
  },
});
