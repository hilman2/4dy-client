/// CSP-Override: passt die Content Security Policy der gehosteten Seite an,
/// damit Tauri-IPC (ipc.localhost, tauri.localhost) funktioniert.
/// Muss als ALLERERSTER initialization_script laufen.
pub fn get_csp_override() -> String {
    r#"
(function() {
  'use strict';

  // CSP Meta-Tags entfernen sobald der DOM da ist
  function removeCSP() {
    document.querySelectorAll('meta[http-equiv="Content-Security-Policy"]').forEach(function(el) {
      console.log('[4dy] CSP Meta-Tag entfernt');
      el.remove();
    });
  }

  // Sofort versuchen (falls <head> schon da)
  removeCSP();

  // MutationObserver für dynamisch hinzugefügte CSP-Tags
  var observer = new MutationObserver(function(mutations) {
    for (var i = 0; i < mutations.length; i++) {
      for (var j = 0; j < mutations[i].addedNodes.length; j++) {
        var node = mutations[i].addedNodes[j];
        if (node.nodeName === 'META' && node.httpEquiv === 'Content-Security-Policy') {
          console.log('[4dy] Dynamisches CSP Meta-Tag entfernt');
          node.remove();
        }
      }
    }
  });

  // Auf <head> warten und beobachten
  if (document.head) {
    observer.observe(document.head, { childList: true });
  }
  if (document.documentElement) {
    observer.observe(document.documentElement, { childList: true, subtree: true });
  }

  // Nochmal nach DOMContentLoaded aufräumen
  document.addEventListener('DOMContentLoaded', function() {
    removeCSP();
    // Observer nach 5s stoppen (CSP-Tags kommen nur beim Laden)
    setTimeout(function() { observer.disconnect(); }, 5000);
  });

  console.log('[4dy] CSP-Override aktiv');
})();
"#
    .to_string()
}

