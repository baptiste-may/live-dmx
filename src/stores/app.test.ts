import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { useApp } from "./app";
import { ShowStatus, useShow } from "./show/show";

describe("app store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should report isShowLoaded state based on show store", () => {
    const app = useApp();
    const show = useShow();

    expect(app.isShowLoaded).toBe(false);

    show.state = ShowStatus.LOADED;
    expect(app.isShowLoaded).toBe(true);
  });

  it("should create new show and set loaded state", async () => {
    const app = useApp();
    const show = useShow();

    await app.createShow();

    expect(show.isLoaded).toBe(true);
  });

  it("should load show from path and set loaded state", async () => {
    const app = useApp();
    const show = useShow();

    await app.loadShow("/path/to/show.json");

    expect(show.isLoaded).toBe(true);
  });

  it("should unload show when closeShow is called", async () => {
    const app = useApp();
    const show = useShow();

    await app.createShow();
    expect(show.isLoaded).toBe(true);

    app.closeShow();
    expect(show.isLoaded).toBe(false);
  });
});
