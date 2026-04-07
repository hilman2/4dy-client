import { describe, it, expect } from "vitest";
import { SELECTORS } from "./selectors.js";

describe("SELECTORS", () => {
  it("exposes the expected top-level groups", () => {
    expect(SELECTORS).toHaveProperty("dialpad");
    expect(SELECTORS).toHaveProperty("callControls");
    expect(SELECTORS).toHaveProperty("callHistory");
    expect(SELECTORS).toHaveProperty("incomingCall");
  });

  it("dialpad selectors are non-empty strings", () => {
    expect(typeof SELECTORS.dialpad.input).toBe("string");
    expect(SELECTORS.dialpad.input.length).toBeGreaterThan(0);
    expect(typeof SELECTORS.dialpad.dialButton).toBe("string");
    expect(SELECTORS.dialpad.dialButton.length).toBeGreaterThan(0);
  });

  it("call control selectors are non-empty strings", () => {
    expect(typeof SELECTORS.callControls.answerButton).toBe("string");
    expect(SELECTORS.callControls.answerButton.length).toBeGreaterThan(0);
    expect(typeof SELECTORS.callControls.hangupButton).toBe("string");
    expect(SELECTORS.callControls.hangupButton.length).toBeGreaterThan(0);
  });

  it("call history selectors are non-empty strings", () => {
    expect(typeof SELECTORS.callHistory.container).toBe("string");
    expect(typeof SELECTORS.callHistory.entry).toBe("string");
    expect(typeof SELECTORS.callHistory.deleteButton).toBe("string");
    expect(typeof SELECTORS.callHistory.contactName).toBe("string");
    expect(typeof SELECTORS.callHistory.phoneNumber).toBe("string");
  });

  it("incoming call selectors are non-empty strings", () => {
    expect(typeof SELECTORS.incomingCall.notification).toBe("string");
    expect(typeof SELECTORS.incomingCall.callerName).toBe("string");
    expect(typeof SELECTORS.incomingCall.callerNumber).toBe("string");
  });

  it("each selector is parsable as a comma-separated query selector list", () => {
    // jsdom is the test environment, so document.querySelector exists.
    // querySelector throws SyntaxError on invalid CSS, which is what we want
    // to catch here — typo'd selectors must fail at test time, not at runtime.
    const collect = (obj) =>
      Object.values(obj).flatMap((v) =>
        typeof v === "string" ? [v] : collect(v),
      );

    for (const sel of collect(SELECTORS)) {
      expect(() => document.querySelector(sel)).not.toThrow();
    }
  });
});
