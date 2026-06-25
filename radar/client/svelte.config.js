import adapter from "@sveltejs/adapter-static";

/** @type {import("@sveltejs/vite-plugin-svelte").SvelteConfig} */
export default {
    kit: {
        adapter: adapter({
            pages: "build",
            assets: "build",
            fallback: undefined,
            precompress: false,
            strict: true,
        }),
    },
};
