/**
 * Pure-logic helpers for the setup wizard.
 *
 * Kept free of Svelte runes and Tauri imports so it can be unit-tested
 * with vitest in a plain jsdom environment.
 */

export type ValidationResult =
  | { ok: true; value: string }
  | { ok: false; error: string };

/**
 * Validates a PBX URL entered in the setup wizard.
 * Trims whitespace, requires a non-empty input, and rejects strings
 * that the WHATWG `URL` parser cannot parse.
 */
export function validatePbxUrl(input: string): ValidationResult {
  const trimmed = input.trim();

  if (!trimmed) {
    return { ok: false, error: "Bitte eine URL eingeben." };
  }

  try {
    // eslint-disable-next-line no-new
    new URL(trimmed);
  } catch {
    return {
      ok: false,
      error: "Bitte eine gültige URL eingeben (z.B. https://meine-firma.3cx.de)",
    };
  }

  return { ok: true, value: trimmed };
}
