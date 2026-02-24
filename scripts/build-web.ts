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
type Channel = "stable" | "nightly";

// Paths
const DIRNAME = import.meta.dirname;
if (!DIRNAME)
{
    throw new Error("`import.meta.dirname` is undefined. This script must be run locally.");
}
const TARGET_DIR = path.join(DIRNAME, "../target/wasm32-unknown-unknown");

const OUT_DIR = path.join(TARGET_DIR, "web");
function wasmDir(bounded: boolean, multithreaded: boolean)
{
    return bounded ?
        path.join(
            OUT_DIR,
            (multithreaded ? "multithreaded" : "singlethreaded"),
        ) : path.join(
            TARGET_DIR,
            (RELEASE_MODE ? "release" : "debug") + (multithreaded ? "-multithreaded" : ""),
        );
}
function wasmPath(bounded: boolean, multithreaded: boolean)
{
    return path.join(
        wasmDir(bounded, multithreaded),
        GAME_NAME + (bounded ? "_bg" : "") + ".wasm"
    );
}
const REPO_ASSETS_PATH = path.join(DIRNAME, "../assets");
const SRC_INDEX_HTML_PATH = path.join(DIRNAME, "../web/index.html");
const SRC_JS_PATH = path.join(DIRNAME, "../web/main.js");