/// Notification-Polyfill: wird als ZWEITES initialization_script geladen,
/// BEVOR irgendein Page-Script läuft. Ersetzt die Web Notification API komplett,
/// da WebView2 sie standardmäßig blockiert.
pub fn get_notification_polyfill() -> String {
    r#"
// Notification-Polyfill für WebView2: läuft vor allen Page-Scripts
(function() {
  'use strict';

  // Komplette Notification-Implementierung
  var _handlers = [];
  var _nextId = 0;

  function TauriNotification(title, options) {
    options = options || {};
    this.title = title;
    this.body = options.body || '';
    this.icon = options.icon || '';
    this.tag = options.tag || '';
    this.data = options.data || null;
    this.silent = options.silent || false;
    this.requireInteraction = options.requireInteraction || false;
    this._id = _nextId++;
    this.onclick = null;
    this.onclose = null;
    this.onerror = null;
    this.onshow = null;

    var self = this;

    // Native Notification via Tauri auslösen
    try {
      if (window.__TAURI__ && window.__TAURI__.core) {
        window.__TAURI__.core.invoke('show_notification', {
          title: title,
          body: options.body || '',
        }).catch(function(e) {
          console.warn('[4dy] Notification fehlgeschlagen:', e);
        });
      }
    } catch(e) {}

    // onshow simulieren
    setTimeout(function() {
      if (self.onshow) self.onshow(new Event('show'));
      _handlers.forEach(function(h) {
        if (h.type === 'show') h.fn.call(self, new Event('show'));
      });
    }, 0);
  }

  TauriNotification.prototype.close = function() {
    if (this.onclose) this.onclose(new Event('close'));
  };

  TauriNotification.prototype.addEventListener = function(type, fn) {
    _handlers.push({type: type, fn: fn});
  };

  TauriNotification.prototype.removeEventListener = function(type, fn) {
    _handlers = _handlers.filter(function(h) {
      return !(h.type === type && h.fn === fn);
    });
  };

  TauriNotification.prototype.dispatchEvent = function(event) {
    _handlers.forEach(function(h) {
      if (h.type === event.type) h.fn(event);
    });
    return true;
  };

  // Statische Properties
  Object.defineProperty(TauriNotification, 'permission', {
    get: function() { return 'granted'; },
    configurable: true
  });

  TauriNotification.requestPermission = function(callback) {
    var result = Promise.resolve('granted');
    if (typeof callback === 'function') callback('granted');
    return result;
  };

  TauriNotification.maxActions = 2;

  // Global ersetzen
  window.Notification = TauriNotification;

  // navigator.permissions.query patchen
  if (navigator.permissions && navigator.permissions.query) {
    var _origQuery = navigator.permissions.query.bind(navigator.permissions);
    navigator.permissions.query = function(desc) {
      if (desc && desc.name === 'notifications') {
        return Promise.resolve({
          state: 'granted',
          status: 'granted',
          onchange: null,
          addEventListener: function() {},
          removeEventListener: function() {},
          dispatchEvent: function() { return true; }
        });
      }
      return _origQuery(desc);
    };
  }

  // ServiceWorker Notification Support
  if (navigator.serviceWorker) {
    try {
      var origSWRProto = ServiceWorkerRegistration.prototype;
      if (origSWRProto && !origSWRProto.__patched_notification) {
        origSWRProto.showNotification = function(title, options) {
          new TauriNotification(title, options);
          return Promise.resolve();
        };
        origSWRProto.getNotifications = function() {
          return Promise.resolve([]);
        };
        origSWRProto.__patched_notification = true;
      }
    } catch(e) {}
  }

  // PushManager patchen: WebView2 unterstützt kein Push API,
  // aber 3CX braucht es für Benachrichtigungen
  try {
    if (typeof PushManager !== 'undefined') {
      // WebView2 unterstützt kein Push: wir liefern eine realistisch
      // aussehende Subscription, damit 3CX den User als "online" führt.
      // Push-Nachrichten werden nicht zugestellt (Endpoint ist fake),
      // aber Notifications laufen über unsere eigene Injection.

      // Zufällige base64url-Strings generieren
      function randomBase64url(bytes) {
        var arr = new Uint8Array(bytes);
        crypto.getRandomValues(arr);
        var str = '';
        for (var i = 0; i < arr.length; i++) str += String.fromCharCode(arr[i]);
        return btoa(str).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
      }

      // Einmal generieren und beibehalten (pro Session)
      var fakeToken = randomBase64url(128);
      var fakeP256dh = randomBase64url(65);
      var fakeAuth = randomBase64url(16);
      var fakeEndpoint = 'https://fcm.googleapis.com/fcm/send/' + fakeToken;

      // p256dh und auth als ArrayBuffer
      function base64urlToBuffer(b64) {
        var str = b64.replace(/-/g, '+').replace(/_/g, '/');
        while (str.length % 4) str += '=';
        var bin = atob(str);
        var buf = new ArrayBuffer(bin.length);
        var view = new Uint8Array(buf);
        for (var i = 0; i < bin.length; i++) view[i] = bin.charCodeAt(i);
        return buf;
      }

      var p256dhBuffer = base64urlToBuffer(fakeP256dh);
      var authBuffer = base64urlToBuffer(fakeAuth);

      var fakeSubscription = {
        endpoint: fakeEndpoint,
        expirationTime: null,
        options: { userVisibleOnly: true },
        getKey: function(name) {
          if (name === 'p256dh') return p256dhBuffer;
          if (name === 'auth') return authBuffer;
          return null;
        },
        toJSON: function() {
          return {
            endpoint: fakeEndpoint,
            expirationTime: null,
            keys: {
              p256dh: fakeP256dh,
              auth: fakeAuth
            }
          };
        },
        unsubscribe: function() { return Promise.resolve(true); }
      };

      PushManager.prototype.subscribe = function(options) {
        console.log('[4dy] PushManager.subscribe → Fake-Subscription geliefert');
        return Promise.resolve(fakeSubscription);
      };
      PushManager.prototype.getSubscription = function() {
        return Promise.resolve(fakeSubscription);
      };
      PushManager.prototype.permissionState = function() {
        return Promise.resolve('granted');
      };
    }
  } catch(e) {}

  console.log('[4dy] Notification-Polyfill aktiv (permission: granted)');
})();
"#
    .to_string()
}

