let bridge = null;

function mobileInput() {
  if (bridge) {
    return bridge;
  }
  const SENTINEL = "\u200b";
  const REFOCUS_AFTER_SHOW_MS = 250;
  const TEXT_REGION_TOUCH_SLOP = 8;
  let input = null;
  let active = false;
  let composing = false;
  let deactivating = false;
  let pendingAction = null;
  let pendingActionHandle = 0;
  let suppressInput = false;
  let suppressInputHandle = 0;
  let refocusFrame = 0;
  let lastShowAt = 0;
  let textRegions = [];
  let textRegionBatchOpen = false;
  let touchGesture = null;
  let textActivation = null;
  let bridgedCanvas = null;
  let bridgeAbort = null;

  const touchKeyboard = () =>
    navigator.maxTouchPoints > 0 ||
    window.matchMedia("(pointer: coarse)").matches;

  const canvas = () => document.querySelector("canvas");

  const hideInputFromAssistiveTechnology = () => {
    if (!input) {
      return;
    }

    input.setAttribute("aria-hidden", "true");
    input.setAttribute("inert", "");
  };

  const showInputToAssistiveTechnology = () => {
    if (!input) {
      return;
    }

    input.removeAttribute("aria-hidden");
    input.removeAttribute("inert");
  };

  const codeFor = (key) => {
    if (key.length === 1) {
      const upper = key.toUpperCase();

      if (/^[A-Z]$/.test(upper)) {
        return `Key${upper}`;
      }

      if (/^[0-9]$/.test(key)) {
        return `Digit${key}`;
      }
    }

    return {
      " ": "Space",
      ArrowDown: "ArrowDown",
      ArrowLeft: "ArrowLeft",
      ArrowRight: "ArrowRight",
      ArrowUp: "ArrowUp",
      Backspace: "Backspace",
      Delete: "Delete",
      End: "End",
      Enter: "Enter",
      Escape: "Escape",
      Home: "Home",
      Tab: "Tab",
    }[key] || "";
  };

  const sendKey = (key) => {
    const target = canvas();

    if (!target) {
      return;
    }

    target.dispatchEvent(
      new KeyboardEvent("keydown", {
        key,
        code: codeFor(key),
        bubbles: true,
        cancelable: true,
        composed: true,
      }),
    );
  };

  const clearPendingAction = () => {
    if (pendingActionHandle) {
      window.clearTimeout(pendingActionHandle);
      pendingActionHandle = 0;
    }

    pendingAction = null;
  };

  const clearSuppressedInput = () => {
    if (suppressInputHandle) {
      window.clearTimeout(suppressInputHandle);
      suppressInputHandle = 0;
    }

    suppressInput = false;
  };

  const suppressFollowingInput = () => {
    clearSuppressedInput();
    suppressInput = true;
    suppressInputHandle = window.setTimeout(() => {
      suppressInput = false;
      suppressInputHandle = 0;
    }, 0);
  };

  const clearRefocus = () => {
    if (refocusFrame) {
      window.cancelAnimationFrame(refocusFrame);
      refocusFrame = 0;
    }
  };

  const resetInput = () => {
    if (!input || composing) {
      return;
    }

    input.value = SENTINEL;

    try {
      input.setSelectionRange(SENTINEL.length, SENTINEL.length);
    } catch (_) {}
  };

  const focusInput = () => {
    if (!input) {
      return;
    }

    showInputToAssistiveTechnology();

    if (document.activeElement !== input) {
      resetInput();
      input.focus({ preventScroll: true });
    }
  };

  const activateInput = () => {
    if (!ensureInput()) {
      return;
    }

    if (!active) {
      clearSuppressedInput();
    }

    active = true;
    lastShowAt = Date.now();
    clearRefocus();
    focusInput();
  };

  const deactivateInput = () => {
    deactivating = true;
    lastShowAt = 0;
    clearRefocus();
    clearPendingAction();
    clearSuppressedInput();

    if (!input) {
      active = false;
      composing = false;
      deactivating = false;
      return;
    }

    input.blur();
    active = false;
    composing = false;
    resetInput();
    hideInputFromAssistiveTechnology();
    deactivating = false;
  };

  const scheduleRefocus = () => {
    clearRefocus();

    refocusFrame = window.requestAnimationFrame(() => {
      refocusFrame = 0;

      const target = canvas();
      const focused = document.activeElement;
      if (
        !active ||
        !input ||
        focused === input ||
        (focused &&
          focused !== target &&
          focused !== document.body &&
          focused !== document.documentElement)
      ) {
        return;
      }

      focusInput();
    });
  };

  const sendText = (text) => {
    for (const char of text) {
      sendKey(char);
    }
  };

  const inputAction = (inputType) => {
    if (inputType === "insertLineBreak" || inputType === "insertParagraph") {
      return "Enter";
    }

    if (inputType?.startsWith("delete") && inputType.endsWith("Backward")) {
      return "Backspace";
    }

    if (inputType?.startsWith("delete") && inputType.endsWith("Forward")) {
      return "Delete";
    }

    return null;
  };

  const compositionInput = (event) =>
    event?.isComposing ||
    event?.inputType === "insertCompositionText" ||
    event?.inputType === "deleteCompositionText";

  const handleInput = (event) => {
    if (!input || !active || composing || compositionInput(event)) {
      return;
    }

    if (suppressInput) {
      clearSuppressedInput();
      resetInput();
      return;
    }

    const value = input.value;

    if (value === SENTINEL) {
      if (pendingAction) {
        const action = pendingAction;
        clearPendingAction();
        sendKey(action);
        resetInput();
      }

      return;
    }

    if (value === "") {
      sendKey(pendingAction || inputAction(event?.inputType) || "Backspace");
      clearPendingAction();
      resetInput();
      return;
    }

    const text = value.startsWith(SENTINEL)
      ? value.slice(SENTINEL.length)
      : value;

    clearPendingAction();

    if (text) {
      sendText(text);
    }

    resetInput();
  };

  const schedulePendingAction = (action) => {
    clearPendingAction();
    pendingAction = action;
    pendingActionHandle = window.setTimeout(() => {
      pendingActionHandle = 0;

      if (active && !composing && pendingAction === action) {
        pendingAction = null;
        sendKey(action);
        resetInput();
      }
    }, 0);
  };

  const canvasPoint = (event) => {
    const target = canvas();

    if (!target) {
      return null;
    }

    const rect = target.getBoundingClientRect();
    return {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
    };
  };

  const pointInTextRegion = (point) =>
    textRegions.some(
      (region) =>
        point.x >= region.x &&
        point.x <= region.x + region.width &&
        point.y >= region.y &&
        point.y <= region.y + region.height,
    );

  const isTouchPointer = (event) =>
    event.pointerType === "touch" ||
    event.pointerType === "pen" ||
    (event.pointerType === "" && touchKeyboard());

  const textActivationMatches = (event) =>
    textActivation && textActivation.pointerId === event.pointerId;

  const touchGestureMatches = (event) =>
    touchGesture && touchGesture.pointerId === event.pointerId;

  const invalidateTextRegions = () => {
    textRegions = [];
    textRegionBatchOpen = false;
    textActivation = null;
  };

  const beginTextActivation = (event) => {
    if (!touchKeyboard() || !isTouchPointer(event)) {
      textActivation = null;
      touchGesture = null;
      return;
    }

    const point = canvasPoint(event);
    touchGesture = point
      ? {
          pointerId: event.pointerId,
          x: point.x,
          y: point.y,
        }
      : null;

    if (!point || !pointInTextRegion(point)) {
      textActivation = null;
      deactivateInput();
      return;
    }

    if (!ensureInput()) {
      textActivation = null;
      return;
    }

    textActivation = {
      pointerId: event.pointerId,
      x: point.x,
      y: point.y,
    };
  };

  const moveTextActivation = (event) => {
    const point = canvasPoint(event);

    if (touchGestureMatches(event)) {
      if (!point) {
        touchGesture = null;
        invalidateTextRegions();
        deactivateInput();
        return;
      }

      const dx = point.x - touchGesture.x;
      const dy = point.y - touchGesture.y;

      if (
        dx * dx + dy * dy >
        TEXT_REGION_TOUCH_SLOP * TEXT_REGION_TOUCH_SLOP
      ) {
        touchGesture = null;
        invalidateTextRegions();
        deactivateInput();
        return;
      }
    }

    if (!textActivationMatches(event)) {
      return;
    }

    if (!point) {
      textActivation = null;
      return;
    }

    const dx = point.x - textActivation.x;
    const dy = point.y - textActivation.y;

    if (dx * dx + dy * dy > TEXT_REGION_TOUCH_SLOP * TEXT_REGION_TOUCH_SLOP) {
      textActivation = null;
    }
  };

  const finishTextActivation = (event) => {
    if (!textActivationMatches(event)) {
      return;
    }

    const point = canvasPoint(event);
    textActivation = null;
    if (touchGestureMatches(event)) {
      touchGesture = null;
    }

    if (!point || !pointInTextRegion(point)) {
      return;
    }

    activateInput();
  };

  const installCanvasActivationBridge = () => {
    const target = canvas();

    if (!target || target === bridgedCanvas) {
      return;
    }

    if (bridgeAbort) {
      bridgeAbort.abort();
    }

    const controller = new AbortController();
    const options = {
      capture: true,
      passive: true,
      signal: controller.signal,
    };

    target.addEventListener("pointerdown", beginTextActivation, options);
    target.addEventListener("pointermove", moveTextActivation, options);
    target.addEventListener("pointerup", finishTextActivation, options);
    target.addEventListener(
      "wheel",
      () => {
        invalidateTextRegions();
        deactivateInput();
      },
      options,
    );
    target.addEventListener(
      "pointercancel",
      () => {
        textActivation = null;
        touchGesture = null;
      },
      options,
    );

    bridgedCanvas = target;
    bridgeAbort = controller;
  };

  const ensureInput = () => {
    if (input) {
      return input;
    }

    if (!document.body) {
      return null;
    }

    input = document.createElement("input");
    input.id = "material-ui-rs-mobile-keyboard";
    input.autocapitalize = "off";
    input.autocomplete = "off";
    input.autocorrect = "off";
    input.inputMode = "text";
    input.spellcheck = false;
    input.enterKeyHint = "done";
    input.tabIndex = -1;
    input.setAttribute("aria-label", "Text input");
    input.style.cssText = [
      "position:fixed",
      "left:0",
      "top:0",
      "width:1px",
      "height:1px",
      "font-size:16px",
      "opacity:0.01",
      "color:transparent",
      "background:transparent",
      "border:0",
      "outline:0",
      "padding:0",
      "margin:0",
      "caret-color:transparent",
      "pointer-events:none",
    ].join(";");

    input.addEventListener("keydown", (event) => {
      if (!active) {
        return;
      }

      if (
        composing ||
        event.isComposing ||
        event.key === "Process" ||
        event.keyCode === 229
      ) {
        return;
      }

      if (event.key === "Backspace" || event.key === "Delete") {
        schedulePendingAction(event.key);

        return;
      }

      if (
        event.key === "Enter" ||
        event.key === "Escape" ||
        event.key === "Tab" ||
        event.key === "ArrowLeft" ||
        event.key === "ArrowRight" ||
        event.key === "ArrowUp" ||
        event.key === "ArrowDown" ||
        event.key === "Home" ||
        event.key === "End"
      ) {
        sendKey(event.key);
        event.preventDefault();
      }
    });

    input.addEventListener("beforeinput", (event) => {
      if (!active || composing || compositionInput(event)) {
        return;
      }

      const action = inputAction(event.inputType);
      if (!action) {
        return;
      }

      clearSuppressedInput();

      if (event.cancelable) {
        event.preventDefault();
      } else {
        suppressFollowingInput();
      }

      clearPendingAction();
      sendKey(action);
      resetInput();
    });
    input.addEventListener("input", handleInput);
    input.addEventListener("compositionstart", () => {
      clearPendingAction();
      clearSuppressedInput();
      composing = true;
    });
    input.addEventListener("compositionend", (event) => {
      composing = false;
      clearPendingAction();

      if (!active || !input) {
        return;
      }

      const value = input.value;
      const valueText = value.startsWith(SENTINEL)
        ? value.slice(SENTINEL.length)
        : value;
      const text = event.data || valueText;

      if (text) {
        sendText(text);
        suppressFollowingInput();
      }

      resetInput();
    });
    input.addEventListener("blur", () => {
      clearPendingAction();
      clearSuppressedInput();
      const wasComposing = composing;
      composing = false;

      if (deactivating) {
        return;
      }

      const recentlyShown =
        Date.now() - lastShowAt <= REFOCUS_AFTER_SHOW_MS;

      if (active && recentlyShown && !wasComposing) {
        scheduleRefocus();
        return;
      }

      active = false;
      clearRefocus();
      resetInput();
      hideInputFromAssistiveTechnology();
    });

    document.body.appendChild(input);
    resetInput();
    hideInputFromAssistiveTechnology();

    return input;
  };

  const mountInput = () => {
    if (touchKeyboard()) {
      ensureInput();
      installCanvasActivationBridge();
    }
  };

  window.addEventListener("resize", invalidateTextRegions, {
    passive: true,
  });
  window.visualViewport?.addEventListener(
    "resize",
    () => {
      textActivation = null;
      touchGesture = null;
    },
    { passive: true },
  );

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mountInput, {
      once: true,
    });
  } else {
    mountInput();
  }

  const showMobileKeyboard = () => {
    if (!touchKeyboard()) {
      return;
    }

    activateInput();

    window.requestAnimationFrame(() => {
      if (!active || !input) {
        return;
      }

      if (document.activeElement !== input) {
        focusInput();
      }
    });
  };

  const registerTextRegion = (x, y, width, height) => {
    if (!touchKeyboard() || width <= 0 || height <= 0) {
      return;
    }

    if (!textRegionBatchOpen) {
      textRegionBatchOpen = true;
      textRegions = [];

      window.queueMicrotask(() => {
        textRegionBatchOpen = false;
      });
    }

    textRegions.push({ x, y, width, height });
    installCanvasActivationBridge();
  };

  const hideMobileKeyboard = () => {
    deactivateInput();
  };

  bridge = { hideMobileKeyboard, registerTextRegion, showMobileKeyboard };

  return bridge;
}

export function showMobileKeyboard() {
  mobileInput().showMobileKeyboard();
}

export function registerTextRegion(x, y, width, height) {
  mobileInput().registerTextRegion(x, y, width, height);
}

export function hideMobileKeyboard() {
  mobileInput().hideMobileKeyboard();
}
