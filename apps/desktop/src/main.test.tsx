import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  appSettings: vi.fn(),
  updateAppSettings: vi.fn(),
  listAreas: vi.fn(),
  listGoals: vi.fn(),
  listArchivedAreas: vi.fn(),
  archiveArea: vi.fn(),
  areaHistory: vi.fn(),
  auditEntries: vi.fn(),
  createArea: vi.fn(),
  createGoal: vi.fn(),
  listTrashedAreas: vi.fn(),
  restoreArea: vi.fn(),
  trashArea: vi.fn(),
  updateArea: vi.fn(),
}));

vi.mock("@lifeos/contracts/bindings", () => ({
  commands: {
    appSettings: mocks.appSettings,
    updateAppSettings: mocks.updateAppSettings,
    listAreas: mocks.listAreas,
    listGoals: mocks.listGoals,
    listArchivedAreas: mocks.listArchivedAreas,
    listTrashedAreas: mocks.listTrashedAreas,
    createArea: mocks.createArea,
    createGoal: mocks.createGoal,
    archiveArea: mocks.archiveArea,
    areaHistory: mocks.areaHistory,
    auditEntries: mocks.auditEntries,
    trashArea: mocks.trashArea,
    restoreArea: mocks.restoreArea,
    undoAction: vi.fn(),
    updateArea: mocks.updateArea,
  },
}));

import { App } from "./main";

