<template>
  <div class="flex h-screen w-screen items-center justify-center">
    <UPageCard class="w-full max-w-md" spotlight spotlight-color="primary">
      <UChatShimmer
        text="正在登入……"
        :duration="1"
        :spread="10"
        class="text-center text-xl"
      />
    </UPageCard>
  </div>
</template>

<script setup lang="ts">
const supabase = useSupabaseClient();
const toast = useToast();

const checkUser = async () => {
  const start = Date.now();

  let authError = null;
  while (true) {
    const {
      data: { user },
      error,
    } = await supabase.auth.getUser();

    if (user != null) {
      return navigateTo("/");
    }

    if (error != null && Date.now() - start >= 5000) {
      authError = error;
      break;
    }

    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  console.error(authError);
  console.error(authError.toJSON());
  toast.add({
    title: "咪呀，登陆失败了……",
    description: "也许尝试重新登录或者刷新页面？",
    icon: "mdi:paw-off",
    color: "error",
  });
  return navigateTo("/signin");
};

onMounted(() => {
  checkUser();
});
</script>
