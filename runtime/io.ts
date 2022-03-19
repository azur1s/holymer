import { writeAllSync } from "https://deno.land/std@0.129.0/streams/conversion.ts";

/**
 * Writes text to the stdout stream.
 * @param text The text to write. Can be any type.
 */
export function write(text: any): void {
    const bytes: Uint8Array = new TextEncoder().encode(text);
    writeAllSync(Deno.stdout, bytes);
}

/**
 * Read text from the stdin stream.
 * @param prompt_str The prompt string to display.
 * @returns The text read from the stdin stream.
 */
export function read(prompt_str: string): string {
    const input = prompt(prompt_str, "");
    return input ? input : "";
}

/**
 * Writes text to the file.
 * @param text The text to write. Can be any type.
 * @param path The path to the file.
 */
export function writeFile(text: any, path: string): void {
    const bytes: Uint8Array = new TextEncoder().encode(text);
    Deno.writeFileSync(path, bytes);
}

/**
 * Read text from the file.
 * @param path The path to the file.
 * @returns The text read from the file.
 */
export function readFile(path: string): string {
    const decoder = new TextDecoder("utf-8");
    const text = decoder.decode(Deno.readFileSync(path));
    return text;
}