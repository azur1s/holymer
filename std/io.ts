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
 * Writes text to the file.
 * @param text The text to write. Can be any type.
 * @param path The path to the file.
 */
export function writeFile(text: any, path: string): void {
    const bytes: Uint8Array = new TextEncoder().encode(text);
    Deno.writeFileSync(path, bytes);
}