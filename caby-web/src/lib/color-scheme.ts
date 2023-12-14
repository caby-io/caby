import { browser } from '$app/environment';
import type { Writable, Updater } from 'svelte/store';
import { writable, get } from 'svelte/store'

const DEFAULT_SCHEME = "light"

const new_color_scheme = () => {
    const store = writable<string>(DEFAULT_SCHEME);
    const { set, subscribe } = store;

    const setScheme = (scheme: string) => {
        set(scheme)

        if (!browser) {
            return
        }

        // Persist
        localStorage.color_scheme = scheme;

        // Reset
        document.body.classList.remove("light")
        document.body.classList.remove("dark")

        if (scheme) {
            document.body.classList.add(scheme)
        }
    }

    const toggleScheme = () => {
        let new_scheme = "dark"
        if (get(store) == "dark") {
            new_scheme = "light"
        }

        setScheme(new_scheme)
    }

    const clearStorage = () => {
        localStorage.color_scheme = null;
    }

    if (browser) {
        let init_scheme = DEFAULT_SCHEME
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            init_scheme = "dark"
        }

        const stored_scheme = localStorage.color_scheme;
        if (stored_scheme) {
            init_scheme = stored_scheme;
        }
        setScheme(stored_scheme);
    }

    return {
        subscribe,
        setScheme,
        toggleScheme,
        clearStorage,
    };
}

export let color_scheme = new_color_scheme();
