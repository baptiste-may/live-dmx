import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { useShowChanges } from "./show-changes";
import { useShowIdentity } from "./show-identity";

describe("show-identity store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with default display name", () => {
    const store = useShowIdentity();

    expect(store.displayName).toBe("My show");
  });

  it("should invoke backend command, update state and set isDirty when updateDisplayName is called", async () => {
    const store = useShowIdentity();
    const changes = useShowChanges();
    expect(changes.isDirty).toBe(false);

    await store.updateDisplayName("Concert 2026");

    expect(store.displayName).toBe("Concert 2026");
    expect(changes.isDirty).toBe(true);
    expect(invoke).toHaveBeenCalledWith("show_identity_update", {
      displayName: "Concert 2026",
    });
  });
});
