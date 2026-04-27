<template>
  <div class="flex h-screen w-screen items-center justify-center">
    <UPageCard class="w-full max-w-md" spotlight spotlight-color="primary">
      <UAuthForm
        title="登入"
        description="使用 QQ 登入波波信息在线"
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

async function signinWithQQ() {
  const { error } = await supabase.auth.signInWithOAuth({
    provider: "custom:bobot",
  });

  if (error) console.error(error);
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
