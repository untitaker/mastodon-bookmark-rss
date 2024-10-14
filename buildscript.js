import esbuild from "esbuild";

esbuild
  .build({
    entryPoints: ["src/app.js"],
    mainFields: ["browser", "module", "main"],
    bundle: true,
    minify: false,
    format: "esm",
    outfile: "build/bundle.js",
    logLevel: "info",
  })
  .catch((error, location) => {
    console.warn(`Errors: `, error, location);
    process.exit(1)
  });
