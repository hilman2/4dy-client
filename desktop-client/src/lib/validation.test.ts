import { describe, it, expect } from "vitest";
import { validatePbxUrl } from "./validation";

describe("validatePbxUrl", () => {
  it("rejects an empty string", () => {
    const result = validatePbxUrl("");
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).toBe("Bitte eine URL eingeben.");
    }
  });

  it("rejects whitespace-only input", () => {
    const result = validatePbxUrl("   \t  ");
    expect(result.ok).toBe(false);
  });

  it("rejects garbage that is not a URL", () => {
    const result = validatePbxUrl("not-a-url");
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).toContain("https://");
    }
  });

  it("accepts a typical https PBX URL", () => {
    const result = validatePbxUrl("https://meine-firma.3cx.de");
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe("https://meine-firma.3cx.de");
    }
  });

  it("accepts http URLs as well", () => {
    const result = validatePbxUrl("http://localhost:5001");
    expect(result.ok).toBe(true);
  });

  it("trims surrounding whitespace before returning the value", () => {
    const result = validatePbxUrl("   https://example.3cx.de   ");
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe("https://example.3cx.de");
    }
  });

  it("accepts URLs with port and path", () => {
    const result = validatePbxUrl("https://pbx.example.com:8443/webclient");
    expect(result.ok).toBe(true);
  });
});
