import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  listAreas: vi.fn(),
  createArea: vi.fn(),
}));

vi.mock("@lifeos/contracts/bindings", () => ({
  commands: {
    listAreas: mocks.listAreas,
    createArea: mocks.createArea,
    undoAction: vi.fn(),
  },
}));

import { App } from "./main";

describe("LifeOS application shell", () => {
  afterEach(() => cleanup());
  beforeEach(() => {
    localStorage.clear();
    mocks.listAreas.mockResolvedValue({ status: "ok", data: [] });
    mocks.createArea.mockReset();
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
});
