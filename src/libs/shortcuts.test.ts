import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent } from "vue";

import { ShowStatus, useShow } from "@/stores/show/show";
import { useShowChanges } from "@/stores/show/show-changes";

import {
  SHORTCUT_REDO_SHIFT,
  SHORTCUT_REDO_Y,
  SHORTCUT_SAVE,
  SHORTCUT_UNDO,
  SHOW_SHORTCUTS,
  useGlobalShortcuts,
} from "./shortcuts";

const TestComponent = defineComponent({
  setup() {
    useGlobalShortcuts();
    return () => null;
  },
});

describe("useGlobalShortcuts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should register shortcuts on mount and unregister on unmount", async () => {
    const wrapper = mount(TestComponent);

    expect(register).toHaveBeenCalledWith(SHOW_SHORTCUTS, expect.any(Function));

    wrapper.unmount();
    expect(unregisterAll).toHaveBeenCalled();
  });

  it("should trigger undo on undo shortcut", async () => {
    const show = useShow();
    const changes = useShowChanges();
    show.state = ShowStatus.LOADED;

    mount(TestComponent);

    const handler = vi.mocked(register).mock.calls[0][1];
    handler({ shortcut: SHORTCUT_UNDO, state: "Pressed", id: 1 });

    expect(changes.undo).toHaveBeenCalledTimes(1);
  });

  it("should trigger redo on redo shortcuts", async () => {
    const show = useShow();
    const changes = useShowChanges();
    show.state = ShowStatus.LOADED;

    mount(TestComponent);

    const handler = vi.mocked(register).mock.calls[0][1];
    handler({ shortcut: SHORTCUT_REDO_SHIFT, state: "Pressed", id: 2 });
    expect(changes.redo).toHaveBeenCalledTimes(1);

    handler({ shortcut: SHORTCUT_REDO_Y, state: "Pressed", id: 3 });
    expect(changes.redo).toHaveBeenCalledTimes(2);
  });

  it("should trigger save on save shortcut when canSave is true", async () => {
    const show = useShow();
    const changes = useShowChanges();
    show.state = ShowStatus.LOADED;
    changes.isDirty = true;

    mount(TestComponent);

    const handler = vi.mocked(register).mock.calls[0][1];
    handler({ shortcut: SHORTCUT_SAVE, state: "Pressed", id: 4 });

    expect(changes.save).toHaveBeenCalledTimes(1);
  });

  it("should not trigger actions when state is Released or show is not loaded", async () => {
    const show = useShow();
    const changes = useShowChanges();
    show.state = ShowStatus.NOT_LOADED;

    mount(TestComponent);

    const handler = vi.mocked(register).mock.calls[0][1];

    // Ignored when not loaded
    handler({ shortcut: SHORTCUT_UNDO, state: "Pressed", id: 1 });
    expect(changes.undo).not.toHaveBeenCalled();

    show.state = ShowStatus.LOADED;

    // Ignored when state is Released
    handler({ shortcut: SHORTCUT_UNDO, state: "Released", id: 1 });
    expect(changes.undo).not.toHaveBeenCalled();
  });
});
