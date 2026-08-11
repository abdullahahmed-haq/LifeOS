import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  appSettings: vi.fn(),
  updateAppSettings: vi.fn(),
  listAreas: vi.fn(),
  archiveArea: vi.fn(),
  createArea: vi.fn(),
  listTrashedAreas: vi.fn(),
  restoreArea: vi.fn(),
  trashArea: vi.fn(),
}));

vi.mock("@lifeos/contracts/bindings", () => ({
  commands: {
    appSettings: mocks.appSettings,
    updateAppSettings: mocks.updateAppSettings,
    listAreas: mocks.listAreas,
    listTrashedAreas: mocks.listTrashedAreas,
    createArea: mocks.createArea,
    archiveArea: mocks.archiveArea,
    trashArea: mocks.trashArea,
    restoreArea: mocks.restoreArea,
    undoAction: vi.fn(),
  },
}));

import { App } from "./main";

describe("LifeOS application shell", () => {
  afterEach(() => cleanup());
  beforeEach(() => {
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
    mocks.listTrashedAreas.mockResolvedValue({ status: "ok", data: [] });
    mocks.createArea.mockReset();
    mocks.archiveArea.mockReset();
    mocks.trashArea.mockReset();
    mocks.restoreArea.mockReset();
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
});
