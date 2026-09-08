import { fireEvent, render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useFixtureBrowser } from "@/stores/fixture-browser";

import SettingsGdtf from "./SettingsGdtf.vue";

describe("SettingsGdtf.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders unauthenticated state with login form", () => {
    const store = useFixtureBrowser();
    store.isAuthenticated = false;

    render(SettingsGdtf);

    expect(screen.getByText("GDTF-Share Library")).toBeInTheDocument();
    expect(screen.getByText("Not Connected")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("GDTF-Share username")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Password")).toBeInTheDocument();
  });

  it("calls setCredentials and syncCatalog on form submit", async () => {
    const store = useFixtureBrowser();
    store.isAuthenticated = false;
    store.setCredentials = vi.fn().mockResolvedValue(true);
    store.syncCatalog = vi.fn().mockResolvedValue(true);

    render(SettingsGdtf);

    const userInput = screen.getByPlaceholderText("GDTF-Share username");
    const passInput = screen.getByPlaceholderText("Password");
    const submitBtn = screen.getByRole("button", { name: /Connect & Sync/i });

    await fireEvent.update(userInput, "stage_tech");
    await fireEvent.update(passInput, "secret123");
    await fireEvent.click(submitBtn);

    expect(store.setCredentials).toHaveBeenCalledWith("stage_tech", "secret123");
    expect(store.syncCatalog).toHaveBeenCalled();
  });

  it("renders authenticated state and triggers sync and logout", async () => {
    const store = useFixtureBrowser();
    store.isAuthenticated = true;
    store.syncCatalog = vi.fn().mockResolvedValue(true);
    store.logout = vi.fn().mockResolvedValue(undefined);

    render(SettingsGdtf);

    expect(screen.getByText("Connected")).toBeInTheDocument();
    const syncBtn = screen.getByRole("button", { name: /Sync Online Catalog/i });
    await fireEvent.click(syncBtn);
    expect(store.syncCatalog).toHaveBeenCalled();

    const logoutBtn = screen.getByRole("button", { name: /Disconnect Account/i });
    await fireEvent.click(logoutBtn);
    expect(store.logout).toHaveBeenCalled();
  });

  it("displays error message when login fails", async () => {
    const store = useFixtureBrowser();
    store.isAuthenticated = false;
    store.setCredentials = vi.fn().mockResolvedValue(false);

    render(SettingsGdtf);

    const userInput = screen.getByPlaceholderText("GDTF-Share username");
    const passInput = screen.getByPlaceholderText("Password");
    const submitBtn = screen.getByRole("button", { name: /Connect & Sync/i });

    await fireEvent.update(userInput, "bad_user");
    await fireEvent.update(passInput, "wrong_pass");
    await fireEvent.click(submitBtn);

    expect(
      screen.getByText("Login failed. Please verify your username and password."),
    ).toBeInTheDocument();
  });
});
