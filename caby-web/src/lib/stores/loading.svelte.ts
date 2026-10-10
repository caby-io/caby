export const app_loading = $state({ active: false });

export const useAppLoading = (loading: () => boolean) => {
	$effect(() => {
		app_loading.active = loading();
		return () => (app_loading.active = false);
	});
};
