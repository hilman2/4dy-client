/**
 * DOM-Selektoren für den gehosteten Web-Client.
 * Diese Datei wird angepasst, wenn sich die Web-Client-DOM-Struktur ändert.
 * Strategie: ARIA/data-Attribute > Textinhalt > DOM-Struktur
 * NIEMALS minifizierte Klassennamen verwenden.
 */
export const SELECTORS = {
  // Wählfeld
  dialpad: {
    input: '[aria-label="Phone number"], input[type="tel"], #phoneInput',
    dialButton: '[aria-label="Dial"], button[title="Dial"], [data-action="dial"]',
  },

  // Anruf-Steuerung
  callControls: {
    answerButton: '[aria-label="Answer"], button[title="Answer"], [data-action="answer"]',
    hangupButton: '[aria-label="Hang up"], button[title="Hang up"], [data-action="hangup"], [aria-label="End call"]',
  },

  // Anrufliste
  callHistory: {
    container: '[role="list"], .call-history, #callHistory',
    entry: '[role="listitem"], .call-history-entry',
    deleteButton: '[aria-label="Delete"], button[title="Delete"]',
    contactName: '.contact-name, [data-field="name"]',
    phoneNumber: '.phone-number, [data-field="number"]',
  },

  // Eingehender Anruf
  incomingCall: {
    notification: '.incoming-call, [data-call-state="ringing"]',
    callerName: '.caller-name, .incoming-call .contact-name',
    callerNumber: '.caller-number, .incoming-call .phone-number',
  },
};