/// Gibt das vollständige Injection-Skript zurück, das in die Web-Client-Webview injiziert wird.
/// Enthält: Selektoren, Dialer, Call-Detection, Call-List Tools, CSS-Injection, Event-Bridge.
pub fn get_injection_script() -> String {
    fourdy_i18n::init();

    let i18n_select_multiple = fourdy_i18n::t("hist.select_multiple");
    let i18n_all = fourdy_i18n::t("hist.all");
    let i18n_selected_count = fourdy_i18n::t("hist.selected_count");
    let i18n_delete_selected = fourdy_i18n::t("hist.delete_selected");
    let i18n_cancel = fourdy_i18n::t("hist.cancel");
    let i18n_confirm_delete = fourdy_i18n::t("hist.confirm_delete");
    let i18n_deleting = fourdy_i18n::t("hist.deleting");
    let i18n_deleted = fourdy_i18n::t("hist.deleted");

    format!(
        r#"
(function() {{
  'use strict';

  // Verhindere doppelte Injection
  if (window.__4DY_CLIENT_INJECTED) return;
  window.__4DY_CLIENT_INJECTED = true;

  console.log('[4dy] Injection wird geladen...');

  // ============================================================
  // SELEKTOREN: bei 3CX-Updates hier anpassen
  // Strategie: ARIA/data-Attribute > Textinhalt > Struktur
  // ============================================================
  const SEL = {{
    dialpad: {{
      input: '[aria-label="Phone number"], input[type="tel"], #phoneInput, input[placeholder*="umber"], input[placeholder*="earch"]',
      dialButton: '[aria-label="Dial"], button[title="Dial"], [data-action="dial"], button[aria-label="Call"]',
    }},
    callControls: {{
      answerButton: '#btnAnswer, [aria-label="Answer"], button[title="Answer"]',
      hangupButton: '#btnReject, #btnDrop, [aria-label="Hang up"], button[title="Hang up"], [aria-label="End call"]',
    }},
    callHistory: {{
      container: '[role="list"], .call-history, #callHistory, [class*="history"], [class*="recents"]',
      entry: '[role="listitem"], .call-history-entry, [class*="history-item"], [class*="recent-item"]',
      deleteButton: '[aria-label="Delete"], button[title="Delete"], [class*="delete"]',
      contactName: '.contact-name, [data-field="name"], [class*="caller-name"], [class*="contact"]',
      phoneNumber: '.phone-number, [data-field="number"], [class*="phone"], [class*="number"]',
    }},
    incomingCall: {{
      notification: '.incoming-call, [data-call-state="ringing"], [class*="incoming"], [class*="ringing"]',
      callerName: '.caller-name, .incoming-call .contact-name, [class*="caller"] [class*="name"]',
      callerNumber: '.caller-number, .incoming-call .phone-number, [class*="caller"] [class*="number"]',
    }},
  }};

  // ============================================================
  // HILFSFUNKTIONEN
  // ============================================================
  function findElement(selectorString) {{
    for (const sel of selectorString.split(', ')) {{
      const el = document.querySelector(sel.trim());
      if (el) return el;
    }}
    return null;
  }}

  function findAllElements(selectorString) {{
    const results = [];
    for (const sel of selectorString.split(', ')) {{
      results.push(...document.querySelectorAll(sel.trim()));
    }}
    return [...new Set(results)];
  }}

  function waitForElement(selectorString, timeout = 10000) {{
    return new Promise((resolve, reject) => {{
      const el = findElement(selectorString);
      if (el) return resolve(el);
      const observer = new MutationObserver(() => {{
        const el = findElement(selectorString);
        if (el) {{
          observer.disconnect();
          resolve(el);
        }}
      }});
      observer.observe(document.body, {{ childList: true, subtree: true }});
      setTimeout(() => {{
        observer.disconnect();
        reject(new Error('Timeout: ' + selectorString));
      }}, timeout);
    }});
  }}

  // ============================================================
  // DIALER: Nummer wählen, Annehmen, Auflegen
  // ============================================================
  window.__4DY_DIAL = function(number) {{
    const input = findElement(SEL.dialpad.input);
    if (!input) {{
      console.warn('[4dy] Wählfeld nicht gefunden');
      return false;
    }}
    // React-kompatibel: nativeInputValueSetter verwenden
    const nativeSetter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype, 'value'
    )?.set;
    if (nativeSetter) {{
      nativeSetter.call(input, number);
    }} else {{
      input.value = number;
    }}
    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
    input.dispatchEvent(new Event('change', {{ bubbles: true }}));

    setTimeout(() => {{
      const dialBtn = findElement(SEL.dialpad.dialButton);
      if (dialBtn) {{
        dialBtn.click();
      }} else {{
        input.dispatchEvent(new KeyboardEvent('keydown', {{
          key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true
        }}));
        input.dispatchEvent(new KeyboardEvent('keyup', {{
          key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true
        }}));
      }}
    }}, 200);
    return true;
  }};

  window.__4DY_ANSWER = function() {{
    const btn = findElement(SEL.callControls.answerButton);
    if (btn) {{ btn.click(); return true; }}
    console.warn('[4dy] Annehmen-Button nicht gefunden');
    return false;
  }};

  window.__4DY_HANGUP = function() {{
    const btn = findElement(SEL.callControls.hangupButton);
    if (btn) {{ btn.click(); return true; }}
    console.warn('[4dy] Auflegen-Button nicht gefunden');
    return false;
  }};

  // ============================================================
  // ANRUFLISTE: Checkboxen + Bulk-Delete
  // Injiziert Checkboxen in jeden Eintrag und eine Toolbar
  // ============================================================
  function setupCallHistoryBulkDelete() {{
    var injected = false;
    var lastCheckedCb = null;

    function injectCheckboxes() {{
      var scrollBox = document.getElementById('scrollBox');
      if (!scrollBox) return;

      var entries = scrollBox.querySelectorAll('call.grid-group-item');
      if (entries.length === 0) return;

      // Nur einmal die Toolbar injizieren
      if (!document.getElementById('tcx-bulk-toolbar')) {{
        var toolbar = document.createElement('div');
        toolbar.id = 'tcx-bulk-toolbar';
        toolbar.style.cssText = 'display:none;padding:6px 10px;background:#1e293b;border-bottom:1px solid #334155;' +
          'position:sticky;top:0;z-index:100;gap:8px;align-items:center;flex-wrap:wrap;';
        toolbar.innerHTML =
          '<label style="color:#94a3b8;font-size:12px;cursor:pointer;display:flex;align-items:center;gap:4px;">' +
            '<input type="checkbox" id="tcx-select-all" style="accent-color:#ef4444;width:16px;height:16px;"> {i18n_all}</label>' +
          '<span id="tcx-selected-count" style="color:#e2e8f0;font-size:12px;margin-left:8px;">0 {i18n_selected_count}</span>' +
          '<button id="tcx-delete-selected" style="margin-left:auto;padding:4px 12px;background:#ef4444;color:#fff;' +
            'border:none;border-radius:4px;font-size:12px;cursor:pointer;">{i18n_delete_selected}</button>' +
          '<button id="tcx-cancel-select" style="padding:4px 8px;background:transparent;color:#94a3b8;' +
            'border:1px solid #475569;border-radius:4px;font-size:12px;cursor:pointer;">{i18n_cancel}</button>';

        var listContainer = scrollBox.closest('.layout-type2-content') || scrollBox.parentElement;
        if (listContainer) {{
          listContainer.insertBefore(toolbar, listContainer.firstChild);
        }}

        // Alle auswählen
        document.getElementById('tcx-select-all').addEventListener('change', function() {{
          var cbs = scrollBox.querySelectorAll('.tcx-cb');
          cbs.forEach(function(cb) {{ cb.checked = this.checked; }}.bind(this));
          updateCount();
        }});

        // Ausgewählte löschen
        document.getElementById('tcx-delete-selected').addEventListener('click', function() {{
          var checked = scrollBox.querySelectorAll('.tcx-cb:checked');
          if (checked.length === 0) return;
          if (!confirm('{i18n_confirm_delete}'.replace('{{count}}', checked.length))) return;
          deleteEntries(Array.from(checked).map(function(cb) {{ return cb.closest('call'); }}));
        }});

        // Abbrechen
        document.getElementById('tcx-cancel-select').addEventListener('click', function() {{
          toggleSelectMode(false);
        }});
      }}

      // Checkboxen in Einträge injizieren
      entries.forEach(function(entry) {{
        if (entry.querySelector('.tcx-cb')) return;
        var cb = document.createElement('input');
        cb.type = 'checkbox';
        cb.className = 'tcx-cb';
        cb.style.cssText = 'display:none;width:18px;height:18px;accent-color:#ef4444;cursor:pointer;' +
          'flex-shrink:0;margin-right:4px;position:absolute;left:4px;top:50%;transform:translateY(-50%);z-index:10;';
        cb.addEventListener('change', updateCount);
        cb.addEventListener('click', function(e) {{
          e.stopPropagation();
          // Shift+Klick: Bereich auswählen
          if (e.shiftKey && lastCheckedCb) {{
            var allCbs = Array.from(document.querySelectorAll('.tcx-cb'));
            var from = allCbs.indexOf(lastCheckedCb);
            var to = allCbs.indexOf(cb);
            if (from > -1 && to > -1) {{
              var start = Math.min(from, to);
              var end = Math.max(from, to);
              for (var j = start; j <= end; j++) {{
                allCbs[j].checked = cb.checked;
              }}
            }}
            updateCount();
          }}
          lastCheckedCb = cb;
        }});
        entry.style.position = 'relative';
        entry.insertBefore(cb, entry.firstChild);
      }});

      injected = true;
    }}

    function toggleSelectMode(on) {{
      var toolbar = document.getElementById('tcx-bulk-toolbar');
      var cbs = document.querySelectorAll('.tcx-cb');
      if (toolbar) toolbar.style.display = on ? 'flex' : 'none';
      cbs.forEach(function(cb) {{
        cb.style.display = on ? 'block' : 'none';
        if (!on) cb.checked = false;
      }});
      // Einträge einrücken damit Checkbox Platz hat
      document.querySelectorAll('call.grid-group-item').forEach(function(entry) {{
        entry.style.paddingLeft = on ? '28px' : '';
      }});
      var selectAll = document.getElementById('tcx-select-all');
      if (selectAll) selectAll.checked = false;
      updateCount();
    }}

    function updateCount() {{
      var count = document.querySelectorAll('.tcx-cb:checked').length;
      var el = document.getElementById('tcx-selected-count');
      if (el) el.textContent = count + ' {i18n_selected_count}';
    }}

    function deleteEntries(entries) {{
      if (entries.length === 0) return;
      var status = document.getElementById('tcx-selected-count');
      var total = entries.length;
      if (status) status.textContent = '{i18n_deleting} 0/' + total + '...';

      // Sequentiell: Delete → Dialog abwarten → OK → nächster
      var idx = 0;
      function deleteNext() {{
        if (idx >= entries.length) {{
          if (status) status.textContent = idx + ' {i18n_deleted}';
          setTimeout(function() {{ toggleSelectMode(false); }}, 800);
          return;
        }}
        var entry = entries[idx];
        if (!entry || !entry.parentElement) {{ idx++; deleteNext(); return; }}

        if (status) status.textContent = (idx + 1) + ' / ' + total + ' ...';

        // 1) Hover + Delete klicken
        entry.dispatchEvent(new MouseEvent('mouseenter', {{ bubbles: true }}));
        setTimeout(function() {{
          var delBtn = entry.querySelector('#deleteClickedBtn');
          if (!delBtn) {{ idx++; deleteNext(); return; }}
          delBtn.click();

          // 2) Auf OK-Button warten und klicken
          waitForOk();
        }}, 100);
      }}

      function waitForOk() {{
        var tries = 0;
        function check() {{
          var okBtn = document.querySelector('[data-qa="modal-ok"]');
          if (okBtn) {{
            okBtn.click();
            // 3) Warten bis Dialog weg ist, dann nächster
            waitForDialogGone();
          }} else {{
            tries++;
            if (tries < 15) setTimeout(check, 100);
            else {{ idx++; deleteNext(); }} // Skip wenn kein Dialog kommt
          }}
        }}
        check();
      }}

      function waitForDialogGone() {{
        var tries = 0;
        function check() {{
          var modal = document.querySelector('[data-qa="modal-ok"]');
          if (!modal) {{
            idx++;
            setTimeout(deleteNext, 50);
          }} else {{
            tries++;
            if (tries < 20) setTimeout(check, 100);
            else {{ idx++; deleteNext(); }}
          }}
        }}
        setTimeout(check, 150);
      }}

      deleteNext();
    }}

    // Rechtsklick-Kontextmenü erweitern: "Mehrere auswählen"
    document.addEventListener('contextmenu', function(e) {{
      var callEntry = e.target.closest('call.grid-group-item');
      if (!callEntry) return;

      // Warten bis das 3CX-Kontextmenü erscheint
      setTimeout(function() {{
        var menu = document.querySelector('.angular2-contextmenu .dropdown-menu');
        if (!menu || menu.querySelector('.tcx-multi-select')) return;

        var separator = document.createElement('div');
        separator.style.cssText = 'border-top:1px solid #475569;margin:4px 0;';
        menu.appendChild(separator);

        var selectItem = document.createElement('a');
        selectItem.href = '';
        selectItem.className = 'dropdown-item tcx-multi-select';
        selectItem.innerHTML = '<span>{i18n_select_multiple}</span>';
        selectItem.addEventListener('click', function(ev) {{
          ev.preventDefault();
          ev.stopPropagation();
          menu.style.display = 'none';
          injectCheckboxes();
          toggleSelectMode(true);
          // Aktuellen Eintrag vorauswählen
          var cb = callEntry.querySelector('.tcx-cb');
          if (cb) {{ cb.checked = true; updateCount(); }}
        }});
        menu.appendChild(selectItem);
      }}, 100);
    }}, true);

    // MutationObserver um neue Einträge zu erkennen (infinite scroll)
    var listObserver = new MutationObserver(function() {{
      if (injected && document.getElementById('tcx-bulk-toolbar')?.style.display === 'flex') {{
        injectCheckboxes();
      }}
    }});
    var scrollBox = document.getElementById('scrollBox');
    if (scrollBox) {{
      listObserver.observe(scrollBox, {{ childList: true, subtree: true }});
    }}

    console.log('[4dy] Anrufliste Bulk-Delete aktiv (Rechtsklick → "Mehrere auswählen")');
  }}

  // ============================================================
  // ANRUFERKENNUNG via 3CX CRM-Integration
  // Statt DOM-Scraping nutzen wir die eingebaute CRM-Integration:
  // 3CX ruft bei Klingeln eine URL auf mit %CallerNumber% und
  // %CallerDisplayName%, wir fangen sie hier ab.
  //
  // Konfiguration in 3CX: Optionen → Integration →
  //   "Kontakt in personalisiertem CRM öffnen"
  //   URL: http://4dy-client.localhost/incoming?number=%CallerNumber%&name=%CallerDisplayName%
  //   Benachrichtigen: Klingeln
  // ============================================================
  function setupCallDetection() {{
    // window.open abfangen (3CX öffnet die CRM-URL in neuem Fenster)
    var origOpen = window.open;
    window.open = function(url) {{
      if (url && typeof url === 'string' && url.indexOf('4dy-client.localhost/incoming') !== -1) {{
        try {{
          var parsed = new URL(url);
          var callerNumber = parsed.searchParams.get('number') || '';
          var callerName = parsed.searchParams.get('name') || 'Unbekannt';
          console.log('[4dy] CRM-Integration Anruf:', callerName, callerNumber);

          if (window.__TAURI__) {{
            window.__TAURI__.core.invoke('on_incoming_call', {{
              callerName: callerName,
              callerNumber: callerNumber,
            }}).catch(function(err) {{
              console.warn('[4dy] Tauri-Invoke Fehler:', err);
            }});
          }}
        }} catch(e) {{
          console.warn('[4dy] CRM-URL Parse-Fehler:', e);
        }}
        return null; // Fenster nicht tatsächlich öffnen
      }}
      return origOpen.apply(this, arguments);
    }};

    // Links mit target="_blank" abfangen
    document.addEventListener('click', function(e) {{
      var link = e.target.closest ? e.target.closest('a') : null;
      if (link && link.href && link.href.indexOf('4dy-client.localhost/incoming') !== -1) {{
        e.preventDefault();
        e.stopPropagation();
        window.open(link.href); // Wird vom Hook oben abgefangen
      }}
    }}, true);

    console.log('[4dy] CRM-Integration Anruferkennung aktiv');
    console.log('[4dy] Konfiguriere in 3CX: Optionen → Integration →');
    console.log('[4dy]   "Kontakt in personalisiertem CRM öffnen"');
    console.log('[4dy]   URL: http://4dy-client.localhost/incoming?number=%CallerNumber%&name=%CallerDisplayName%');
  }}

  // ============================================================
  // ANRUF-ERKENNUNG via Notification-Polyfill + Audio-Erkennung
  // Unser Notification-Polyfill fängt new Notification() ab.
  // Zusätzlich überwachen wir <audio>-Elemente (Klingelton).
  // ============================================================
  function setupRingDetection() {{
    var lastCallNumber = '';

    var observer = new MutationObserver(function(mutations) {{
      for (var m = 0; m < mutations.length; m++) {{
        for (var n = 0; n < mutations[m].addedNodes.length; n++) {{
          var node = mutations[m].addedNodes[n];
          if (node.nodeType !== 1) continue;

          // .call-info Container erkennen
          var callInfo = null;
          if (node.classList && node.classList.contains('call-info')) {{
            callInfo = node;
          }} else if (node.querySelector) {{
            callInfo = node.querySelector('.call-info');
          }}
          if (!callInfo) continue;

          // Nummer aus .callNumber
          var numEl = callInfo.querySelector('.callNumber');
          var number = numEl ? numEl.textContent.trim() : '';
          if (!number) continue;

          // Duplikat-Check
          if (number === lastCallNumber) continue;
          lastCallNumber = number;
          setTimeout(function() {{ lastCallNumber = ''; }}, 5000);

          // Name kommt verzögert (Angular-Rendering) → kurz warten
          (function(ci, num) {{
            setTimeout(function() {{
              // Alle display-names sammeln
              var names = [];
              ci.querySelectorAll('[data-qa="display-name"]').forEach(function(el) {{
                var t = el.textContent.trim();
                if (t) names.push(t);
              }});
              // Fallback: contact-name
              if (names.length === 0) {{
                ci.querySelectorAll('[data-qa="contact-name"]').forEach(function(el) {{
                  var t = el.textContent.trim();
                  if (t) names.push(t);
                }});
              }}
              var name = names.join(', ') || 'Unbekannt';

              console.log('[4dy] Eingehender Anruf erkannt:', name, num);

              if (window.__TAURI__) {{
                window.__TAURI__.core.invoke('show_notification', {{
                  title: name,
                  body: num,
                }}).catch(function(err) {{
                  console.warn('[4dy] Toast-Fehler:', err);
                }});
              }}
            }}, 800);
          }})(callInfo, number);
        }}
      }}
    }});

    observer.observe(document.body, {{ childList: true, subtree: true }});
    console.log('[4dy] Klingel-Erkennung (.call-info) aktiv');
  }}

  // ============================================================
  // VERBUNDEN-ERKENNUNG via DOM (call-view + call-duration)
  // Wenn Anruf angenommen → callback-popup.exe starten (Mailto)
  // ============================================================
  function setupConnectedDetection() {{
    var lastConnectedNumber = '';

    var observer = new MutationObserver(function(mutations) {{
      for (var m = 0; m < mutations.length; m++) {{
        for (var n = 0; n < mutations[m].addedNodes.length; n++) {{
          var node = mutations[m].addedNodes[n];
          if (node.nodeType !== 1) continue;

          // #btnHold erscheint NUR bei verbundenem Anruf
          var holdBtn = null;
          if (node.id === 'btnHold') {{
            holdBtn = node;
          }} else if (node.querySelector) {{
            holdBtn = node.querySelector('#btnHold');
          }}
          if (!holdBtn) continue;

          // Nummer + Name aus .call-info lesen
          setTimeout(function() {{
            var callInfo = document.querySelector('.call-info');
            if (!callInfo) return;

            var numEl = callInfo.querySelector('.callNumber');
            var number = numEl ? numEl.textContent.trim() : '';
            if (!number || number === lastConnectedNumber) return;

            lastConnectedNumber = number;
            setTimeout(function() {{ lastConnectedNumber = ''; }}, 60000);

            var names = [];
            callInfo.querySelectorAll('[data-qa="display-name"]').forEach(function(el) {{
              var t = el.textContent.trim();
              if (t) names.push(t);
            }});
            var name = names.join(', ') || 'Unbekannt';

            console.log('[4dy] Anruf verbunden:', name, number);

            if (window.__TAURI__) {{
              window.__TAURI__.core.invoke('on_incoming_call', {{
                callerName: name,
                callerNumber: number,
              }}).catch(function(err) {{
                console.warn('[4dy] Callback-Popup Fehler:', err);
              }});
            }}
          }}, 500);
        }}
      }}
    }});

    observer.observe(document.body, {{ childList: true, subtree: true }});
    console.log('[4dy] Verbunden-Erkennung (#btnHold) aktiv');
  }}

  // ============================================================
  // CSS-INJECTION
  // ============================================================
  function injectStyles() {{
    const style = document.createElement('style');
    style.id = '4dy-client-styles';
    style.textContent = `
      ::-webkit-scrollbar {{ width: 8px; }}
      ::-webkit-scrollbar-track {{ background: transparent; }}
      ::-webkit-scrollbar-thumb {{ background: rgba(255,255,255,0.2); border-radius: 4px; }}
      ::-webkit-scrollbar-thumb:hover {{ background: rgba(255,255,255,0.3); }}
    `;
    document.head.appendChild(style);
  }}

  // ============================================================
  // INITIALISIERUNG
  // ============================================================

  // Dummy-Invoke um den postMessage-Fallback SOFORT zu aktivieren
  // (erster IPC-Versuch scheitert an CSP, danach geht postMessage)
  if (window.__TAURI__) {{
    window.__TAURI__.core.invoke('get_config').catch(function() {{}});
  }}

  function initAll() {{
    try {{ setupCallDetection(); }} catch(e) {{ console.warn('[4dy] setupCallDetection Fehler:', e); }}
    try {{ setupRingDetection(); }} catch(e) {{ console.warn('[4dy] setupRingDetection Fehler:', e); }}
    try {{ setupConnectedDetection(); }} catch(e) {{ console.warn('[4dy] setupConnectedDetection Fehler:', e); }}
    try {{ setupCallHistoryBulkDelete(); }} catch(e) {{ console.warn('[4dy] setupCallHistoryBulkDelete Fehler:', e); }}
    try {{ injectStyles(); }} catch(e) {{ console.warn('[4dy] injectStyles Fehler:', e); }}
  }}

  if (document.readyState === 'complete' || document.readyState === 'interactive') {{
    initAll();
  }} else {{
    document.addEventListener('DOMContentLoaded', () => {{
      initAll();
    }});
  }}

  console.log('[4dy] Injection geladen ✓');
}})();
"#,
        i18n_select_multiple = i18n_select_multiple,
        i18n_all = i18n_all,
        i18n_selected_count = i18n_selected_count,
        i18n_delete_selected = i18n_delete_selected,
        i18n_cancel = i18n_cancel,
        i18n_confirm_delete = i18n_confirm_delete,
        i18n_deleting = i18n_deleting,
        i18n_deleted = i18n_deleted,
    )
}
