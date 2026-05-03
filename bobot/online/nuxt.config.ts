// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  ssr: false,
  compatibilityDate: "2025-07-15",
  devtools: { enabled: true },
  modules: ["@nuxt/icon", "@nuxt/ui", "@nuxtjs/supabase"],
  css: ["~/assets/css/main.css"],
  supabase: {
    url: process.env.SUPABASE_URL,
    key: process.env.SUPABASE_KEY,
    useSsrCookies: false,
    redirectOptions: {
      login: "/signin",
      callback: "/signin/confirm",
    },
  },
});