describe("LifeOS application shell", () => {
  afterEach(() => {
    cleanup();
    window.history.pushState(null, "", "/");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  beforeEach(() => {
    window.history.pushState(null, "", "/");
    window.dispatchEvent(new PopStateEvent("popstate"));
    localStorage.clear();
    mocks.appSettings.mockResolvedValue({
      status: "ok",
      data: {
        locale: "en",
        theme: "system",
        timezone: "Africa/Cairo",
        weekStartsOn: 1,
        revision: 1,
      },
    });
    mocks.updateAppSettings.mockReset();
    mocks.updateAppSettings.mockImplementation(async (request) => ({
      status: "ok",
      data: {
        data: {
          locale: request.locale,
          theme: request.theme,
          timezone: request.timezone,
          weekStartsOn: request.weekStartsOn,
          revision: request.expectedRevision + 1,
        },
        operationId: request.operationId,
        affectedEntityIds: [],
        resultingRevisions: [],
        domainEventIds: ["settings-event"],
        undoBatchId: null,
      },
    }));
    mocks.listAreas.mockResolvedValue({ status: "ok", data: [] });
    mocks.listGoals.mockResolvedValue({ status: "ok", data: [] });
    mocks.listArchivedAreas.mockResolvedValue({ status: "ok", data: [] });
    mocks.listTrashedAreas.mockResolvedValue({ status: "ok", data: [] });
    mocks.createArea.mockReset();
    mocks.createGoal.mockReset();
    mocks.archiveArea.mockReset();
    mocks.areaHistory.mockReset();
    mocks.auditEntries.mockResolvedValue({ status: "ok", data: [] });
    mocks.trashArea.mockReset();
    mocks.restoreArea.mockReset();
    mocks.updateArea.mockReset();
  });

  it("renders the empty Areas state and switches the document to Arabic RTL", async () => {
    const user = userEvent.setup();
    render(<App />);

    expect(
      await screen.findByRole("heading", { name: "No areas yet" }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "العربية" }));

    expect(
      await screen.findByRole("heading", { name: "لا توجد مجالات بعد" }),
    ).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("ar");
    expect(document.documentElement.dir).toBe("rtl");
  });

  it("keeps the draft and reports an error when area creation fails", async () => {
    const user = userEvent.setup();
    mocks.createArea.mockResolvedValue({
      status: "error",
      error: { code: "INTERNAL", details: { operationId: "test-operation" } },
    });
    render(<App />);

    const input = await screen.findByRole("textbox", { name: "Area name" });
    await user.type(input, "Important draft");
    await user.click(screen.getByRole("button", { name: "Create area" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Something went wrong. Your local data remains safe.",
    );
    expect(input).toHaveValue("Important draft");
  });

  it("clears the draft only after canonical area creation succeeds", async () => {
    const user = userEvent.setup();
    mocks.createArea.mockResolvedValue({
      status: "ok",
      data: {
        data: {
          id: "018f0000-0000-7000-8000-000000000001",
          title: "Health",
          revision: 1,
          createdAtMs: "1",
          updatedAtMs: "1",
        },
        operationId: "create-health",
        affectedEntityIds: ["018f0000-0000-7000-8000-000000000001"],
        resultingRevisions: [
          {
            entityId: "018f0000-0000-7000-8000-000000000001",
            revision: 1,
          },
        ],
        domainEventIds: ["event-1"],
        undoBatchId: "undo-1",
      },
    });
    render(<App />);

    const input = await screen.findByRole("textbox", { name: "Area name" });
    await user.type(input, "Health");
    await user.click(screen.getByRole("button", { name: "Create area" }));

    await waitFor(() => expect(input).toHaveValue(""));
    expect(screen.getByRole("status")).toHaveTextContent("Area created");
  });

  it("opens the command palette from the keyboard", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole("heading", { name: "No areas yet" });
    await user.keyboard("{Control>}k{/Control}");

    expect(
      await screen.findByRole("heading", { name: "Command palette" }),
    ).toBeInTheDocument();
  });

  it("sends an archive command with the canonical revision", async () => {
    const user = userEvent.setup();
    mocks.listAreas.mockResolvedValue({
      status: "ok",
      data: [
        {
          id: "018f0000-0000-7000-8000-000000000001",
          title: "Health",
          revision: 3,
          createdAtMs: "1",
          updatedAtMs: "2",
          archivedAtMs: null,
          deletedAtMs: null,
        },
      ],
    });
    mocks.archiveArea.mockResolvedValue({
      status: "ok",
      data: {
        data: {},
        operationId: "area-archive",
        affectedEntityIds: [],
        resultingRevisions: [],
        domainEventIds: [],
        undoBatchId: "undo-archive",
      },
    });
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "Archive" }));

    expect(mocks.archiveArea).toHaveBeenCalledWith(
      expect.objectContaining({
        id: "018f0000-0000-7000-8000-000000000001",
        expectedRevision: 3,
      }),
    );
  });

  it("edits an Area through the revision-checked canonical command", async () => {
    const user = userEvent.setup();
    mocks.listAreas.mockResolvedValue({
      status: "ok",
      data: [
        {
          id: "018f0000-0000-7000-8000-000000000001",
          title: "Health",
          revision: 3,
          createdAtMs: "1",
          updatedAtMs: "2",
          archivedAtMs: null,
          deletedAtMs: null,
        },
      ],
    });
    mocks.updateArea.mockResolvedValue({
      status: "ok",
      data: {
        data: {},
        operationId: "area-update",
        affectedEntityIds: [],
        resultingRevisions: [],
        domainEventIds: [],
        undoBatchId: "undo-update",
      },
    });
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "Edit" }));
    const editForm = screen.getByRole("form", { name: "Edit area" });
    const input = within(editForm).getByRole("textbox", { name: "Area name" });
    await user.clear(input);
    await user.type(input, "Wellbeing");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(mocks.updateArea).toHaveBeenCalledWith(
      expect.objectContaining({
        id: "018f0000-0000-7000-8000-000000000001",
        title: "Wellbeing",
        expectedRevision: 3,
      }),
    );
  });

  it("restores an archived Area through the Archive route", async () => {
    const user = userEvent.setup();
    mocks.listArchivedAreas.mockResolvedValue({
      status: "ok",
      data: [
        {
          id: "018f0000-0000-7000-8000-000000000002",
          title: "Health",
          revision: 4,
          createdAtMs: "1",
          updatedAtMs: "2",
          archivedAtMs: "2",
          deletedAtMs: null,
        },
      ],
    });
    mocks.restoreArea.mockResolvedValue({ status: "ok", data: {} });
    render(<App />);

    await user.click(await screen.findByRole("link", { name: "Archive" }));
    await user.click(
      await screen.findByRole("button", { name: "Restore area" }),
    );

    expect(mocks.restoreArea).toHaveBeenCalledWith(
      expect.objectContaining({
        id: "018f0000-0000-7000-8000-000000000002",
        expectedRevision: 4,
      }),
    );
  });

  it("shows newest-first bounded canonical Area history", async () => {
    const user = userEvent.setup();
    mocks.listAreas.mockResolvedValue({
      status: "ok",
      data: [
        {
          id: "018f0000-0000-7000-8000-000000000003",
          title: "Health",
          revision: 3,
          createdAtMs: "1",
          updatedAtMs: "3",
          archivedAtMs: null,
          deletedAtMs: null,
        },
      ],
    });
    mocks.areaHistory.mockResolvedValue({
      status: "ok",
      data: [
        {
          revision: 3,
          title: "Wellbeing",
          operationId: "update",
          createdAtMs: "3",
        },
        {
          revision: 1,
          title: "Health",
          operationId: "create",
          createdAtMs: "1",
        },
      ],
    });
    render(<App />);

    await user.click(await screen.findByRole("link", { name: "Areas" }));
    await user.click(await screen.findByRole("link", { name: "History" }));

    expect(
      await screen.findByRole("heading", { name: "Version history" }),
    ).toBeInTheDocument();
    expect(mocks.areaHistory).toHaveBeenCalledWith(
      expect.objectContaining({
        id: "018f0000-0000-7000-8000-000000000003",
        limit: 100,
      }),
    );
    expect(
      screen.getAllByRole("listitem").map((item) => item.textContent),
    ).toEqual([expect.stringContaining("v3"), expect.stringContaining("v1")]);
  });

  it("persists planning preferences through the settings route", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("link", { name: "Settings" }));
    const timezone = await screen.findByRole("textbox", { name: "Timezone" });
    await user.clear(timezone);
    await user.type(timezone, "Asia/Riyadh");
    await user.click(screen.getByRole("button", { name: "Save preferences" }));

    expect(mocks.updateAppSettings).toHaveBeenCalledWith(
      expect.objectContaining({
        timezone: "Asia/Riyadh",
        expectedRevision: 1,
      }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Preferences saved",
    );
  });

  it("loads the bounded audit timeline through the Audit route", async () => {
    const user = userEvent.setup();
    mocks.auditEntries.mockResolvedValue({
      status: "ok",
      data: [
        {
          id: "018f0000-0000-7000-8000-000000000004",
          action: "area.updated",
          actorKind: "user",
          occurredAtMs: "0",
        },
      ],
    });
    render(<App />);

    await user.click(await screen.findByRole("link", { name: "Audit" }));

    expect(
      await screen.findByRole("heading", { name: "Audit timeline" }),
    ).toBeInTheDocument();
    expect(mocks.auditEntries).toHaveBeenCalledWith({ limit: 100 });
    expect(screen.getByRole("listitem")).toHaveTextContent("area.updated");
  });

  it("creates a Goal through the canonical typed command", async () => {
    const user = userEvent.setup();
    mocks.createGoal.mockResolvedValue({
      status: "ok",
      data: {
        data: {
          id: "018f0000-0000-7000-8000-000000000005",
          title: "Learn Arabic",
          horizon: "long",
          status: "active",
          startDate: "2026-08-12",
          targetDate: "2027-08-12",
          revision: 1,
          createdAtMs: "1",
          updatedAtMs: "1",
        },
        operationId: "goal-create",
        affectedEntityIds: ["018f0000-0000-7000-8000-000000000005"],
        resultingRevisions: [],
        domainEventIds: ["goal-event"],
        undoBatchId: "goal-undo",
      },
    });
    render(<App />);

    await user.click(await screen.findByRole("link", { name: "Goals" }));
    await user.type(
      await screen.findByRole("textbox", { name: "Goal name" }),
      "Learn Arabic",
    );
    await user.selectOptions(screen.getByLabelText("Horizon"), "long");
    fireEvent.change(screen.getByLabelText("Start date"), {
      target: { value: "2026-08-12" },
    });
    fireEvent.change(screen.getByLabelText("Target date"), {
      target: { value: "2027-08-12" },
    });
    await user.click(screen.getByRole("button", { name: "Create goal" }));

    expect(mocks.createGoal).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Learn Arabic",
        horizon: "long",
        startDate: "2026-08-12",
        targetDate: "2027-08-12",
      }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent("Goal created");
  });
});
