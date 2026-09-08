import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useApp } from "@/stores/app.ts";
import { useNavigation } from "@/stores/navigation.ts";
import { ShowStatus, useShow } from "@/stores/show/show.ts";
import { useShowChanges } from "@/stores/show/show-changes.ts";
import { useShowIdentity } from "@/stores/show/show-identity.ts";

import AppSidebar from "./AppSidebar.vue";

describe("AppSidebar.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render show title and menu actions", () => {
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;
    showStore.identity.displayName = "My Stage Show";

    render(AppSidebar);

    expect(screen.getByText("My Stage Show")).toBeInTheDocument();
    expect(screen.getByText("New show")).toBeInTheDocument();
    expect(screen.getByText("Open show")).toBeInTheDocument();
    expect(screen.getByText("Save show")).toBeInTheDocument();
    expect(screen.getByText("Close show")).toBeInTheDocument();
  });

  it("should call app.createShow when New show is clicked", async () => {
    const user = userEvent.setup();
    const appStore = useApp();

    render(AppSidebar);

    await user.click(screen.getByText("New show"));

    expect(appStore.createShow).toHaveBeenCalledTimes(1);
  });

  it("should call app.askLoadShow when Open show is clicked", async () => {
    const user = userEvent.setup();
    const appStore = useApp();

    render(AppSidebar);

    await user.click(screen.getByText("Open show"));

    expect(appStore.askLoadShow).toHaveBeenCalledTimes(1);
  });

  it("should call changes.save when Save show is clicked", async () => {
    const user = userEvent.setup();
    const changesStore = useShowChanges();
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;
    changesStore.path = undefined; // new show -> canSave is true

    render(AppSidebar);

    await user.click(screen.getByText("Save show"));

    expect(changesStore.save).toHaveBeenCalledTimes(1);
  });

  it("should call changes.undo when Undo is clicked and enabled", async () => {
    const user = userEvent.setup();
    const changesStore = useShowChanges();
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;
    changesStore.canUndo = true;

    render(AppSidebar);

    await user.click(screen.getByText("Undo"));

    expect(changesStore.undo).toHaveBeenCalledTimes(1);
  });

  it("should call changes.redo when Redo is clicked and enabled", async () => {
    const user = userEvent.setup();
    const changesStore = useShowChanges();
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;
    changesStore.canRedo = true;

    render(AppSidebar);

    await user.click(screen.getByText("Redo"));

    expect(changesStore.redo).toHaveBeenCalledTimes(1);
  });

  it("should call app.closeShow when Close show is clicked", async () => {
    const user = userEvent.setup();
    const appStore = useApp();

    render(AppSidebar);

    await user.click(screen.getByText("Close show"));

    expect(appStore.closeShow).toHaveBeenCalledTimes(1);
  });

  it("should navigate to selected window when page item is clicked", async () => {
    const user = userEvent.setup();
    const navStore = useNavigation();

    render(AppSidebar);

    const patchPageLink = screen.getByText("Patch");
    await user.click(patchPageLink);

    expect(navStore.navigate).toHaveBeenCalledWith("patch");

    const intensityPageLink = screen.getByText("Intensity");
    await user.click(intensityPageLink);

    expect(navStore.navigate).toHaveBeenCalledWith("intensity");
  });

  it("should allow editing and saving show name", async () => {
    const user = userEvent.setup();
    const identityStore = useShowIdentity();
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;
    identityStore.displayName = "Original Name";

    render(AppSidebar);

    const editBtn = screen.getByRole("button", { name: "Edit show name" });
    await user.click(editBtn);

    const input = screen.getByRole("textbox");
    expect(input).toHaveValue("Original Name");

    await user.clear(input);
    await user.type(input, "New Show Name");

    const saveBtn = screen.getByRole("button", { name: "Save" });
    await user.click(saveBtn);

    expect(identityStore.displayName).toBe("New Show Name");
  });

  it("should allow canceling show name editing", async () => {
    const user = userEvent.setup();
    const identityStore = useShowIdentity();
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;
    identityStore.displayName = "Original Name";

    render(AppSidebar);

    const editBtn = screen.getByRole("button", { name: "Edit show name" });
    await user.click(editBtn);

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "Canceled Name");

    const cancelBtn = screen.getByRole("button", { name: "Cancel" });
    await user.click(cancelBtn);

    expect(identityStore.displayName).toBe("Original Name");
  });

  it("should toggle between expanded and collapsed states when hide/show button is clicked", async () => {
    const user = userEvent.setup();
    const appStore = useApp();
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;
    showStore.identity.displayName = "My Stage Show";

    render(AppSidebar);

    // Initially expanded
    expect(screen.getByText("My Stage Show")).toBeInTheDocument();
    expect(screen.getByText("Show")).toBeInTheDocument();
    expect(screen.getByText("Setup")).toBeInTheDocument();
    expect(screen.getByText("Live")).toBeInTheDocument();
    expect(screen.getByText("New show")).toBeInTheDocument();
    const hideBtn = screen.getByRole("button", { name: "Hide sidebar" });
    expect(hideBtn).toBeInTheDocument();

    // Click hide button to collapse
    await user.click(hideBtn);

    // Labels and titles should be hidden
    expect(screen.queryByText("My Stage Show")).not.toBeInTheDocument();
    expect(screen.queryByText("Show")).not.toBeInTheDocument();
    expect(screen.queryByText("Setup")).not.toBeInTheDocument();
    expect(screen.queryByText("Live")).not.toBeInTheDocument();
    expect(screen.queryByText("New show")).not.toBeInTheDocument();
    expect(screen.queryByText("Hide")).not.toBeInTheDocument();

    // Buttons are still present and functional
    const newShowBtn = screen.getByRole("button", { name: "New show" });
    expect(newShowBtn).toBeInTheDocument();
    await user.click(newShowBtn);
    expect(appStore.createShow).toHaveBeenCalledTimes(1);

    // Expand sidebar again
    const showBtn = screen.getByRole("button", { name: "Show sidebar" });
    expect(showBtn).toBeInTheDocument();
    await user.click(showBtn);

    // Labels and titles are visible again
    expect(screen.getByText("My Stage Show")).toBeInTheDocument();
    expect(screen.getByText("Show")).toBeInTheDocument();
    expect(screen.getByText("Setup")).toBeInTheDocument();
    expect(screen.getByText("Live")).toBeInTheDocument();
    expect(screen.getByText("New show")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Hide sidebar" })).toBeInTheDocument();
  });

  it("should cancel show name editing when sidebar is collapsed", async () => {
    const user = userEvent.setup();
    const identityStore = useShowIdentity();
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;
    showStore.identity.displayName = "My Stage Show";

    render(AppSidebar);

    const editBtn = screen.getByRole("button", { name: "Edit show name" });
    await user.click(editBtn);
    expect(identityStore.nameForm.isEditing).toBe(true);

    const hideBtn = screen.getByRole("button", { name: "Hide sidebar" });
    await user.click(hideBtn);

    expect(identityStore.nameForm.isEditing).toBe(false);
  });
});
