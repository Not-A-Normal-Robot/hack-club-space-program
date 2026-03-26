#!/usr/bin/env -S deno -P
"use strict";

if (!("Deno" in globalThis))
{
    throw new Error("Deno object not found; please run this script using Deno. You can get deno at https://deno.land/");
}

// Paths
const DIRNAME = import.meta.dirname;
if (!DIRNAME)
{
    throw new Error("`import.meta.dirname` is undefined. This script must be run locally.");
}

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
    stdout: Uint8Array | null,
    stderr: Uint8Array | null,
): Error
{
    const decoder = new TextDecoder("utf-8");
    return new Error(
        `An error occurred while ${action}:\n` +
        `'${cmd}' returned with exit code ${code}\n\n` +
        (stdout ? `===== ${cmd} stdout =====\n\n${decoder.decode(stdout)}` : "") +
        (stderr ? `===== ${cmd} stderr =====\n\n${decoder.decode(stderr)}` : "")
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

async function prepWasmPack()
{
    if (!await isExecAvailable("wasm-pack"))
    {
        console.log("wasm-pack not found, installing...");
        const command = new Deno.Command("cargo", {
            args: ["install", "wasm-pack"],
            stdout: "piped",
        });
        const output = await command.output();
        if (output.success)
        {
            console.log("Finished installing wasm-pack");
        } else
        {
            throw new Error(
                "Error occurred installing wasm-pack\n\n" +
                `===== cargo stdout =====\n${output.stdout}\n\n` +
                `===== cargo stderr =====\n${output.stderr}`
            );
        }
    }
    if (!await isExecAvailable("wasm-pack"))
    {
        throw new Error("Could not access wasm-pack after installation, something must have gone awry");
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

async function main()
{
    // Prep and checks
    await checkDependencies();
    await Promise.all([prepWasmPack, prepRustupTarget]);

    await new Deno.Command(
        "wasm-pack",
        {
            args: ["test", "--headless", "--firefox", ...Deno.args],
            cwd: DIRNAME,
            env: {
                WASM_BINDGEN_USE_DENO: "1",
                WASM_BINDGEN_TEST_TIMEOUT: "10000000",
            },
            stdout: "inherit",
            stderr: "inherit"
        }
    ).output();
}

await main();