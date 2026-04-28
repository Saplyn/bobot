<template>
  <div class="flex h-screen w-screen items-center justify-center">
    <UPageCard
      class="w-full max-w-3xs md:max-w-md"
      spotlight
      spotlight-color="primary"
    >
      <UAuthForm
        title="登入"
        description="登入波波信息在线"
        icon="mdi:paw"
        :providers="providers"
        class="h-full grow"
      />
    </UPageCard>
  </div>
</template>

<script setup lang="ts">
import type { ButtonProps } from "@nuxt/ui";

const supabase = useSupabaseClient();
const url = useRequestURL();

async function signinWithQQ() {
  const { error } = await supabase.auth.signInWithOAuth({
    provider: "custom:bobot",
    options: {
      redirectTo: `${url.origin}/signin/confirm`,
    },
  });

  if (error) {
    console.error(error);
    console.error(error.toJSON());
  }
}

const providers = ref<ButtonProps[]>([
  {
    label: "QQ",
    icon: "mdi:qqchat",
    color: "primary",
    variant: "solid",
    onClick: signinWithQQ,
  },
]);
</script>
