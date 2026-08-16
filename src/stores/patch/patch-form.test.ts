import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { mockFixturesData } from "@/stores/__mocks__/fixture-browser";
import { useFixtureBrowser } from "@/stores/fixture-browser";

import { usePatchForm } from "./patch-form";

describe("patch-form store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with undefined selection", () => {
    const store = usePatchForm();

    expect(store.selectedFixture).toBeUndefined();
    expect(store.selectedMode).toBeUndefined();
  });

  it("should update selected fixture data", () => {
    const fixtureBrowser = useFixtureBrowser();
    fixtureBrowser.manufacturers = mockFixturesData;

    const store = usePatchForm();

    store.select("chauvet", "gig-bar-2", 0);

    expect(store.selectedFixture?.manufacturerKey).toBe("chauvet");
    expect(store.selectedFixture?.fixtureKey).toBe("gig-bar-2");
    expect(store.selectedMode?.name).toBe("Standard");
  });
});
