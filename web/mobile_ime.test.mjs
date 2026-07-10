import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const SENTINEL = "\u200b";
const bridgeSource = readFileSync(
  new URL("../src/internal/web_input.js", import.meta.url),
  "utf8",
).replaceAll("export function ", "function ");

class FakeEventTarget {
  constructor() {
    this.listeners = new Map();
  }

  addEventListener(type, listener, options = {}) {
    const listeners = this.listeners.get(type) || [];
    listeners.push({ listener, once: Boolean(options.once) });
    this.listeners.set(type, listeners);
  }

  dispatchEvent(event) {
    event.target ||= this;
    event.currentTarget = this;
    event.defaultPrevented ||= false;
    event.preventDefault ||= function preventDefault() {
      if (this.cancelable) {
        this.defaultPrevented = true;
      }
    };

    for (const entry of [...(this.listeners.get(event.type) || [])]) {
      entry.listener.call(this, event);

      if (entry.once) {
        const listeners = this.listeners.get(event.type) || [];
        this.listeners.set(
          event.type,
          listeners.filter((candidate) => candidate !== entry),
        );
      }
    }

    return !event.defaultPrevented;
  }
}

class FakeCanvas extends FakeEventTarget {
  constructor() {
    super();
    this.keys = [];
  }

  dispatchEvent(event) {
    if (event.type === "keydown") {
      this.keys.push(event.key);
    }

    return super.dispatchEvent(event);
  }

  getBoundingClientRect() {
    return { left: 0, top: 0 };
  }
}

class FakeInput extends FakeEventTarget {
  constructor(document) {
    super();
    this.document = document;
    this.attributes = new Map();
    this.style = {};
    this.value = "";
  }

  setAttribute(name, value) {
    this.attributes.set(name, value);
  }

  removeAttribute(name) {
    this.attributes.delete(name);
  }

  setSelectionRange(start, end) {
    this.selectionStart = start;
    this.selectionEnd = end;
  }

  focus() {
    this.document.activeElement = this;
  }

  blur() {
    if (this.document.activeElement !== this) {
      return;
    }

    this.document.activeElement = this.document.body;
    this.dispatchEvent(fakeEvent("blur"));
  }
}

class FakeDocument extends FakeEventTarget {
  constructor(canvas) {
    super();
    this.canvas = canvas;
    this.documentElement = {};
    this.readyState = "complete";
    this.body = new FakeEventTarget();
    this.body.appendChild = (child) => {
      this.input = child;
    };
    this.activeElement = this.body;
  }

  createElement(name) {
    assert.equal(name, "input");
    return new FakeInput(this);
  }

  querySelector(selector) {
    return selector === "canvas" ? this.canvas : null;
  }
}

class FakeKeyboardEvent {
  constructor(type, init) {
    Object.assign(this, init, { type });
  }
}

const fakeEvent = (type, init = {}) => ({
  bubbles: true,
  cancelable: false,
  composed: true,
  defaultPrevented: false,
  type,
  ...init,
  preventDefault() {
    if (this.cancelable) {
      this.defaultPrevented = true;
    }
  },
});

const delay = () => new Promise((resolve) => setTimeout(resolve, 0));

function createBridge() {
  const canvas = new FakeCanvas();
  const document = new FakeDocument(canvas);
  const visualViewport = new FakeEventTarget();
  const window = new FakeEventTarget();
  const animationFrames = new Map();
  let nextAnimationFrame = 1;

  Object.assign(window, {
    cancelAnimationFrame(handle) {
      animationFrames.delete(handle);
    },
    clearTimeout,
    matchMedia: () => ({ matches: true }),
    queueMicrotask,
    requestAnimationFrame(callback) {
      const handle = nextAnimationFrame++;
      animationFrames.set(handle, callback);
      return handle;
    },
    setTimeout,
    visualViewport,
  });

  const context = vm.createContext({
    AbortController,
    Date,
    KeyboardEvent: FakeKeyboardEvent,
    console,
    document,
    navigator: { maxTouchPoints: 1 },
    window,
  });
  vm.runInContext(bridgeSource, context, { filename: "web/index.html" });
  const api = vm.runInContext(
    "({ hideMobileKeyboard, registerTextRegion, showMobileKeyboard })",
    context,
  );

  return {
    ...api,
    canvas,
    document,
    visualViewport,
    window,
    get input() {
      return document.input;
    },
    flushAnimationFrames() {
      const callbacks = [...animationFrames.values()];
      animationFrames.clear();

      for (const callback of callbacks) {
        callback();
      }
    },
  };
}

test("wasm module installs the bridge lazily and only once", () => {
  const bridge = createBridge();

  assert.equal(bridge.input, undefined);

  bridge.showMobileKeyboard();
  const input = bridge.input;
  bridge.showMobileKeyboard();

  assert.equal(bridge.input, input);
  assert.equal(bridge.document.activeElement, input);
});

