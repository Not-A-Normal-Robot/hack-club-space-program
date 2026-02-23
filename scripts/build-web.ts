#!/usr/bin/env -S deno -P
"use strict";

if (!("Deno" in globalThis))
{
    throw new Error("Deno object not found; please run this script using Deno. You can get deno at https://deno.land/");
}

import * as path from "@std/path";
import * as fs from "@std/fs";
import ClosureCompiler from "google-closure-compiler";
import { minify as minifyHtml } from "@minify-html/deno";

// General
const GAME_NAME = "hack-club-space-program";
const RELEASE_MODE = Deno.args.includes("--release");

// Paths
const DIRNAME = import.meta.dirname;
if (!DIRNAME)
{
    throw new Error("`import.meta.dirname` is undefined. This script must be run locally.");
}

const OUT_DIR = path.join(DIRNAME, "../target/wasm32-unknown-unknown/web");
const UNBOUND_WASM_PATH = path.join(
    DIRNAME,
    "../target/wasm32-unknown-unknown",
    RELEASE_MODE ? "release" : "debug",
    GAME_NAME + ".wasm",
);
const BOUND_WASM_PATH = path.join(
    OUT_DIR,
    GAME_NAME + "_bg.wasm",
);
const WBG_JS_PATH = path.join(OUT_DIR, GAME_NAME + ".js");
const ASSETS_PATH = path.join(DIRNAME, "../assets");
const INDEX_HTML_PATH = path.join(DIRNAME, "../web/index.html");
const MAIN_JS_PATH = path.join(DIRNAME, "../web/main.js");

