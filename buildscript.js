import esbuild from "esbuild";
import sveltePlugin from "esbuild-svelte";
import sveltePreprocess from "svelte-preprocess";

esbuild
  .build({
    entryPoints: ["src/app.ts"],
    mainFields: ["svelte", "browser", "module", "main"],
    bundle: true,
    minify: true,
    format: "esm",
    outfile: "build/bundle.js",
    plugins: [
      sveltePlugin({
        preprocess: sveltePreprocess()
      })
    ],
    logLevel: "info",
  })
  .catch((error, location) => {
    console.warn(`Errors: `, error, location);
    process.exit(1)
  });