// Web file processing
const TEMPLATES = {
    JS: {
        SINGLETHREADED_WASM_BYTES: "/*{{INLINER:WASM_SINGLE_BYTES}}*/",
        MULTITHREADED_WASM_BYTES: "/*{{INLINER:WASM_MULTI_BYTES}}*/",
        DYNAMIC_IMPORT: /\bimport\b(?=\s*\()/,
        IMPORT_REPLACEMENT: "__Wk2hQs1ynle8rKILAmsDoYFD__",
    },
    HTML: {
        INLINE_JS: "{{mainscript}}",
    }
};
const MINIFY_HTML_CONFIG = {
    allow_noncompliant_unquoted_attribute_values: true,
    allow_optimal_entities: true,
    allow_removing_spaces_between_attributes: true,
    minify_css: true,
    minify_doctype: true,
    preserve_brace_template_syntax: true,
};
const WASM_OPT_LEVEL = "-O4";

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

async function installRustupTarget(channel: Channel)
{
    const installCommand = new Deno.Command("rustup", {
        args: ["target", "install", "wasm32-unknown-unknown", "--toolchain", channel],
        stdin: "null",
        stdout: "piped",
    });
    const installOutput = await installCommand.output();

    if (!installOutput.success)
    {
        throw execError(
            `installing the ${channel} wasm32-unknown-unknown rustup target`,
            "rustup",
            installOutput.code,
            installOutput.stdout,
            installOutput.stderr,
        );
    }

    console.log(`Installed ${channel} wasm32-unknown-unknown rustup target`);
}

async function installNightlyRustSrc()
{
    const installCommand = new Deno.Command("rustup", {
        args: ["component", "add", "rust-src", "--toolchain", "nightly"],
        stdin: "null",
        stdout: "piped",
    });
    const installOutput = await installCommand.output();

    if (!installOutput.success)
    {
        throw execError(
            `installing the nightly rust-src rustup component`,
            "rustup",
            installOutput.code,
            installOutput.stdout,
            installOutput.stderr,
        );
    }

    console.log("Installed the nightly rust-src rustup component");
}

async function prepRustupTarget(channel: Channel)
{
    console.log(`Checking list of installed ${channel} rustup targets`);

    const listCommand = new Deno.Command("rustup", {
        args: ["target", "list", "--installed", "--toolchain", channel],
        stdin: "null",
        stdout: "piped"
    });
    const listOutput = await listCommand.output();
    if (!listOutput.success)
    {
        throw execError(
            `checking rustup's installed ${channel} targets list`,
            "rustup",
            listOutput.code,
            listOutput.stdout,
            listOutput.stderr,
        );
    }

    const list = new TextDecoder().decode(listOutput.stdout)
        .split("\n")
        .map(s => s.trim());

    if (!list.includes("wasm32-unknown-unknown"))
    {
        console.log(`rustup ${channel}-wasm32-unknown-unknown target is not installed, installing...`);
        await installRustupTarget(channel);
    } else
    {
        console.log(`No need to install ${channel}-wasm32-unknown-unknown as it's already installed`);
    }

    if (channel === "nightly")
    {
        console.log("Checking nightly components");

        const componentsCmd = new Deno.Command("rustup", {
            args: ["component", "list", "--installed", "--toolchain", "nightly"],
            stdin: "null",
            stdout: "piped",
        });

        const componentsOutput = await componentsCmd.output();

        if (!componentsOutput.success)
        {
            throw execError(
                "checking rustup's installed nightly components",
                "rustup",
                componentsOutput.code,
                componentsOutput.stdout,
                componentsOutput.stderr,
            );
        }

        const componentsList = new TextDecoder().decode(componentsOutput.stdout)
            .split("\n")
            .map(s => s.trim());

        if (componentsList.includes("rust-src"))
        {
            console.log("No need to intsall nightly rust-src as it's already installed");
        } else
        {
            console.log("Nightly rust-src not found, installing");
            await installNightlyRustSrc();
            return;
        }
    }
}

async function prepRustup()
{
    console.log("Preparing Rustup channels");
    await Promise.all([
        prepRustupTarget("stable"),
        prepRustupTarget("nightly")
    ]);
}

async function buildWasm(multithreaded: boolean)
{
    const args = [
        multithreaded ? "+nightly" : "+stable",
        "build",
        "--target",
        "wasm32-unknown-unknown"
    ];

    if (multithreaded)
    {
        args.push("-Z", "build-std=panic_abort,std");
    }

    if (RELEASE_MODE)
    {
        args.push(...(multithreaded ? ["--profile", "release-multithreaded"] : ["--release"]));
    } else if (multithreaded)
    {
        args.push("--profile", "debug-multithreaded");
    }

    const RUSTFLAGS = multithreaded ? "-C target-feature=+atomics,+bulk-memory,+mutable-globals" : "";

    console.log(`Compiling the Rust codebase to WASM (${multithreaded ? "multithreaded" : "single-threaded"} binary)...`);

    const command = new Deno.Command("cargo", {
        args,
        stdin: "null",
        stdout: "piped",
        env: { RUSTFLAGS },
    });

    const output = await command.output();

    if (!output.success)
    {
        throw execError(
            `compiling Rust to ${multithreaded ? "multithreaded" : "singlethreaded"} WASM`,
            "cargo",
            output.code,
            output.stdout,
            output.stderr,
        );
    }

    console.log(`${multithreaded ? "Multithreaded" : "Singlethreaded"} Rust to WASM compilation complete!`);
}

async function bindWasm(multithreaded: boolean)
{
    console.log(`Binding ${multithreaded ? "multithreaded" : "single-threaded"} WASM to JS using wasm-bindgen...`);

    const args = ["--target", "web", "--no-typescript", "--out-dir", wasmDir(true, multithreaded), "--out-name", GAME_NAME, wasmPath(false, multithreaded)];

    const command = new Deno.Command("wasm-bindgen", {
        args,
        stdin: "null",
        stdout: "piped",
    });

    const output = await command.output();

    if (!output.success)
    {
        throw execError(
            `running ${multithreaded ? "multithreaded" : "single-threaded"} wasm-bindgen`,
            "wasm-bindgen",
            output.code,
            output.stdout,
            output.stderr,
        );
    }

    console.log(`${multithreaded ? "Multithreaded" : "Single-threaded"} WASM bound!`);
}

async function optimizeWasm(multithreaded: boolean)
{
    if (!(await isExecAvailable("wasm-opt")))
    {
        console.warn("Could not find optional dependency `wasm-opt`, the build will not be fully optimized");
        console.warn("Make sure you have it installed. To install it on Debian, do `sudo apt install binaryen`");
        console.warn("Make sure it's in your PATH.");
        return;
    }

    console.log(`Optimizing the ${multithreaded ? "multithreaded" : "single-threaded"} WASM using wasm-opt`);

    const tempFile = await Deno.makeTempFile({ dir: OUT_DIR });

    try
    {
        const command = new Deno.Command("wasm-opt", {
            args: [wasmPath(true, multithreaded), WASM_OPT_LEVEL, "-o", tempFile],
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
        await fs.move(tempFile, wasmPath(true, multithreaded), {
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
    const init = await Deno.readFile(SRC_INDEX_HTML_PATH);
    const min = new TextDecoder("utf-8").decode(minifyHtml(init, MINIFY_HTML_CONFIG));
    const inlined = min.replace(
        TEMPLATES.HTML.INLINE_JS,
        `<script async type="module">${await js}</script>`
    );
    await Deno.writeTextFile(path.join(OUT_DIR, "index.html"), inlined);
    console.log("Processed index.html");
}

async function getJsString(): Promise<string>
{
    console.log("Processing the JS bootstrap...");

    const [singleWasmStat, multiWasmStat] = await Promise.all(
        [false, true].map(
            multithreaded => Deno.open(wasmPath(true, multithreaded))
                .then(p => p.stat())
        )
    );

    const init = await Deno.readTextFile(SRC_JS_PATH);
    const inlined = init
        .replace(TEMPLATES.JS.SINGLETHREADED_WASM_BYTES, singleWasmStat.size + ";")
        .replace(TEMPLATES.JS.MULTITHREADED_WASM_BYTES, multiWasmStat.size + ";")
        .replace(TEMPLATES.JS.DYNAMIC_IMPORT, TEMPLATES.JS.IMPORT_REPLACEMENT);
    const inlinedFile = await Deno.makeTempFile({ dir: OUT_DIR });

    await Deno.writeTextFile(inlinedFile, inlined);

    const externsFile = await Deno.makeTempFile({ dir: OUT_DIR });
    await Deno.writeTextFile(
        externsFile,
        `
        /** @externs */
        /** @returns {Promise<*>} */
        async function ${TEMPLATES.JS.IMPORT_REPLACEMENT}() {}
        
        /**
         * @param {string} a
         * @param {function(): void} b
         */
        function addEventListener(a, b) {}
        
        /** @param {{module_or_path: WebAssembly.Module}} a */
        async function wbg(a) {}`
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
        string = (await compilerProcess).replace(TEMPLATES.JS.IMPORT_REPLACEMENT, "import");
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
    if (await fs.exists(REPO_ASSETS_PATH))
    {
        console.log("Copying assets folder...");
        await fs.copy(REPO_ASSETS_PATH, OUT_DIR);
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
    await Promise.all([prepWasmBindgen(), prepRustup()]);

    console.log("Clearing and recreating output directory...");
    await fs.emptyDir(OUT_DIR);
    await Promise.all([false, true].map(x => fs.emptyDir(wasmDir(true, x))));

    await Promise.all([false, true].map(async multithreaded =>
    {
        await buildWasm(multithreaded);
        await bindWasm(multithreaded);
        if (RELEASE_MODE)
        {
            await optimizeWasm(multithreaded);
        }
    }));

    console.log("Copying additional files...");
    await copyAdditionalFiles();
}

await main();
console.log("Done!");