test("composition-owned keys stay in the IME and commit exactly once", async () => {
  const bridge = createBridge();
  bridge.showMobileKeyboard();

  bridge.input.dispatchEvent(fakeEvent("compositionstart"));
  bridge.input.dispatchEvent(
    fakeEvent("keydown", {
      cancelable: true,
      isComposing: true,
      key: "Backspace",
      keyCode: 229,
    }),
  );
  bridge.input.dispatchEvent(
    fakeEvent("keydown", {
      cancelable: true,
      isComposing: true,
      key: "Enter",
      keyCode: 229,
    }),
  );
  bridge.input.value = `${SENTINEL}中文`;
  bridge.input.dispatchEvent(
    fakeEvent("input", {
      inputType: "insertCompositionText",
      isComposing: true,
    }),
  );

  assert.deepEqual(bridge.canvas.keys, []);

  bridge.input.dispatchEvent(fakeEvent("compositionend", { data: "中文" }));
  bridge.input.value = `${SENTINEL}中文`;
  bridge.input.dispatchEvent(
    fakeEvent("input", { inputType: "insertFromComposition" }),
  );
  await delay();

  assert.deepEqual(bridge.canvas.keys, ["中", "文"]);
  assert.equal(bridge.input.value, SENTINEL);
});

test("canceling composition does not delete application text", () => {
  const bridge = createBridge();
  bridge.showMobileKeyboard();
  bridge.input.dispatchEvent(fakeEvent("compositionstart"));
  bridge.input.value = "";
  bridge.input.dispatchEvent(fakeEvent("compositionend", { data: "" }));

  assert.deepEqual(bridge.canvas.keys, []);
});

test("composition commit is flushed before a following blur can reset it", () => {
  const bridge = createBridge();
  bridge.showMobileKeyboard();
  bridge.input.dispatchEvent(fakeEvent("compositionstart"));
  bridge.input.value = `${SENTINEL}語`;

  bridge.input.dispatchEvent(fakeEvent("compositionend", { data: "語" }));
  bridge.hideMobileKeyboard();

  assert.deepEqual(bridge.canvas.keys, ["語"]);
});

test("beforeinput forwards soft-keyboard delete and enter actions once", async () => {
  const bridge = createBridge();
  bridge.showMobileKeyboard();

  bridge.input.dispatchEvent(
    fakeEvent("keydown", { cancelable: true, key: "Backspace" }),
  );
  const backward = fakeEvent("beforeinput", {
    cancelable: true,
    inputType: "deleteContentBackward",
  });
  bridge.input.dispatchEvent(backward);

  const forward = fakeEvent("beforeinput", {
    cancelable: true,
    inputType: "deleteContentForward",
  });
  bridge.input.dispatchEvent(forward);

  const enter = fakeEvent("beforeinput", {
    cancelable: true,
    inputType: "insertParagraph",
  });
  bridge.input.dispatchEvent(enter);
  await delay();

  assert(backward.defaultPrevented);
  assert(forward.defaultPrevented);
  assert(enter.defaultPrevented);
  assert.deepEqual(bridge.canvas.keys, ["Backspace", "Delete", "Enter"]);
});

test("non-cancelable mobile deletion suppresses the following input duplicate", async () => {
  const bridge = createBridge();
  bridge.showMobileKeyboard();

  bridge.input.dispatchEvent(
    fakeEvent("beforeinput", {
      inputType: "deleteContentBackward",
    }),
  );
  bridge.input.value = "";
  bridge.input.dispatchEvent(
    fakeEvent("input", { inputType: "deleteContentBackward" }),
  );

  bridge.input.dispatchEvent(
    fakeEvent("beforeinput", {
      inputType: "deleteContentForward",
    }),
  );
  await delay();

  assert.deepEqual(bridge.canvas.keys, ["Backspace", "Delete"]);
});

test("refocus frame does not reset an active composition", () => {
  const bridge = createBridge();
  bridge.showMobileKeyboard();
  bridge.input.dispatchEvent(fakeEvent("compositionstart"));
  bridge.input.value = `${SENTINEL}pin`;

  bridge.flushAnimationFrames();

  assert.equal(bridge.input.value, `${SENTINEL}pin`);
});

test("visual keyboard resize preserves registered text regions", () => {
  const bridge = createBridge();
  bridge.registerTextRegion(0, 0, 100, 100);
  bridge.hideMobileKeyboard();
  bridge.visualViewport.dispatchEvent(fakeEvent("resize"));

  bridge.canvas.dispatchEvent(
    fakeEvent("pointerdown", {
      clientX: 10,
      clientY: 10,
      pointerId: 1,
      pointerType: "touch",
    }),
  );
  bridge.canvas.dispatchEvent(
    fakeEvent("pointerup", {
      clientX: 10,
      clientY: 10,
      pointerId: 1,
      pointerType: "touch",
    }),
  );

  assert.equal(bridge.document.activeElement, bridge.input);
});
