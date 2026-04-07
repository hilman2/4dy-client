/**
 * Web-Client JS-Injection – Einstiegspunkt
 * Wird nach vollständigem Laden des Web-Clients ausgeführt.
 */
import { SELECTORS } from './selectors.js';

(function () {
  'use strict';

  console.log('[4dy] JS-Injection geladen');

  // Warten bis der Web-Client vollständig geladen ist
  function waitForElement(selector, timeout = 10000) {
    return new Promise((resolve, reject) => {
      const el = document.querySelector(selector);
      if (el) return resolve(el);

      const observer = new MutationObserver((mutations, obs) => {
        const el = document.querySelector(selector);
        if (el) {
          obs.disconnect();
          resolve(el);
        }
      });

      observer.observe(document.body, {
        childList: true,
        subtree: true,
      });

      setTimeout(() => {
        observer.disconnect();
        reject(new Error(`Element ${selector} nicht gefunden (Timeout)`));
      }, timeout);
    });
  }

  // Findet ein Element anhand mehrerer Selektoren (Fallback-Kette)
  function findElement(selectorString) {
    const selectors = selectorString.split(', ');
    for (const sel of selectors) {
      const el = document.querySelector(sel.trim());
      if (el) return el;
    }
    return null;
  }

  // Nummer wählen
  window.__4DY_DIAL = function (number) {
    const input = findElement(SELECTORS.dialpad.input);
    if (!input) {
      console.warn('[4dy] Wählfeld nicht gefunden');
      return false;
    }

    // Wert setzen und Input-Event auslösen
    input.value = number;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new Event('change', { bubbles: true }));

    // Wählen-Button klicken
    setTimeout(() => {
      const dialBtn = findElement(SELECTORS.dialpad.dialButton);
      if (dialBtn) {
        dialBtn.click();
      } else {
        // Fallback: Enter drücken
        input.dispatchEvent(
          new KeyboardEvent('keydown', { key: 'Enter', keyCode: 13, bubbles: true })
        );
      }
    }, 100);

    return true;
  };

  // Anruf annehmen
  window.__4DY_ANSWER = function () {
    const btn = findElement(SELECTORS.callControls.answerButton);
    if (btn) {
      btn.click();
      return true;
    }
    console.warn('[4dy] Annehmen-Button nicht gefunden');
    return false;
  };

  // Auflegen
  window.__4DY_HANGUP = function () {
    const btn = findElement(SELECTORS.callControls.hangupButton);
    if (btn) {
      btn.click();
      return true;
    }
    console.warn('[4dy] Auflegen-Button nicht gefunden');
    return false;
  };

  // Eingehenden Anruf erkennen (MutationObserver)
  function setupCallDetection() {
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of mutation.addedNodes) {
          if (node.nodeType !== Node.ELEMENT_NODE) continue;

          // Prüfen ob es eine Anruf-Benachrichtigung ist
          const selectors = SELECTORS.incomingCall.notification.split(', ');
          for (const sel of selectors) {
            if (node.matches?.(sel) || node.querySelector?.(sel)) {
              const callEl = node.matches?.(sel) ? node : node.querySelector(sel);
              if (callEl) {
                const name =
                  callEl.querySelector(
                    SELECTORS.incomingCall.callerName
                      .split(', ')
                      .join(', ')
                  )?.textContent?.trim() || 'Unbekannt';

                const number =
                  callEl.querySelector(
                    SELECTORS.incomingCall.callerNumber
                      .split(', ')
                      .join(', ')
                  )?.textContent?.trim() || '';

                // Event an Tauri senden
                if (window.__TAURI__) {
                  window.__TAURI__.core.invoke('__cmd__on_incoming_call', {
                    name,
                    number,
                  }).catch(() => {});
                }

                window.dispatchEvent(
                  new CustomEvent('4dy:incoming-call', {
                    detail: { name, number },
                  })
                );
              }
            }
          }
        }
      }
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });
  }

  // Initialisierung
  if (document.readyState === 'complete') {
    setupCallDetection();
  } else {
    window.addEventListener('load', setupCallDetection);
  }

  console.log('[4dy] JS-Injection initialisiert');
})();