// Web file processing
const JS_TEMPLATE_WASM_TOTAL_BYTES = "/*{{INLINER:WASM_TOTAL_SIZE}}*/";
const IMPORT_REGEX = /\bimport\b(?=\s*\()/;
const IMPORT_REPLACEMENT = "__Wk2hQs1ynle8rKILAmsDoYFD__";
const HTML_TEMPLATE_JS = "{{mainscript}}";
const MINIFY_HTML_CONFIG = {
    allow_noncompliant_unquoted_attribute_values: true,
    allow_optimal_entities: true,
    allow_removing_spaces_between_attributes: true,
    minify_css: true,
    minify_doctype: true,
    preserve_brace_template_syntax: true,
};
const WASM_OPT_LEVEL = "-O3";

/** Checks if the given executable is accessible, by running `<cmd> --version`. */
async function isExecAvailable(cmd: string): Promise<boolean>
{
    try
    {
        const command = new Deno.Command(cmd, {
            args: ["--version"],
            stdout: "null",
            stdin: "null",
        });
        return (await command.output()).success;
    } catch (e)
    {
        if (e instanceof Deno.errors.NotFound)
        {
            return false;
        }
        throw e;
    }
}

/**
 * Generates an error for when an executed process fails
 */
function execError(
    action: string,
    cmd: string,
    code: number,
    stdout: Uint8Array,
    stderr: Uint8Array,
): Error
{
    const decoder = new TextDecoder("utf-8");
    return new Error(
        `An error occurred while ${action}:\n` +
        `'${cmd}' returned with exit code ${code}\n\n` +
        `===== ${cmd} stdout =====\n\n${decoder.decode(stdout)}` +
        `===== ${cmd} stderr =====\n\n${decoder.decode(stderr)}`
    );
}

async function checkDependencies()
{
    const results = await Promise.allSettled(["cargo", "rustup"].map((cmd) =>
    {
        return isExecAvailable(cmd)
            .then((available) =>
            {
                if (available)
                {
                    return Promise.resolve();
                } else
                {
                    return Promise.reject(`Could not access \`${cmd}\`: is it in your PATH?\n` +
                        "One way to get cargo, rustup, and the rest of the rust toolchain is rustup: https://rustup.rs");
                }
            });
    }));

    const rejections = results.filter(r => r.status === "rejected").map(r => r.reason);

    if (rejections.length > 0)
    {
        for (const rejection of rejections)
        {
            console.error(rejection);
        }
        Deno.exit(1);
    }
}

async function prepWasmBindgen()
{
    if (!await isExecAvailable("wasm-bindgen"))
    {
        console.log("wasm-bindgen not found, installing...");
        const command = new Deno.Command("cargo", {
            args: ["install", "wasm-bindgen-cli"],
            stdout: "piped",
        });
        const output = await command.output();
        if (output.success)
        {
            console.log("Finished installing wasm-bindgen");
        } else
        {
            throw new Error(
                "Error occurred installing wasm-bindgen\n\n" +
                `===== cargo stdout =====\n${output.stdout}\n\n` +
                `===== cargo stderr =====\n${output.stderr}`
            );
        }
    }
    if (!await isExecAvailable("wasm-bindgen"))
    {
        throw new Error("Could not access wasm-bindgen after installation, something must have gone awry");
    }
}

async function prepRustupTarget()
{
    console.log("Checking list of installed rustup targets");
    const listCommand = new Deno.Command("rustup", {
        args: ["target", "list", "--installed"],
        stdin: "null",
        stdout: "piped"
    });
    const listOutput = await listCommand.output();
    if (!listOutput.success)
    {
        throw execError(
            "checking rustup's installed targets list",
            "rustup",
            listOutput.code,
            listOutput.stdout,
            listOutput.stderr,
        );
    }

    const list = new TextDecoder().decode(listOutput.stdout).split("\n").map(s => s.trim());

    if (list.includes("wasm32-unknown-unknown"))
    {
        console.log("No need to install wasm32-unknown-unknown as it's already installed");
        return;
    }

    console.log("rustup wasm32-unknown-unknown target is not installed, installing...");

    const installCommand = new Deno.Command("rustup", {
        args: ["target", "install", "wasm32-unknown-unknown"],
        stdin: "null",
        stdout: "piped",
    });
    const installOutput = await installCommand.output();

    if (!installOutput.success)
    {
        throw execError(
            "installing the wasm32-unknown-unknown rustup target",
            "rustup",
            installOutput.code,
            installOutput.stdout,
            installOutput.stderr,
        );
    }

    console.log("Installed wasm32-unknown-unknown rustup target");
}

async function buildWasm()
{
    const args = ["build", "--target", "wasm32-unknown-unknown"];

    if (RELEASE_MODE)
    {
        args.push("--release");
    }

    console.log("Compiling the Rust codebase to WASM...");

    const command = new Deno.Command("cargo", {
        args,
        stdin: "null",
        stdout: "piped",
    });

    const output = await command.output();

    if (!output.success)
    {
        throw execError(
            "compiling Rust to WASM",
            "cargo",
            output.code,
            output.stdout,
            output.stderr,
        );
    }

    console.log("Rust to WASM compilation complete!");
}

async function bindWasm()
{
    console.log("Binding WASM to JS using wasm-bindgen...");
    const command = new Deno.Command("wasm-bindgen", {
        args: ["--target", "web", "--out-dir", OUT_DIR, "--out-name", GAME_NAME, UNBOUND_WASM_PATH],
        stdin: "null",
        stdout: "piped",
    });

    const output = await command.output();

    if (!output.success)
    {
        throw execError(
            "running wasm-bindgen",
            "wasm-bindgen",
            output.code,
            output.stdout,
            output.stderr,
        );
    }
}

async function optimizeWasm()
{
    if (!(await isExecAvailable("wasm-opt")))
    {
        console.warn("Could not find optional dependency `wasm-opt`, the build will not be fully optimized");
        console.warn("Make sure you have it installed. To install it on Debian, do `sudo apt install binaryen`");
        console.warn("Make sure it's in your PATH.");
        return;
    }

    console.log("Optimizing the WASM using wasm-opt");

    const tempFile = await Deno.makeTempFile({ dir: OUT_DIR });

    try
    {
        const command = new Deno.Command("wasm-opt", {
            args: [BOUND_WASM_PATH, WASM_OPT_LEVEL, "-o", tempFile],
            stdin: "null",
            stdout: "piped",
        });
        const output = await command.output();

        if (!output.success)
        {
            throw execError(
                "optimizing the WASM",
                "wasm-opt",
                output.code,
                output.stdout,
                output.stderr,
            );
        }

        console.log("Applying the optimized WASM...");
        await fs.move(tempFile, BOUND_WASM_PATH, {
            overwrite: true
        });

        console.log("Finished optimizing the WASM");
    } finally
    {
        if (await fs.exists(tempFile))
            await Deno.remove(tempFile);
    }
}

async function processIndexHtml()
{
    console.log("Processing index.html...");
    const js = getJsString();
    const init = await Deno.readFile(INDEX_HTML_PATH);
    const min = new TextDecoder("utf-8").decode(minifyHtml(init, MINIFY_HTML_CONFIG));
    const inlined = min.replace(
        HTML_TEMPLATE_JS,
        `<script async type="module">${await js}</script>`
    );
    await Deno.writeTextFile(path.join(OUT_DIR, "index.html"), inlined);
    console.log("Processed index.html");
}

async function getJsString(): Promise<string>
{
    console.log("Processing the JS bootstrap...");

    const init = await Deno.readTextFile(MAIN_JS_PATH);
    const wasmStat = await (await Deno.open(BOUND_WASM_PATH)).stat();
    const inlined = init
        .replace(JS_TEMPLATE_WASM_TOTAL_BYTES, wasmStat.size + ";")
        .replace(IMPORT_REGEX, IMPORT_REPLACEMENT);
    const inlinedFile = await Deno.makeTempFile({ dir: OUT_DIR });

    await Deno.writeTextFile(inlinedFile, inlined);

    const externsFile = await Deno.makeTempFile({ dir: OUT_DIR });
    await Deno.writeTextFile(
        externsFile,
        `
        /** @externs */
        /** @returns {Promise<*>} */
        async function ${IMPORT_REPLACEMENT}() {}
        
        /**
         * @param {string} a
         * @param {function(): void} b
         */
        function addEventListener(a, b) {}`
    );

    const compiler = new ClosureCompiler({
        js: [inlinedFile],
        externs: [externsFile],
        compilation_level: "ADVANCED_OPTIMIZATIONS",
        language_in: "ECMASCRIPT_NEXT",
    });
    const compilerProcess: Promise<string> = new Promise((resolve, reject) =>
    {
        compiler.run((exitCode: number, stdout: string, stderr: string) =>
        {
            if (exitCode !== 0)
            {
                reject([exitCode, stdout, stderr]);
            } else
            {
                resolve(stdout);
            }
        });
    });

    let string = "";

    try
    {
        string = (await compilerProcess).replace(IMPORT_REPLACEMENT, "import");
        console.log("Processed the JS bootstrap");
    } catch (e)
    {
        const [exitCode, stdout, stderr] = e as unknown as [number, string, string];
        throw execError(
            "running Closure Compiler",
            "google-closure-compiler",
            exitCode,
            new TextEncoder().encode(stdout),
            new TextEncoder().encode(stderr),
        );
    } finally
    {
        await Promise.all([
            Deno.remove(inlinedFile),
            Deno.remove(externsFile),
        ]);
    }

    return string;
}

async function copyAssets(): Promise<void>
{
    if (await fs.exists(ASSETS_PATH))
    {
        console.log("Copying assets folder...");
        await fs.copy(ASSETS_PATH, OUT_DIR);
        console.log("Finished copying assets folder");
    }

}

async function copyAdditionalFiles()
{
    await Promise.all([copyAssets(), processIndexHtml()]);
}

async function main()
{
    // Prep and checks
    await checkDependencies();
    await Promise.all([prepWasmBindgen, prepRustupTarget]);

    console.log("Clearing and recreating output directory...");
    await fs.ensureDir(OUT_DIR);
    await fs.emptyDir(OUT_DIR);

    await buildWasm();
    await bindWasm();
    if (RELEASE_MODE)
    {
        await optimizeWasm();
    }

    console.log("Copying additional files...");
    await copyAdditionalFiles();
}

await main();
console.log("Done!");